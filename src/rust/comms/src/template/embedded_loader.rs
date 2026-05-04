use std::collections::HashMap;

use crate::error::CommsError;
use crate::template::TemplateLoader;

/// Loader backed by an in-memory map. Pair with `include_str!` to ship
/// compiled-in defaults that don't need filesystem access at runtime.
#[derive(Debug, Default, Clone)]
pub struct EmbeddedTemplateLoader {
    sources: HashMap<String, String>,
}

impl EmbeddedTemplateLoader {
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct from a slice of `(name, toml_source)` pairs. Common pattern:
    /// `EmbeddedTemplateLoader::from_pairs(&[("password_reset", include_str!("../templates/password_reset.toml"))])`.
    pub fn from_pairs(pairs: &[(&str, &str)]) -> Self {
        let sources = pairs
            .iter()
            .map(|(name, src)| ((*name).to_owned(), (*src).to_owned()))
            .collect();
        Self { sources }
    }

    pub fn insert(&mut self, name: impl Into<String>, source: impl Into<String>) {
        self.sources.insert(name.into(), source.into());
    }
}

impl TemplateLoader for EmbeddedTemplateLoader {
    fn load(&self, name: &str) -> Result<String, CommsError> {
        self.sources
            .get(name)
            .cloned()
            .ok_or_else(|| CommsError::TemplateNotFound(name.to_owned()))
    }

    fn names(&self) -> Vec<String> {
        let mut out: Vec<String> = self.sources.keys().cloned().collect();
        out.sort();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn from_pairs_round_trips() {
        let loader = EmbeddedTemplateLoader::from_pairs(&[
            ("a", "channel = \"email\"\nsubject = \"\"\ntext = \"\""),
            ("b", "channel = \"sms\"\ntext = \"\""),
        ]);
        assert_eq!(loader.names(), vec!["a".to_owned(), "b".to_owned()]);
        assert_eq!(
            loader.load("a").unwrap(),
            "channel = \"email\"\nsubject = \"\"\ntext = \"\""
        );
    }

    #[test]
    fn missing_template_returns_not_found() {
        let loader = EmbeddedTemplateLoader::new();
        let err = loader.load("absent").expect_err("must miss");
        assert!(matches!(err, CommsError::TemplateNotFound(name) if name == "absent"));
    }
}
