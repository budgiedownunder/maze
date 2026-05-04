# `comms` Crate

## Introduction

The `comms` crate is written in `Rust` and provides outbound communications
on behalf of the operating company through pluggable providers. It accepts
already-built recipients and render contexts and has no dependency on
`data_model` or `storage`, so it can be reused by any consumer in the workspace.

The crate currently defines the following modules:

- `provider` - cross-cutting `Provider` super-trait and `DeliveryReceipt` type
- `email` - `EmailProvider` trait, `EmailAddress`, and `EmailMessage`
- `sms` - `SmsProvider` trait, `PhoneNumber`, and `SmsMessage`
- `recipient` - `Recipient` enum used for per-medium routing
- `error` - `CommsError` taxonomy with `is_transient()` classification
- `retry` - `RetryPolicy` with bounded exponential backoff
- `oauth` - `OAuthTokenSource` trait, `Clock`/`SystemClock` abstraction, and
  `RefreshTokenStore` trait surface. Per-flow token sources (e.g.
  `ClientCredentialsTokenSource`) live in submodules behind feature flags.

## Cargo features

- `stub` - exposes `StubEmailProvider` and `StubSmsProvider`, in-memory
  provider impls that capture dispatched messages instead of sending them.
  Intended for use in downstream test crates' `dev-dependencies`:
  `comms = { path = "../comms", features = ["stub"] }`.
- `oauth2-microsoft` - exposes `ClientCredentialsTokenSource` for the
  Microsoft Azure AD `client_credentials` OAuth2 flow. Pulls in `reqwest`
  with `rustls-tls`. Used by Microsoft 365 / Microsoft Graph providers.
- `oauth2-google` - exposes `ServiceAccountTokenSource` for the Google
  Workspace JWT-bearer OAuth2 flow (with optional domain-wide delegation
  via the `subject` claim). Pulls in `reqwest` with `rustls-tls` and
  `jsonwebtoken`. Used by Gmail API providers.

## Getting Started

### Build
To build the `comms` crate, run the following from within the `comms` directory:
```
cargo build
```

### Testing
To test the `comms` crate, run the following from within the `comms` directory:
```
cargo test
```

### Benchmarking
No benchmarking tests are currently implemented for the crate

### Generating Documentation
To generate and view `Rust` documentation for the crate in your default browser, run the following from within the `comms` directory:
```
cargo doc --open
```
