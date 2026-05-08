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

    #[error("template '{0}' not found")]
    TemplateNotFound(String),

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
    ///
    /// # Examples
    ///
    /// ```
    /// use comms::CommsError;
    ///
    /// assert!(CommsError::Transient("connection reset".into()).is_transient());
    /// assert!(CommsError::ProviderHttp { status: 503, body: "Unavailable".into() }.is_transient());
    /// assert!(!CommsError::ProviderHttp { status: 401, body: "Unauthorized".into() }.is_transient());
    /// assert!(!CommsError::Config("missing api_key".into()).is_transient());
    /// ```
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            CommsError::Transient(_) | CommsError::ProviderHttp { status: 500..=599, .. }
        )
    }

    /// Diagnostic detail string for ops / forensics. Distinct from
    /// [`Display`](std::fmt::Display): for `ProviderHttp` it includes the
    /// upstream response body, which carries the actual rejection reason
    /// (e.g. an Azure AD `AADSTS70011: ... scope ...` body for a token-mint
    /// failure). For every other variant the result equals `to_string()`.
    ///
    /// Intended for the email audit log's `error_message` field and for
    /// verbose error logs — not for surfacing to end users (the body may
    /// include arbitrary upstream content).
    ///
    /// # Examples
    ///
    /// ```
    /// use comms::CommsError;
    ///
    /// // Body-bearing variant: full detail surfaces.
    /// let aad = CommsError::ProviderHttp {
    ///     status: 400,
    ///     body: r#"{"error":"invalid_scope","error_description":"AADSTS70011: ..."}"#.into(),
    /// };
    /// assert!(aad.detail_message().contains("AADSTS70011"));
    /// assert!(aad.detail_message().contains("status 400"));
    ///
    /// // Body-less ProviderHttp matches Display (no trailing colon).
    /// let bare = CommsError::ProviderHttp { status: 503, body: String::new() };
    /// assert_eq!(bare.detail_message(), "provider HTTP error: status 503");
    ///
    /// // Other variants: same as Display.
    /// let permanent = CommsError::Provider("smtp_oauth2: 535 5.7.3 Authentication unsuccessful".into());
    /// assert_eq!(permanent.detail_message(), permanent.to_string());
    /// ```
    pub fn detail_message(&self) -> String {
        match self {
            CommsError::ProviderHttp { status, body } if !body.is_empty() => {
                format!("provider HTTP error: status {status}: {body}")
            }
            other => other.to_string(),
        }
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
            CommsError::TemplateNotFound("password_reset".into()).to_string(),
            "template 'password_reset' not found"
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
        assert!(!CommsError::TemplateNotFound("x".into()).is_transient());
        assert!(!CommsError::TemplateRender("x".into()).is_transient());
        assert!(!CommsError::ProviderHttp { status: 400, body: "".into() }.is_transient());
        assert!(!CommsError::ProviderHttp { status: 401, body: "".into() }.is_transient());
        assert!(!CommsError::ProviderHttp { status: 404, body: "".into() }.is_transient());
        assert!(!CommsError::Provider("permanent".into()).is_transient());
        assert!(!CommsError::Config("x".into()).is_transient());
    }
}
