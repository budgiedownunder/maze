pub mod embedded_loader;
pub mod fs_loader;
pub mod layered_loader;
pub mod renderer;

pub use embedded_loader::EmbeddedTemplateLoader;
pub use fs_loader::FsTemplateLoader;
pub use layered_loader::LayeredTemplateLoader;
pub use renderer::{
    AppContext, BrandingContext, BrandingPartialSources, RenderedTemplate, TemplateContext,
    TemplateRenderer,
};

use serde::Deserialize;

use crate::error::CommsError;

/// Raw, unvalidated TOML form of a template file. Internal; the public API
/// works through `TemplateSource` which has been validated.
#[derive(Debug, Clone, Deserialize)]
struct TemplateFile {
    subject: Option<String>,
    text: String,
    html: Option<String>,
}

/// A loaded, validated template. The strings are raw `minijinja` source —
/// substitution happens in the renderer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateSource {
    pub subject: Option<String>,
    pub text: String,
    pub html: Option<String>,
}

impl TemplateSource {
    /// Parse and validate a template TOML document. Templates require
    /// `subject` and `text`; `html` is optional. Unknown top-level TOML
    /// fields are silently ignored, so an operator can annotate a template
    /// file with metadata fields the parser doesn't recognise without
    /// breaking the load.
    ///
    /// # Examples
    ///
    /// ```
    /// use comms::TemplateSource;
    ///
    /// let toml = r#"
    ///     subject = "Hi {{ name }}"
    ///     text = "Hello {{ name }}"
    /// "#;
    /// let t = TemplateSource::parse("greet", toml).expect("parse");
    /// assert_eq!(t.subject.as_deref(), Some("Hi {{ name }}"));
    /// assert_eq!(t.text, "Hello {{ name }}");
    /// assert!(t.html.is_none());
    /// ```
    pub fn parse(name: &str, toml_text: &str) -> Result<Self, CommsError> {
        let file: TemplateFile = toml::from_str(toml_text)
            .map_err(|e| CommsError::Config(format!("template '{name}': {e}")))?;

        if file.subject.is_none() {
            return Err(CommsError::Config(format!(
                "template '{name}': a 'subject' field is required"
            )));
        }

        Ok(Self {
            subject: file.subject,
            text: file.text,
            html: file.html,
        })
    }
}

/// Source of named template documents. Implementations supply raw TOML; the
/// renderer parses, validates, and renders.
pub trait TemplateLoader: Send + Sync {
    /// Return the TOML source of the named template, or `TemplateNotFound`.
    fn load(&self, name: &str) -> Result<String, CommsError>;

    /// Names of every template this loader can supply. Used at renderer init
    /// to enumerate all templates and surface validation errors up front.
    fn names(&self) -> Vec<String>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_accepts_template_with_subject_text_and_html() {
        let toml = r#"
            subject = "Hi {{ name }}"
            text = "Hello {{ name }}"
            html = "<p>Hello {{ name }}</p>"
        "#;
        let t = TemplateSource::parse("greet", toml).expect("parse");
        assert_eq!(t.subject.as_deref(), Some("Hi {{ name }}"));
        assert_eq!(t.text, "Hello {{ name }}");
        assert_eq!(t.html.as_deref(), Some("<p>Hello {{ name }}</p>"));
    }

    #[test]
    fn parse_accepts_template_without_html() {
        let toml = r#"
            subject = "Hi"
            text = "body"
        "#;
        let t = TemplateSource::parse("greet", toml).expect("parse");
        assert_eq!(t.html, None);
    }

    #[test]
    fn parse_rejects_template_without_subject() {
        let toml = r#"
            text = "body"
        "#;
        let err = TemplateSource::parse("greet", toml).expect_err("must reject");
        let msg = err.to_string();
        assert!(msg.contains("subject"), "{msg}");
        assert!(msg.contains("required"), "{msg}");
    }

    /// Unknown top-level TOML fields are silently ignored. This keeps the
    /// parser forgiving: an operator can annotate a template file with
    /// arbitrary metadata fields without breaking the load.
    #[test]
    fn parse_ignores_unknown_top_level_fields() {
        let toml = r#"
            comment = "hand-edited 2026-04-09 by @cbudg"
            subject = "Hi"
            text = "body"
        "#;
        let t = TemplateSource::parse("greet", toml).expect("parse");
        assert_eq!(t.subject.as_deref(), Some("Hi"));
        assert_eq!(t.text, "body");
    }
}
