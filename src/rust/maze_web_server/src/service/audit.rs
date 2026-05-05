//! Audit-log glue between the dispatch path and `EmailAuditLog`.
//!
//! Every outbound email send goes through [`record_and_dispatch`]:
//!   1. Record an [`AuditOutcome::Pending`] row synchronously, before any
//!      provider work happens. Returns the row id so the caller has a
//!      handle if it ever needs to query the row mid-flight.
//!   2. `tokio::spawn` the actual `comms.send_template` call. When the
//!      provider responds, update the row to either
//!      [`AuditOutcome::Accepted`] (with the provider's message id) or
//!      [`AuditOutcome::Failed`] (with a coarse error class).
//!
//! Reconnaissance flows (e.g. password-reset request for an email that
//! doesn't match any user) record a pending-only row via
//! [`record_pending_only`]: same row shape with
//! `recipient_user_id = None`, no follow-up `update_outcome`. Lets a
//! later audit query distinguish "we sent but failed" from "we
//! deliberately did nothing because there was no recipient" while still
//! capturing the request for rate-limit / abuse forensics.

use std::sync::Arc;

use comms::{Comms, CommsError, EmailAddress};
use data_model::{AuditOutcome, EmailAuditEntry};
use log::{info, warn};
use serde::Serialize;
use storage::SharedStore;
use uuid::Uuid;

/// Coarse error taxonomy persisted on `EmailAuditLog::update_outcome`
/// when a send fails. Stable, low-cardinality strings — designed for
/// dashboards and rate-limit signals, not for surfacing to end users.
///
/// # Examples
///
/// ```
/// use comms::CommsError;
/// use maze_web_server::service::audit::error_class_for;
///
/// assert_eq!(error_class_for(&CommsError::EmailNotConfigured), "email_not_configured");
/// assert_eq!(error_class_for(&CommsError::Transient("blip".into())), "transient");
/// assert_eq!(
///     error_class_for(&CommsError::ProviderHttp { status: 503, body: "".into() }),
///     "provider_5xx"
/// );
/// assert_eq!(
///     error_class_for(&CommsError::ProviderHttp { status: 401, body: "".into() }),
///     "provider_4xx"
/// );
/// ```
pub fn error_class_for(err: &CommsError) -> &'static str {
    match err {
        CommsError::EmailNotConfigured => "email_not_configured",
        CommsError::TemplateNotFound(_) => "template_not_found",
        CommsError::TemplateRender(_) => "template_render",
        CommsError::Config(_) => "config",
        CommsError::ProviderHttp { status, .. } if (500..=599).contains(status) => "provider_5xx",
        CommsError::ProviderHttp { .. } => "provider_4xx",
        CommsError::Provider(_) => "provider",
        CommsError::Transient(_) => "transient",
    }
}

/// Provider-name fallback when `Comms` has no email slot configured
/// (i.e. notifications disabled). The audit row records this string in
/// `provider` so the audit history is still consistent.
const PROVIDER_NAME_UNCONFIGURED: &str = "none";

