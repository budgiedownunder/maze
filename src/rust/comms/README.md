# `comms` Crate

## Introduction

The `comms` crate is written in `Rust` and provides outbound email
communications on behalf of the operating company through pluggable
providers. It accepts already-built render contexts and has no dependency
on `data_model` or `storage`, so it can be reused by any consumer in the
workspace.

The crate is structured around a per-medium provider trait pattern: a thin
`Provider` super-trait carrying cross-cutting concerns (name, retry policy)
plus a per-medium trait — currently `EmailProvider` — over the top. A new
medium would land as one trait + one slot on `Comms`, not a redesign.

The crate currently defines the following modules:

- `provider` - cross-cutting `Provider` super-trait and `DeliveryReceipt` type
- `email` - `EmailProvider` trait, `EmailAddress`, and `EmailMessage`
- `error` - `CommsError` taxonomy with `is_transient()` classification
- `retry` - `RetryPolicy` with bounded exponential backoff
- `oauth` - `OAuthTokenSource` trait, `Clock`/`SystemClock` abstraction, and
  `RefreshTokenStore` trait surface. Per-flow token sources (e.g.
  `ClientCredentialsTokenSource`) live in submodules behind feature flags.
- `orchestrator` - `Comms` dispatcher. Holds the email provider slot, the
  shared `TemplateRenderer`, and the default sender identity; `send_template`
  renders and dispatches, and `send_email` applies the provider's
  `RetryPolicy` (bounded retry on transient errors, immediate return on
  permanent ones).
- `providers` - per-provider `EmailProvider` implementations. Each provider
  lives in a sibling module gated by its own `provider-*` feature flag.

## Cargo features

- `stub` - exposes `StubEmailProvider`, an in-memory provider impl that
  captures dispatched messages instead of sending them. Intended for use
  in downstream test crates' `dev-dependencies`:
  `comms = { path = "../comms", features = ["stub"] }`.
- `oauth2-microsoft` - exposes `ClientCredentialsTokenSource` for the
  Microsoft Azure AD `client_credentials` OAuth2 flow. Pulls in `reqwest`
  with `rustls-tls`. Used by Microsoft 365 / Microsoft Graph providers.
- `oauth2-google` - exposes `ServiceAccountTokenSource` for the Google
  Workspace JWT-bearer OAuth2 flow (with optional domain-wide delegation
  via the `subject` claim). Pulls in `reqwest` with `rustls-tls` and
  `jsonwebtoken`. Used by Gmail API providers.
- `oauth2-refresh-token` - exposes `RefreshTokenTokenSource` for the
  generic OAuth2 refresh-token flow against any token endpoint. Pulls in
  `reqwest` with `rustls-tls`. Used for per-user accounts (e.g. personal
  Gmail) where the OAuth consent dance happens once out-of-band and the
  resulting `refresh_token` is supplied to the server as a long-lived
  secret.
- `provider-mailgun` - exposes `MailgunProvider`, an `EmailProvider`
  backed by the Mailgun HTTP API (US or EU regional host, HTTP Basic
  auth with `api:<api_key>`). Pulls in `reqwest` with `rustls-tls`.
- `provider-smtp-oauth2` - exposes `SmtpOAuth2Provider`, an `EmailProvider`
  that ships messages over SMTP and authenticates with XOAUTH2 using any
  `OAuthTokenSource`. Pairs naturally with `oauth2-microsoft` (Microsoft
  365 client-credentials) or `oauth2-google` (Google Workspace
  service-account); the consumer enables those features separately.
  Pulls in `lettre` with `tokio1-rustls-tls`.

## Getting Started

### Build
To build the `comms` crate, run the following from within the `comms` directory:
```
cargo build --all-features
```

### Testing
To test the `comms` crate, run the following from within the `comms` directory:
```
cargo test --all-features
```

### Linting
To lint the `comms` crate, run the following from within the `comms` directory:
```
cargo clippy --all-targets --all-features
```

Expected: zero errors, zero warnings.

### Benchmarking
No benchmarking tests are currently implemented for the crate.

### Generating Documentation
To generate and view `Rust` documentation for the crate in your default browser, run the following from within the `comms` directory:
```
cargo doc --all-features --open
```
