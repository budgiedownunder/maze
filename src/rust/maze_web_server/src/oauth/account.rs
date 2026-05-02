//! Connector-agnostic resolution of a [`NormalisedIdentity`] to a [`User`].
//!
//! Three branches, in order:
//!
//! 1. **Returning OAuth user** — `(provider, provider_user_id)` already known
//!    on some user. Refresh `provider_email` and `last_seen_at`, persist, log
//!    in. Done.
//! 2. **First-time OAuth, email matches an existing password account**
//!    (auto-link). Append a new [`OAuthIdentity`] to the existing user, log
//!    in. Requires `email_verified = true`. **Not** gated by `allow_signup` —
//!    attaching a sign-in method to an existing account is not signup.
//! 3. **First-time OAuth, no matching account** — create a new user with
//!    `password_hash = ""` and the OAuth identity attached. **Only this
//!    branch is gated by `allow_signup`.**
//!
//! See [`crate::oauth`] module docs for the design rationale.

use crate::oauth::NormalisedIdentity;
use chrono::Utc;
use data_model::{OAuthIdentity, User};
use storage::{Error as StoreError, UserStore};

/// Outcome of [`resolve`]: either a returning user signed in (possibly with a
/// freshly-attached OAuth identity), or a brand-new user was created.
#[derive(Debug, PartialEq)]
pub enum ResolveOutcome {
    SignedIn(User),
    Created(User),
}

/// Why [`resolve`] could not produce a `User`.
#[derive(Debug)]
pub enum ResolveError {
    /// Branch 3 was the only option but signup is disabled server-wide.
    SignupDisabled,
    /// Branches 2 and 3 both need an email to proceed (link target / new
    /// user record), but the provider returned none.
    MissingEmail,
    /// Provider returned an email but did not vouch for it. Branch 2
    /// auto-link is unsafe; branch 3 we also refuse to avoid creating
    /// accounts based on unverified addresses.
    EmailNotVerified,
    /// Wrapped store error (IO, serde, validation, etc.).
    Store(StoreError),
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolveError::SignupDisabled => write!(f, "OAuth sign-up is disabled on this server"),
            ResolveError::MissingEmail => write!(f, "OAuth provider did not return an email address"),
            ResolveError::EmailNotVerified => {
                write!(f, "OAuth provider did not vouch for the email's verification")
            }
            ResolveError::Store(e) => write!(f, "store error: {e}"),
        }
    }
}

impl std::error::Error for ResolveError {}

impl From<StoreError> for ResolveError {
    fn from(e: StoreError) -> Self { ResolveError::Store(e) }
}