/// Record a `Pending` audit row, then `tokio::spawn` the send. The
/// spawned task updates the row to `Accepted` (with
/// `provider_message_id`) or `Failed` (with `error_class`) once the
/// provider responds.
///
/// Returns the audit row id. The HTTP response is already committed by
/// the time the spawn resolves.
///
/// `recipient_user_id`/`triggered_by_user_id` are recorded verbatim. For
/// the anti-enumeration reset path, where no user matches the supplied
/// email, use [`record_pending_only`] instead — it records the request
/// with `recipient_user_id = None` and never schedules a send.
///
/// # Examples
///
/// ```
/// # tokio_test::block_on(async {
/// # use comms::{AppContext, BrandingContext, BrandingPartialSources, Comms,
/// #             EmailAddress, EmbeddedTemplateLoader, StubEmailProvider,
/// #             TemplateLoader, TemplateRenderer};
/// # use std::sync::Arc;
/// # use tokio::sync::RwLock as AsyncRwLock;
/// # use storage::{FileStore, FileStoreConfig, SharedStore, Store};
/// # use maze_web_server::service::audit::record_and_dispatch;
/// # use serde_json::json;
/// # let temp = tempfile::tempdir().expect("tempdir");
/// # let store: SharedStore = Arc::new(AsyncRwLock::new(
/// #     Box::new(FileStore::new(&FileStoreConfig {
/// #         data_dir: temp.path().to_string_lossy().to_string(),
/// #     })) as Box<dyn Store>,
/// # ));
/// # let renderer = TemplateRenderer::new(
/// #     AppContext { app_name: "App".into(), server_url: "https://x".into(),
/// #         branding: BrandingContext { company_name: "X".into(), company_address: "A".into(),
/// #             company_url: "https://x".into(), logo_url: "https://x".into() } },
/// #     Arc::new(EmbeddedTemplateLoader::from_pairs(&[("greet", "subject = \"Hi\"\ntext = \"Hi\"")]))
/// #         as Arc<dyn TemplateLoader>,
/// #     BrandingPartialSources { logo_html: String::new(), logo_text: String::new(),
/// #         header_html: String::new(), header_text: String::new(),
/// #         footer_html: String::new(), footer_text: String::new() },
/// # ).expect("renderer");
/// # let stub = StubEmailProvider::new();
/// # let comms = Arc::new(Comms::new(renderer, Some(Arc::new(stub.clone())),
/// #     Some(EmailAddress::new("noreply@example.com"))));
/// // Record + dispatch a password-reset send.
/// let audit_id = record_and_dispatch(
///     store.clone(),
///     comms,
///     "greet",
///     /* recipient_user_id   */ None,
///     /* triggered_by_user_id*/ None,
///     /* token_id            */ None,
///     EmailAddress::new("alice@example.com"),
///     json!({}),
/// ).await.expect("record_and_dispatch");
/// assert_ne!(audit_id, uuid::Uuid::nil());
/// # });
/// ```
#[allow(clippy::too_many_arguments)]
pub async fn record_and_dispatch<C: Serialize + Send + Sync + 'static>(
    store: SharedStore,
    comms: Arc<Comms>,
    template_id: &str,
    recipient_user_id: Option<Uuid>,
    triggered_by_user_id: Option<Uuid>,
    token_id: Option<Uuid>,
    to: EmailAddress,
    context: C,
) -> Result<Uuid, storage::Error> {
    let provider = comms
        .email_provider_name()
        .unwrap_or(PROVIDER_NAME_UNCONFIGURED);
    let entry = EmailAuditEntry::new_pending(
        recipient_user_id,
        &to.address,
        template_id,
        token_id,
        triggered_by_user_id,
        provider,
    );
    let audit_id = entry.id;
    {
        let mut store_lock = store.write().await;
        store_lock.record_pending(&entry).await?;
    }

    let template = template_id.to_string();
    let store_for_task = store.clone();
    tokio::spawn(async move {
        let outcome = comms.send_template(&template, to, &context).await;
        let mut store_lock = store_for_task.write().await;
        match outcome {
            Ok(receipt) => {
                let provider_message_id = receipt.provider_message_id.as_deref();
                if let Err(err) = store_lock
                    .update_outcome(audit_id, AuditOutcome::Accepted, provider_message_id, None)
                    .await
                {
                    warn!("audit: update_outcome(Accepted) failed for {audit_id}: {err}");
                } else {
                    info!("audit: send accepted for {audit_id}");
                }
            }
            Err(err) => {
                let class = error_class_for(&err);
                if let Err(update_err) = store_lock
                    .update_outcome(audit_id, AuditOutcome::Failed, None, Some(class))
                    .await
                {
                    warn!(
                        "audit: update_outcome(Failed) failed for {audit_id}: {update_err} (original: {err})"
                    );
                } else {
                    warn!("audit: send failed for {audit_id}: {err} (class={class})");
                }
            }
        }
    });

    Ok(audit_id)
}

/// Record a Pending audit row without scheduling a send. Used for the
/// anti-enumeration reset path: the request was made for an email that
/// doesn't match any user, so no send fires, but we still record the
/// request itself for rate-limit / reconnaissance forensics with
/// `recipient_user_id = None`.
///
/// Returns the audit row id. The row is left at `Pending` permanently —
/// no follow-up `update_outcome` call.
///
/// # Examples
///
/// ```
/// # tokio_test::block_on(async {
/// # use std::sync::Arc;
/// # use tokio::sync::RwLock as AsyncRwLock;
/// # use storage::{EmailAuditLog, FileStore, FileStoreConfig, SharedStore, Store};
/// # use maze_web_server::service::audit::record_pending_only;
/// # let temp = tempfile::tempdir().expect("tempdir");
/// # let store: SharedStore = Arc::new(AsyncRwLock::new(
/// #     Box::new(FileStore::new(&FileStoreConfig {
/// #         data_dir: temp.path().to_string_lossy().to_string(),
/// #     })) as Box<dyn Store>,
/// # ));
/// let id = record_pending_only(
///     store.clone(),
///     "password_reset",
///     "ghost@example.com",
///     "stub",
/// ).await.expect("record");
/// let row = store.read().await.find_audit_entry(id).await.expect("find");
/// assert!(row.recipient_user_id.is_none());
/// # });
/// ```
pub async fn record_pending_only(
    store: SharedStore,
    template_id: &str,
    recipient_email: &str,
    provider: &str,
) -> Result<Uuid, storage::Error> {
    let entry = EmailAuditEntry::new_pending(
        None,
        recipient_email,
        template_id,
        None,
        None,
        provider,
    );
    let id = entry.id;
    let mut store_lock = store.write().await;
    store_lock.record_pending(&entry).await?;
    Ok(id)
}
