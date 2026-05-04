use thiserror::Error;

/// Error taxonomy for the `comms` crate.
///
/// Variants are split by `is_transient()` so the retry policy can decide
/// whether to attempt again. Provider implementations map their own
/// HTTP / SMTP statuses into these variants.
#[derive(Error, Debug)]
pub enum CommsError {
    #[error("email provider not configured")]
    EmailNotConfigured,

    #[error("sms provider not configured")]
    SmsNotConfigured,

    #[error("template '{0}' not found")]
    TemplateNotFound(String),

    #[error("template channel mismatch: template channel is {template_channel}, recipient is {recipient_channel}")]
    ChannelMismatch {
        template_channel: String,
        recipient_channel: String,
    },

    #[error("template render error: {0}")]
    TemplateRender(String),

    #[error("provider HTTP error: status {status}")]
    ProviderHttp { status: u16, body: String },

    #[error("provider error: {0}")]
    Provider(String),

    #[error("transient provider error: {0}")]
    Transient(String),

    #[error("invalid configuration: {0}")]
    Config(String),
}

impl CommsError {
    /// Returns true if the error is worth retrying under a `RetryPolicy`.
    /// HTTP 5xx and explicit `Transient` are retryable; everything else is permanent.
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            CommsError::Transient(_) | CommsError::ProviderHttp { status: 500..=599, .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn display_for_each_variant_is_clean() {
        assert_eq!(
            CommsError::EmailNotConfigured.to_string(),
            "email provider not configured"
        );
        assert_eq!(
            CommsError::SmsNotConfigured.to_string(),
            "sms provider not configured"
        );
        assert_eq!(
            CommsError::TemplateNotFound("password_reset".into()).to_string(),
            "template 'password_reset' not found"
        );
        assert_eq!(
            CommsError::ChannelMismatch {
                template_channel: "email".into(),
                recipient_channel: "sms".into(),
            }
            .to_string(),
            "template channel mismatch: template channel is email, recipient is sms"
        );
        assert_eq!(
            CommsError::TemplateRender("missing token: foo".into()).to_string(),
            "template render error: missing token: foo"
        );
        assert_eq!(
            CommsError::ProviderHttp {
                status: 503,
                body: "Service Unavailable".into(),
            }
            .to_string(),
            "provider HTTP error: status 503"
        );
        assert_eq!(
            CommsError::Provider("mailgun rejected sender".into()).to_string(),
            "provider error: mailgun rejected sender"
        );
        assert_eq!(
            CommsError::Transient("connection reset".into()).to_string(),
            "transient provider error: connection reset"
        );
        assert_eq!(
            CommsError::Config("missing api_key".into()).to_string(),
            "invalid configuration: missing api_key"
        );
    }

    #[test]
    fn is_transient_classifies_correctly() {
        assert!(CommsError::Transient("blip".into()).is_transient());
        assert!(CommsError::ProviderHttp { status: 500, body: "".into() }.is_transient());
        assert!(CommsError::ProviderHttp { status: 503, body: "".into() }.is_transient());
        assert!(CommsError::ProviderHttp { status: 599, body: "".into() }.is_transient());

        assert!(!CommsError::EmailNotConfigured.is_transient());
        assert!(!CommsError::SmsNotConfigured.is_transient());
        assert!(!CommsError::TemplateNotFound("x".into()).is_transient());
        assert!(!CommsError::ChannelMismatch {
            template_channel: "email".into(),
            recipient_channel: "sms".into()
        }
        .is_transient());
        assert!(!CommsError::TemplateRender("x".into()).is_transient());
        assert!(!CommsError::ProviderHttp { status: 400, body: "".into() }.is_transient());
        assert!(!CommsError::ProviderHttp { status: 401, body: "".into() }.is_transient());
        assert!(!CommsError::ProviderHttp { status: 404, body: "".into() }.is_transient());
        assert!(!CommsError::Provider("permanent".into()).is_transient());
        assert!(!CommsError::Config("x".into()).is_transient());
    }
}