/// Resolve a [`NormalisedIdentity`] to a stored [`User`], creating one if
/// allowed. See module docs for the branch semantics.
pub async fn resolve(
    store: &mut dyn UserStore,
    identity: &NormalisedIdentity,
    allow_signup: bool,
) -> Result<ResolveOutcome, ResolveError> {
    // ---- Branch 1: existing OAuth identity ---------------------------------
    if let Ok(mut user) = store.find_user_by_oauth_identity(&identity.provider, &identity.provider_user_id).await {
        refresh_identity(&mut user, identity);
        store.update_user(&mut user).await?;
        return Ok(ResolveOutcome::SignedIn(user));
    }

    // For branches 2 and 3 we need an email.
    let email = match &identity.email {
        Some(e) if !e.trim().is_empty() => e.clone(),
        _ => return Err(ResolveError::MissingEmail),
    };

    // ---- Branch 2: email-link to an existing password account --------------
    if let Ok(mut user) = store.find_user_by_verified_email(&email).await {
        if !identity.email_verified {
            // Refuse: linking to an existing account based on an unverified
            // email would let an attacker hijack accounts at providers that
            // tolerate unverified addresses.
            return Err(ResolveError::EmailNotVerified);
        }
        // Defence-in-depth: refresh `verified_at` on the matched row to
        // capture the provider's fresh confirmation. The lookup already
        // gates on `verified = true`, so this is a no-op for the flag, but
        // the timestamp update is meaningful and the assignment survives
        // any future loosening of the lookup.
        if let Some(row) = user
            .emails
            .iter_mut()
            .find(|r| r.email.eq_ignore_ascii_case(&email))
        {
            row.mark_verified();
        }
        user.oauth_identities.push(OAuthIdentity::new(
            identity.provider.clone(),
            identity.provider_user_id.clone(),
            Some(email),
        ));
        store.update_user(&mut user).await?;
        return Ok(ResolveOutcome::SignedIn(user));
    }

    // ---- Branch 3: create a new user (signup) ------------------------------
    if !allow_signup {
        return Err(ResolveError::SignupDisabled);
    }
    if !identity.email_verified {
        return Err(ResolveError::EmailNotVerified);
    }

    let username = unique_username_from_email(store, &email).await;
    let mut new_user = User {
        id: User::new_id(),
        is_admin: false,
        username,
        full_name: identity.display_name.clone().unwrap_or_default(),
        emails: vec![data_model::UserEmail::new_primary_verified(&email)],
        password_hash: String::new(), // OAuth-only account; verify_password hardens against this
        api_key: User::new_api_key(),
        logins: vec![],
        oauth_identities: vec![OAuthIdentity::new(
            identity.provider.clone(),
            identity.provider_user_id.clone(),
            Some(email),
        )],
    };
    store.create_user(&mut new_user).await?;
    Ok(ResolveOutcome::Created(new_user))
}

/// Refresh `provider_email` and `last_seen_at` on the matched OAuth identity
/// row to reflect the freshest provider observation.
fn refresh_identity(user: &mut User, identity: &NormalisedIdentity) {
    if let Some(row) = user.oauth_identities.iter_mut().find(|r| {
        r.provider.eq_ignore_ascii_case(&identity.provider)
            && r.provider_user_id == identity.provider_user_id
    }) {
        row.provider_email = identity.email.clone();
        row.last_seen_at = Utc::now();
    }
}

/// Derive a candidate username from the email's local part, then suffix it
/// with `_2`, `_3`, … until it is not already taken.
async fn unique_username_from_email(store: &dyn UserStore, email: &str) -> String {
    let base = sanitize_username(email.split('@').next().unwrap_or("user"));
    let mut candidate = base.clone();
    let mut counter: u32 = 2;
    while store.find_user_by_name(&candidate).await.is_ok() {
        candidate = format!("{base}_{counter}");
        counter = counter.saturating_add(1);
    }
    candidate
}

