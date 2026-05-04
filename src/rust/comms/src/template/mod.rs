pub mod embedded_loader;
pub mod fs_loader;
pub mod layered_loader;
pub mod renderer;

pub use embedded_loader::EmbeddedTemplateLoader;
pub use fs_loader::FsTemplateLoader;
pub use layered_loader::LayeredTemplateLoader;
pub use renderer::{
    AppContext, BrandingContext, RenderedTemplate, TemplateContext, TemplateRenderer,
};

use serde::{Deserialize, Serialize};

use crate::error::CommsError;

/// Which medium a template targets. Determines which fields are required and
/// which `*Message` shape the renderer produces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Channel {
    Email,
    Sms,
}

/// Raw, unvalidated TOML form of a template file. Internal; the public API
/// works through `TemplateSource` which has been validated for consistency
/// with its declared channel.
#[derive(Debug, Clone, Deserialize)]
struct TemplateFile {
    channel: Channel,
    subject: Option<String>,
    text: String,
    html: Option<String>,
}

/// A loaded, channel-validated template. The strings are raw `minijinja`
/// source — substitution happens in the renderer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateSource {
    pub channel: Channel,
    pub subject: Option<String>,
    pub text: String,
    pub html: Option<String>,
}

impl TemplateSource {
    /// Parse and validate a template TOML document.
    ///
    /// Email templates require `subject` and `text`, accept optional `html`.
    /// SMS templates require `text` only and reject `subject` / `html`.
    pub fn parse(name: &str, toml_text: &str) -> Result<Self, CommsError> {
        let file: TemplateFile = toml::from_str(toml_text)
            .map_err(|e| CommsError::Config(format!("template '{name}': {e}")))?;

        match file.channel {
            Channel::Email => {
                if file.subject.is_none() {
                    return Err(CommsError::Config(format!(
                        "template '{name}': email channel requires a 'subject' field"
                    )));
                }
            }
            Channel::Sms => {
                if file.subject.is_some() {
                    return Err(CommsError::Config(format!(
                        "template '{name}': sms channel must not carry a 'subject' field"
                    )));
                }
                if file.html.is_some() {
                    return Err(CommsError::Config(format!(
                        "template '{name}': sms channel must not carry an 'html' field"
                    )));
                }
            }
        }

        Ok(Self {
            channel: file.channel,
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
    fn parse_accepts_email_with_subject_text_and_html() {
        let toml = r#"
            channel = "email"
            subject = "Hi {{ name }}"
            text = "Hello {{ name }}"
            html = "<p>Hello {{ name }}</p>"
        "#;
        let t = TemplateSource::parse("greet", toml).expect("parse");
        assert_eq!(t.channel, Channel::Email);
        assert_eq!(t.subject.as_deref(), Some("Hi {{ name }}"));
        assert_eq!(t.text, "Hello {{ name }}");
        assert_eq!(t.html.as_deref(), Some("<p>Hello {{ name }}</p>"));
    }

    #[test]
    fn parse_accepts_email_without_html() {
        let toml = r#"
            channel = "email"
            subject = "Hi"
            text = "body"
        "#;
        let t = TemplateSource::parse("greet", toml).expect("parse");
        assert_eq!(t.html, None);
    }

    #[test]
    fn parse_rejects_email_without_subject() {
        let toml = r#"
            channel = "email"
            text = "body"
        "#;
        let err = TemplateSource::parse("greet", toml).expect_err("must reject");
        assert!(err.to_string().contains("requires a 'subject'"), "{err}");
    }

    #[test]
    fn parse_accepts_sms_with_text_only() {
        let toml = r#"
            channel = "sms"
            text = "Maze: {{ link }}"
        "#;
        let t = TemplateSource::parse("ping", toml).expect("parse");
        assert_eq!(t.channel, Channel::Sms);
        assert_eq!(t.subject, None);
        assert_eq!(t.html, None);
    }

    #[test]
    fn parse_rejects_sms_with_subject() {
        let toml = r#"
            channel = "sms"
            subject = "uh oh"
            text = "body"
        "#;
        let err = TemplateSource::parse("ping", toml).expect_err("must reject");
        assert!(err.to_string().contains("must not carry a 'subject'"), "{err}");
    }

    #[test]
    fn parse_rejects_sms_with_html() {
        let toml = r#"
            channel = "sms"
            text = "body"
            html = "<p>nope</p>"
        "#;
        let err = TemplateSource::parse("ping", toml).expect_err("must reject");
        assert!(err.to_string().contains("must not carry an 'html'"), "{err}");
    }

    #[test]
    fn parse_rejects_invalid_channel() {
        let toml = r#"
            channel = "carrier_pigeon"
            text = "body"
        "#;
        let err = TemplateSource::parse("strange", toml).expect_err("must reject");
        assert!(err.to_string().contains("template 'strange'"), "{err}");
    }
}
