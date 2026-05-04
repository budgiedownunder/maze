use std::collections::BTreeSet;
use std::sync::Arc;

use crate::error::CommsError;
use crate::template::TemplateLoader;

/// Loader that consults an FS-backed override layer first, then falls back to
/// an embedded defaults layer. A template name present in both is served from
/// the FS layer, allowing operators to override compiled-in defaults without
/// rebuilding.
pub struct LayeredTemplateLoader {
    /// Optional FS-backed overrides. `None` if the consumer doesn't expose an
    /// override directory.
    pub fs: Option<Arc<dyn TemplateLoader>>,
    /// Embedded defaults compiled into the binary.
    pub embedded: Arc<dyn TemplateLoader>,
}

impl LayeredTemplateLoader {
    pub fn new(
        fs: Option<Arc<dyn TemplateLoader>>,
        embedded: Arc<dyn TemplateLoader>,
    ) -> Self {
        Self { fs, embedded }
    }
}

impl TemplateLoader for LayeredTemplateLoader {
    fn load(&self, name: &str) -> Result<String, CommsError> {
        if let Some(fs) = &self.fs {
            match fs.load(name) {
                Ok(src) => return Ok(src),
                Err(CommsError::TemplateNotFound(_)) => {}
                Err(e) => return Err(e),
            }
        }
        self.embedded.load(name)
    }

    fn names(&self) -> Vec<String> {
        let mut combined: BTreeSet<String> = self.embedded.names().into_iter().collect();
        if let Some(fs) = &self.fs {
            combined.extend(fs.names());
        }
        combined.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::embedded_loader::EmbeddedTemplateLoader;
    use pretty_assertions::assert_eq;

    #[test]
    fn fs_overrides_embedded_for_same_name() {
        let embedded = Arc::new(EmbeddedTemplateLoader::from_pairs(&[
            ("greet", "channel = \"email\"\nsubject = \"baked\"\ntext = \"\""),
        ]));
        let fs = Arc::new(EmbeddedTemplateLoader::from_pairs(&[
            ("greet", "channel = \"email\"\nsubject = \"override\"\ntext = \"\""),
        ]));
        let layered = LayeredTemplateLoader::new(Some(fs), embedded);
        assert!(layered.load("greet").unwrap().contains("override"));
    }

    #[test]
    fn falls_back_to_embedded_when_fs_misses() {
        let embedded = Arc::new(EmbeddedTemplateLoader::from_pairs(&[
            ("only_embedded", "channel = \"email\"\nsubject = \"e\"\ntext = \"\""),
        ]));
        let fs = Arc::new(EmbeddedTemplateLoader::new());
        let layered = LayeredTemplateLoader::new(Some(fs), embedded);
        assert!(layered.load("only_embedded").unwrap().contains("subject = \"e\""));
    }

    #[test]
    fn names_merge_and_dedupe_across_layers() {
        let embedded = Arc::new(EmbeddedTemplateLoader::from_pairs(&[
            ("a", ""),
            ("b", ""),
        ]));
        let fs = Arc::new(EmbeddedTemplateLoader::from_pairs(&[
            ("b", ""),
            ("c", ""),
        ]));
        let layered = LayeredTemplateLoader::new(Some(fs), embedded);
        assert_eq!(
            layered.names(),
            vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]
        );
    }

    #[test]
    fn missing_in_both_returns_not_found() {
        let embedded = Arc::new(EmbeddedTemplateLoader::new());
        let layered = LayeredTemplateLoader::new(None, embedded);
        let err = layered.load("absent").expect_err("must miss");
        assert!(matches!(err, CommsError::TemplateNotFound(n) if n == "absent"));
    }

    #[test]
    fn no_fs_layer_falls_through_to_embedded() {
        let embedded = Arc::new(EmbeddedTemplateLoader::from_pairs(&[
            ("hello", "channel = \"sms\"\ntext = \"hi\""),
        ]));
        let layered = LayeredTemplateLoader::new(None, embedded);
        assert!(layered.load("hello").unwrap().contains("channel = \"sms\""));
    }
}