fn sanitize_username(local: &str) -> String {
    let cleaned: String = local
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = cleaned.trim_matches('_').to_string();
    if trimmed.is_empty() { "user".to_string() } else { trimmed }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use uuid::Uuid;

    /// Minimal in-memory `UserStore` for testing. Only implements the methods
    /// `account::resolve` actually calls (find_by_email, find_by_name,
    /// find_by_oauth_identity, create_user, update_user) plus a couple of
    /// helpers; everything else returns `Other("not used in tests")`.
    #[derive(Default)]
    struct MemStore {
        users: HashMap<Uuid, User>,
    }

    impl MemStore {
        fn insert(&mut self, mut user: User) -> User {
            if user.id == Uuid::nil() { user.id = User::new_id(); }
            self.users.insert(user.id, user.clone());
            user
        }
    }

    #[async_trait]
    impl UserStore for MemStore {
        async fn init_default_admin_user(&mut self, _u: &str, _e: &str, _p: &str) -> Result<User, StoreError> {
            Err(StoreError::Other("not used".into()))
        }
        async fn create_user(&mut self, user: &mut User) -> Result<(), StoreError> {
            if user.id == Uuid::nil() { user.id = User::new_id(); }
            self.users.insert(user.id, user.clone());
            Ok(())
        }
        async fn delete_user(&mut self, _id: Uuid) -> Result<(), StoreError> {
            Err(StoreError::Other("not used".into()))
        }
        async fn update_user(&mut self, user: &mut User) -> Result<(), StoreError> {
            self.users.insert(user.id, user.clone());
            Ok(())
        }
        async fn get_user(&self, id: Uuid) -> Result<User, StoreError> {
            self.users.get(&id).cloned().ok_or(StoreError::UserNotFound())
        }
        async fn find_user_by_name(&self, name: &str) -> Result<User, StoreError> {
            self.users
                .values()
                .find(|u| u.username.eq_ignore_ascii_case(name))
                .cloned()
                .ok_or(StoreError::UserNotFound())
        }
        async fn find_user_by_verified_email(&self, email: &str) -> Result<User, StoreError> {
            self.users
                .values()
                .find(|u| {
                    u.emails
                        .iter()
                        .any(|row| row.verified && row.email.eq_ignore_ascii_case(email))
                })
                .cloned()
                .ok_or(StoreError::UserNotFound())
        }
        async fn find_user_by_api_key(&self, _key: Uuid) -> Result<User, StoreError> {
            Err(StoreError::Other("not used".into()))
        }
        async fn find_user_by_login_id(&self, _id: Uuid) -> Result<User, StoreError> {
            Err(StoreError::Other("not used".into()))
        }
        async fn find_user_by_oauth_identity(&self, provider: &str, provider_user_id: &str) -> Result<User, StoreError> {
            self.users
                .values()
                .find(|u| {
                    u.oauth_identities.iter().any(|i| {
                        i.provider.eq_ignore_ascii_case(provider) && i.provider_user_id == provider_user_id
                    })
                })
                .cloned()
                .ok_or(StoreError::UserNotFound())
        }
        async fn get_users(&self) -> Result<Vec<User>, StoreError> { Ok(self.users.values().cloned().collect()) }
        async fn get_admin_users(&self) -> Result<Vec<User>, StoreError> { Ok(vec![]) }
        async fn has_users(&self) -> Result<bool, StoreError> { Ok(!self.users.is_empty()) }
        async fn add_user_email(&mut self, _u: Uuid, _e: &str, _v: bool) -> Result<data_model::UserEmail, StoreError> {
            Err(StoreError::Other("not used".into()))
        }
        async fn remove_user_email(&mut self, _u: Uuid, _e: &str) -> Result<(), StoreError> {
            Err(StoreError::Other("not used".into()))
        }
        async fn set_primary_email(&mut self, _u: Uuid, _e: &str) -> Result<(), StoreError> {
            Err(StoreError::Other("not used".into()))
        }
        async fn mark_email_verified(&mut self, _u: Uuid, _e: &str) -> Result<(), StoreError> {
            Err(StoreError::Other("not used".into()))
        }
    }

    fn ident(provider: &str, sub: &str, email: Option<&str>, verified: bool) -> NormalisedIdentity {
        NormalisedIdentity {
            provider: provider.to_string(),
            provider_user_id: sub.to_string(),
            email: email.map(|s| s.to_string()),
            email_verified: verified,
            display_name: None,
        }
    }

    fn password_user(email: &str, username: &str) -> User {
        User {
            id: User::new_id(),
            is_admin: false,
            username: username.to_string(),
            full_name: String::new(),
            emails: vec![data_model::UserEmail::new_primary_verified(email)],
            password_hash: "$argon2id$dummy".to_string(),
            api_key: User::new_api_key(),
            logins: vec![],
            oauth_identities: vec![],
        }
    }

    #[tokio::test]
    async fn branch_1_existing_oauth_identity_signs_in_and_refreshes_email() {
        let mut store = MemStore::default();
        let mut user = password_user("alice@example.com", "alice");
        user.oauth_identities.push(OAuthIdentity::new(
            "google".into(),
            "sub-alice".into(),
            Some("old@example.com".into()),
        ));
        let inserted = store.insert(user);

        let identity = ident("google", "sub-alice", Some("alice-new@example.com"), true);
        let outcome = resolve(&mut store, &identity, true).await.expect("ok");
        match outcome {
            ResolveOutcome::SignedIn(u) => {
                assert_eq!(u.id, inserted.id);
                assert_eq!(u.oauth_identities.len(), 1);
                assert_eq!(u.oauth_identities[0].provider_email.as_deref(), Some("alice-new@example.com"));
            }
            other => panic!("expected SignedIn, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn branch_2_auto_link_appends_oauth_identity_when_email_verified() {
        let mut store = MemStore::default();
        let inserted = store.insert(password_user("alice@example.com", "alice"));

        let identity = ident("google", "sub-alice", Some("alice@example.com"), true);
        let outcome = resolve(&mut store, &identity, false /* allow_signup */).await.expect("ok");
        let user = match outcome {
            ResolveOutcome::SignedIn(u) => u,
            other => panic!("expected SignedIn, got {other:?}"),
        };
        assert_eq!(user.id, inserted.id, "must be the existing user, not a new one");
        assert_eq!(user.oauth_identities.len(), 1);
        assert_eq!(user.oauth_identities[0].provider, "google");
        assert_eq!(user.oauth_identities[0].provider_user_id, "sub-alice");
        // Critical: branch 2 must work even when allow_signup is false. Linking
        // a sign-in method to an existing user is not the same as signup.
    }

    #[tokio::test]
    async fn branch_2_refuses_to_link_when_email_not_verified() {
        let mut store = MemStore::default();
        store.insert(password_user("alice@example.com", "alice"));

        let identity = ident("google", "sub-alice", Some("alice@example.com"), false);
        let err = resolve(&mut store, &identity, true).await.unwrap_err();
        assert!(matches!(err, ResolveError::EmailNotVerified));
    }

    #[tokio::test]
    async fn branch_3_creates_new_user_when_no_match_and_signup_allowed() {
        let mut store = MemStore::default();

        let identity = NormalisedIdentity {
            provider: "github".to_string(),
            provider_user_id: "12345".to_string(),
            email: Some("bob@example.com".to_string()),
            email_verified: true,
            display_name: Some("Bob".to_string()),
        };
        let outcome = resolve(&mut store, &identity, true).await.expect("ok");
        let user = match outcome {
            ResolveOutcome::Created(u) => u,
            other => panic!("expected Created, got {other:?}"),
        };
        assert_eq!(user.email(), "bob@example.com");
        assert_eq!(user.username, "bob");
        assert_eq!(user.full_name, "Bob");
        assert!(user.password_hash.is_empty(), "OAuth-only account should have empty password hash");
        assert_eq!(user.oauth_identities.len(), 1);
    }

    #[tokio::test]
    async fn branch_3_refuses_when_signup_disabled() {
        let mut store = MemStore::default();
        let identity = ident("github", "12345", Some("bob@example.com"), true);
        let err = resolve(&mut store, &identity, false).await.unwrap_err();
        assert!(matches!(err, ResolveError::SignupDisabled));
    }

    #[tokio::test]
    async fn branch_3_refuses_when_email_not_verified() {
        let mut store = MemStore::default();
        let identity = ident("github", "12345", Some("bob@example.com"), false);
        let err = resolve(&mut store, &identity, true).await.unwrap_err();
        assert!(matches!(err, ResolveError::EmailNotVerified));
    }

    #[tokio::test]
    async fn missing_email_is_error_when_neither_branch_1_applies() {
        let mut store = MemStore::default();
        let identity = ident("github", "12345", None, true);
        let err = resolve(&mut store, &identity, true).await.unwrap_err();
        assert!(matches!(err, ResolveError::MissingEmail));
    }

    #[tokio::test]
    async fn re_sign_in_with_changed_provider_email_updates_stored_value() {
        // Locks in the "provider_email is a fresh observation" semantic: on
        // every successful sign-in via branch 1, the stored row's email is
        // replaced with whatever the provider just told us.
        let mut store = MemStore::default();
        let mut user = password_user("alice@example.com", "alice");
        user.oauth_identities.push(OAuthIdentity::new(
            "google".into(),
            "sub-alice".into(),
            Some("first@example.com".into()),
        ));
        let original_seen = user.oauth_identities[0].last_seen_at;
        let inserted_id = store.insert(user).id;
        std::thread::sleep(std::time::Duration::from_millis(5));

        let identity = ident("google", "sub-alice", Some("second@example.com"), true);
        resolve(&mut store, &identity, false).await.expect("ok");
        let stored = store.users.get(&inserted_id).unwrap();
        assert_eq!(stored.oauth_identities[0].provider_email.as_deref(), Some("second@example.com"));
        assert!(stored.oauth_identities[0].last_seen_at > original_seen);
    }

    /// Builds a password account whose primary email is **unverified** —
    /// the squatter shape used by the attacker-squat regression test below.
    fn password_user_unverified(email: &str, username: &str) -> User {
        User {
            id: User::new_id(),
            is_admin: false,
            username: username.to_string(),
            full_name: String::new(),
            emails: vec![data_model::UserEmail {
                email: email.to_string(),
                is_primary: true,
                verified: false,
                verified_at: None,
            }],
            password_hash: "$argon2id$dummy".to_string(),
            api_key: User::new_api_key(),
            logins: vec![],
            oauth_identities: vec![],
        }
    }

    #[tokio::test]
    async fn branch_2_does_not_auto_link_when_local_email_is_unverified() {
        // Attacker-squat regression: an attacker registers an account with
        // `victim@example.com` but never proves ownership (`verified =
        // false`). Later the real victim signs in via OAuth and the
        // provider returns `email_verified = true` for the same address.
        // `resolve()` must NOT silently link the OAuth identity to the
        // squatter — that would hand the victim's account to the attacker.
        // The verified-only filter on `find_user_by_verified_email` is
        // what enforces this, so Branch 2 doesn't fire and the flow falls
        // through to Branch 3.
        let mut store = MemStore::default();
        let squatter_id = store
            .insert(password_user_unverified("victim@example.com", "squatter"))
            .id;

        let identity = ident("google", "sub-victim", Some("victim@example.com"), true);
        let outcome = resolve(&mut store, &identity, true /* allow_signup */)
            .await
            .expect("ok");
        match outcome {
            ResolveOutcome::Created(u) => {
                assert_ne!(
                    u.id, squatter_id,
                    "must NOT link to the unverified squatter — that would be the security hole"
                );
                // Sanity: the OAuth identity is on the new user, not the squatter.
                assert_eq!(u.oauth_identities.len(), 1);
                assert_eq!(u.oauth_identities[0].provider_user_id, "sub-victim");
            }
            other => panic!("expected Created (Branch 3), got {other:?}"),
        }

        // The squatter is still in the store, untouched.
        let squatter = store.users.get(&squatter_id).expect("squatter still present");
        assert!(
            squatter.oauth_identities.is_empty(),
            "the attacker's record must not have gained an OAuth identity"
        );
    }

    #[tokio::test]
    async fn branch_2_refreshes_verified_at_on_link() {
        // Defence-in-depth: linking via Branch 2 must refresh `verified_at`
        // on the matched row to capture the provider's fresh observation,
        // even though the row was already verified (the lookup gates on it).
        let mut store = MemStore::default();
        let mut user = password_user("alice@example.com", "alice");
        // Force a clearly old verified_at so we can assert it advances.
        user.emails[0].verified_at =
            Some(chrono::Utc::now() - chrono::Duration::hours(24));
        let stale = user.emails[0].verified_at.unwrap();
        let inserted_id = store.insert(user).id;

        let identity = ident("google", "sub-alice", Some("alice@example.com"), true);
        resolve(&mut store, &identity, false).await.expect("ok");

        let stored = store.users.get(&inserted_id).expect("user still present");
        let row = stored
            .emails
            .iter()
            .find(|r| r.email == "alice@example.com")
            .expect("primary row still present");
        assert!(row.verified, "row must remain verified");
        assert!(
            row.verified_at.expect("verified_at must be Some") > stale,
            "verified_at must be refreshed to a newer instant on link"
        );
    }

    #[tokio::test]
    async fn branch_3_new_user_has_primary_verified_email_row() {
        // Tightens `branch_3_creates_new_user_when_no_match_and_signup_allowed`
        // around the email-row shape: the new user's first (and only) email
        // row must be `is_primary = true, verified = true` with the
        // provider-supplied address, mirroring what the storage layer
        // expects for OAuth signups.
        let mut store = MemStore::default();
        let identity = ident("github", "12345", Some("bob@example.com"), true);
        let outcome = resolve(&mut store, &identity, true).await.expect("ok");
        let user = match outcome {
            ResolveOutcome::Created(u) => u,
            other => panic!("expected Created, got {other:?}"),
        };
        assert_eq!(user.emails.len(), 1);
        let row = &user.emails[0];
        assert_eq!(row.email, "bob@example.com");
        assert!(row.is_primary, "OAuth-created user's first email must be primary");
        assert!(row.verified, "OAuth-created user's first email must be verified");
        assert!(row.verified_at.is_some(), "verified_at must be populated");
    }

    #[tokio::test]
    async fn branch_2_links_even_if_user_has_an_oauth_identity_with_a_different_provider() {
        // Sanity: an existing user already linked to provider X must still
        // be linkable to a fresh identity at provider Y via Branch 2 (the
        // email match drives the link; the unrelated identity at X doesn't
        // interfere). Confirms that "same email, different provider" is
        // a Branch 2 case, not a Branch 1 case in disguise.
        let mut store = MemStore::default();
        let mut user = password_user("alice@example.com", "alice");
        user.oauth_identities.push(OAuthIdentity::new(
            "google".into(),
            "sub-alice-google".into(),
            Some("alice@example.com".into()),
        ));
        let inserted_id = store.insert(user).id;

        let identity = ident("github", "sub-alice-github", Some("alice@example.com"), true);
        let outcome = resolve(&mut store, &identity, false).await.expect("ok");
        let user = match outcome {
            ResolveOutcome::SignedIn(u) => u,
            other => panic!("expected SignedIn (Branch 2), got {other:?}"),
        };
        assert_eq!(user.id, inserted_id);
        assert_eq!(user.oauth_identities.len(), 2, "both identities must coexist");
        let providers: Vec<&str> = user
            .oauth_identities
            .iter()
            .map(|i| i.provider.as_str())
            .collect();
        assert!(providers.contains(&"google"));
        assert!(providers.contains(&"github"));
    }

    #[tokio::test]
    async fn unique_username_appends_suffix_on_collision() {
        let mut store = MemStore::default();
        store.insert(password_user("any1@example.com", "alice"));

        let identity = ident("google", "sub-alice2", Some("alice@another.com"), true);
        let outcome = resolve(&mut store, &identity, true).await.expect("ok");
        let user = match outcome {
            ResolveOutcome::Created(u) => u,
            other => panic!("expected Created, got {other:?}"),
        };
        assert_eq!(user.username, "alice_2", "should disambiguate against existing 'alice'");
    }

    #[test]
    fn sanitize_username_strips_special_chars_and_lowercases() {
        assert_eq!(sanitize_username("Alice.Smith+demo"), "alice_smith_demo");
        assert_eq!(sanitize_username("___odd___"), "odd");
        assert_eq!(sanitize_username("@@@"), "user");
        assert_eq!(sanitize_username(""), "user");
    }
}
