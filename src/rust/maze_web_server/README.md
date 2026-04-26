# `maze_web_server` Crate

## Introduction

The `maze_web_server` crate is written in `Rust` and is a web server console application that hosts the `REST`-compliant `Maze Web API` and the React [`maze_web_server`](../../react/maze_web_server/README.md) front-end.

It leverages the `Rust` library crates for calculation and generation ([`maze`](../maze/README.md)) and storage ([`storage`](../storage/README.md)). It then exposes them using [`actix`](https://actix.rs/) to serve the API and [`utoipa`](https://docs.rs/utoipa/latest/utoipa/) to publish it as an [`OpenAPI`](https://www.openapis.org/)-compliant interface for use in third party products such as [`Swagger`](https://swagger.io/). 

In addition to the API interfaces, it also supports the following documentation and game endpoints:

| EndPoint                  | Description
|:--------------------------|:------------
| `/api-docs/v1/rapidoc`    | [RapiDoc](https://rapidocweb.com/) 
| `/api-docs/v1/redoc`      | [ReDoc](https://redocly.com/)
| `/api-docs/v1/swagger-ui/`| [Swagger UI](https://swagger.io/tools/swagger-ui/)
| `/game/`                  | First-person 3D maze game — [`Bevy`](https://bevyengine.org/) WASM binary compiled from [`maze_game_bevy_wasm`](../maze_game_bevy_wasm/README.md); loads maze via `/api/v1/mazes/{id}` with bearer token; touch D-pad on mobile

These pages provide interactive documentation and, in the case of the `RapiDoc` and `Swagger UI` interfaces, the ability to manually tests the API as well.

## Getting Started

### Build
To build the `maze_web_server` application, run the following from within the `maze_web_server` directory:
```
cargo build
```

### Testing
To test the `maze_web_server` application, run the following from within the `maze_web_server` directory:
```
cargo test
```

### Running

Run with:
```
cargo run
```

This will utilise the following self-signed certificate files:

|  Name         | Description         | Format
|:--------------|:--------------------|:------
| `cert.pem`    | Certficate file     | `PKCS#8`
| `key.pem`     | Private key file    | `PKCS#8`

These curremtly have an expiry of `07-APR-2027`. Hence, they will need to be renewed after this time has elapsed by using tools such as `openssl` or, for production, a trusted Certificate Authority (e.g. Let's Encrypt). 

Any new files must be generated in `PKCS#8` format. The following command using `openssl` (1.11 and later) will regenerate these files with a `365` day expiry in this format:

```
openssl req -x509 -nodes -newkey rsa:2048 -keyout key.pem -out cert.pem -days 365
```

In addition, the following files are included for development/testing purposes:

|  Name             | Description             | Format
|:------------------|:------------------------|:------
| `empty_cert.pem`  | Empty certficate file   | `Text`
| `empty_key.pem`   | Empty private key file  | `Text`

### Benchmarking
No benchmarking tests are currently implemented for the crate

### Generating Documentation
To generate and view `Rust` documentation for the crate in your default browser, run the following from within the `maze_web_server` directory:
```
cargo doc --open
```

### Configuration

The following configuration settings exist:

| Type     | Name         | Type    | Default Value    | Environment Variable Override
|:---------|:-------------|:--------|:-----------------|:------------
| Global   | `port`             | Integer | `8443`   | `MAZE_WEB_SERVER_PORT`
| Security | `cert_file`        | Text    | `cert.pem` | `MAZE_WEB_SERVER_SECURITY_CERT_FILE`
|          | `key_file`         | Text    | `key.pem`  | `MAZE_WEB_SERVER_SECURITY_KEY_FILE`
| Static   | `static_dir`       | Text    | `static`          | `MAZE_WEB_SERVER_STATIC_DIR`
| Logging  | `log_dir`          | Text    | `logs`            | `MAZE_WEB_SERVER_LOGGING_LOG_DIR`
|          | `log_level`        | Text    | `info`            | `MAZE_WEB_SERVER_LOGGING_LOG_LEVEL`
|          | `log_file_prefix`  | Text    | `maze_web_server_`| `MAZE_WEB_SERVER_LOGGING_LOG_FILE_PREFIX`
| Features | `allow_signup`     | Boolean | `true`            | `MAZE_WEB_SERVER_FEATURES_ALLOW_SIGNUP`
| OAuth    | `oauth.enabled`    | Boolean | `false`           | `MAZE_WEB_SERVER_OAUTH_ENABLED`
|          | `oauth.connector`  | Text (`internal` / `auth0`) | `internal` | `MAZE_WEB_SERVER_OAUTH_CONNECTOR`
|          | `oauth.mobile_redirect_scheme` | Text | `maze-app` | `MAZE_WEB_SERVER_OAUTH_MOBILE_REDIRECT_SCHEME`

These can also be set in a local configuration file called `config.toml` as follows

```toml
port = 8443

[security]
cert_file = "cert.pem"
key_file = "key.pem"

[logging]
log_dir = "logs"
log_level = "info"

[features]
allow_signup = true

[oauth]
enabled = false
connector = "internal"
mobile_redirect_scheme = "maze-app"

# Internal connector: speaks OAuth/OIDC directly to each provider.
# Client secrets are NOT stored here — set the env var named in
# `client_secret_env` (e.g. MAZE_OAUTH_GOOGLE_SECRET).
[oauth.internal.providers.google]
enabled = false
display_name = "Google"
client_id = ""
client_secret_env = "MAZE_OAUTH_GOOGLE_SECRET"
redirect_uri = "https://your-host:8443/api/v1/auth/oauth/google/callback"

[oauth.internal.providers.github]
enabled = false
display_name = "GitHub"
client_id = ""
client_secret_env = "MAZE_OAUTH_GITHUB_SECRET"
redirect_uri = "https://your-host:8443/api/v1/auth/oauth/github/callback"
```

Notes:

- Any environment variable values will take precedence over their corresponding configuration file values.
- `log_dir` is relative to the server working directory. Log files are named `{log_file_prefix}{YYYY-MM-DD}.log` and a new file is started each calendar day. Old log files are not deleted automatically.
- `log_file_prefix` is used verbatim — include any desired separator as the final character (e.g. `"maze_web_server_"` produces `maze_web_server_2026-04-09.log`, while `"my-app-"` produces `my-app-2026-04-09.log`).
- Valid `log_level` values are: `error`, `warn`, `info`, `debug`, `trace`.
- `allow_signup` controls whether new users can self-register. Set to `false` to disable public registration.
- `oauth.enabled` is the master switch — when `false`, no OAuth buttons render in any client and the per-provider sections below are not validated.
- `oauth.connector` selects the implementation. `internal` ships in v1; `auth0` is reserved for a future drop-in and will error with a clear "not yet implemented" message at startup.
- OAuth client secrets are **always** read from the environment variable named in `client_secret_env`, never from `config.toml`. On startup the server walks every enabled provider and reports *all* misconfigurations in one error (empty `client_id`, missing env var, etc.) rather than fix-restart-fix-restart looping. See the **OAuth Sign-In** subsection below for full setup steps.


## Web Frontend

A React Single Page Application (SPA) is available at [`src/react/maze_web_server/`](../../../src/react/maze_web_server/README.md). Build it and point `static_dir` at the output:

```bash
cd src/react/maze_web_server
npm install
npm run build
```

Then set `static_dir` in `config.toml`:

```toml
static_dir = "../../react/maze_web_server/dist"
```

The server will serve `index.html` for all non-API routes, enabling client-side routing. If `static_dir` does not exist or is not set, the server runs as API-only.

## Authentication

The server supports two authentication mechanisms:

| Mechanism | Header | Usage |
|:----------|:-------|:------|
| Static API key | `X-API-Key: <key>` | API access; key is a UUID stored per user in the data store |
| Bearer token | `Authorization: Bearer <token>` | Per-user login; token obtained via `POST /api/v1/login` |

The following endpoints manage user identity:

| Method | Path | Auth required | Description |
|:-------|:-----|:--------------|:------------|
| `POST` | `/api/v1/signup` | None | Register a new (non-admin) user account; requires email and password only — username is auto-generated from the email local part |
| `POST` | `/api/v1/login` | None | Sign in; returns a bearer token |
| `POST` | `/api/v1/logout` | Bearer | Invalidate the current bearer token |
| `GET` | `/api/v1/auth/oauth/{provider}/start` | None | Begin an OAuth sign-in flow; 302-redirects to the provider's consent page (see **OAuth Sign-In** below) |
| `GET` | `/api/v1/auth/oauth/{provider}/callback` | None | Provider redirects here after consent; mints a bearer token and redirects back to the SPA or mobile app |
| `GET` | `/api/v1/users/me` | Either | Return the signed-in user's profile |
| `PUT` | `/api/v1/users/me/profile` | Either | Update the signed-in user's profile (username, full name, email) |
| `PUT` | `/api/v1/users/me/password` | Either | Change the signed-in user's password |
| `DELETE` | `/api/v1/users/me` | Either | Delete the signed-in user's account and all their mazes |

In addition, `GET /api/v1/features` returns an `oauth_providers` array describing the canonical name and human-readable display name of each provider currently enabled — clients render one button per entry.

The full API reference (including maze and admin-user endpoints) is available interactively via the documentation endpoints listed above.

### Password Requirements

The following password complexity rules apply when creating an account (`POST /api/v1/signup`) or changing a password (`PUT /api/v1/users/me/password`):

| Rule | Requirement |
|:-----|:------------|
| Minimum length | 8 characters |
| Uppercase letter | At least one (`A`–`Z`) |
| Lowercase letter | At least one (`a`–`z`) |
| Digit | At least one (`0`–`9`) |
| Special character | At least one non-alphanumeric character (e.g. `!`, `@`, `#`) |

A password such as `Password1!` satisfies all rules.

> **Note:** These rules are enforced server-side. The MAUI client also validates them locally before submitting the request.

### OAuth Sign-In

The server supports a server-mediated OAuth / OIDC sign-in flow behind a pluggable `OAuthConnector` trait. v1 ships with the `InternalOAuthConnector` (built on the [`oauth2`](https://crates.io/crates/oauth2) and [`openidconnect`](https://crates.io/crates/openidconnect) crates) which speaks OAuth/OIDC directly to each configured provider. A future `Auth0Connector` (or other broker) can be added as a drop-in implementation of the same trait without touching the handler layer or storage.

#### Provider setup

For each provider you want to enable, register an OAuth client with the provider:

| Provider | Where | Notes |
|:---------|:------|:------|
| **Google** | [Google Cloud Console](https://console.cloud.google.com/apis/credentials) → Create OAuth client ID → Web application | Add the value of `redirect_uri` to **Authorized redirect URIs**. Copy Client ID into `client_id`; the generated client secret goes in the env var named by `client_secret_env`. |
| **GitHub** | [GitHub Developer Settings](https://github.com/settings/developers) → New OAuth App | Set **Authorization callback URL** to match `redirect_uri` exactly. Copy Client ID into `client_id`; the generated client secret goes in the env var. The signed-in GitHub account must have a verified primary email at <https://github.com/settings/emails> for sign-in to succeed. |

Then in `config.toml` (for Google):

```toml
[oauth]
enabled = true
connector = "internal"
mobile_redirect_scheme = "maze-app"

[oauth.internal.providers.google]
enabled = true
display_name = "Google"
client_id = "<Google client id>"
client_secret_env = "MAZE_OAUTH_GOOGLE_SECRET"
redirect_uri = "https://your-host:8443/api/v1/auth/oauth/google/callback"
```

…and set the corresponding environment variable before starting the server:

```powershell
$env:MAZE_OAUTH_GOOGLE_SECRET = "<Google client secret>"
```

```bash
export MAZE_OAUTH_GOOGLE_SECRET="<Google client secret>"
```

The server's `redirect_uri` and the provider's registered redirect URI must match **exactly** (scheme, host, port, path).

#### Account resolution

When a callback arrives the server applies these rules in order:

1. **Returning OAuth user** — if `(provider, provider_user_id)` is already linked to a user, sign that user in. The user's `provider_email` and `last_seen_at` are refreshed from the latest provider response.
2. **First-time-OAuth, email matches an existing user** — if the provider asserts a verified email that matches an existing password (or other OAuth) account, append a new OAuth identity to that user and sign them in. **Not** gated by `allow_signup` — attaching a sign-in method to an existing account is not the same as creating one.
3. **First-time-OAuth, no matching account** — create a new user with an empty `password_hash` and the OAuth identity attached. **Only this branch is gated by `allow_signup`.** Username is auto-generated from the email local part with a `_2`, `_3`, … suffix on collision.

OAuth-only users (those created via branch 3) cannot sign in via `POST /api/v1/login` — `verify_password` is hardened to reject empty / non-Argon2 hashes, returning `401 Invalid email or password`. They sign in only via the OAuth flow.

> **Mismatched-email edge case (deliberately unsolved in v1):** if a user's existing password account uses a different email than their Google / GitHub account, branch (2) cannot see the connection and falls through to branch (3), creating a duplicate user. A future "Linked accounts" UI in My Account will provide an explicit-link path that side-steps this.

### Default Admin Account

On first run, if no admin user exists in the data store, the server automatically creates one with the following credentials:

| Field | Value |
|:------|:------|
| Username | `admin` |
| Email | `admin@maze.local` |
| Password | `Admin1!` |

Sign in using the **email address** and password. The username is used for display purposes only.

> **Important:** The default password is intentionally simple. **Change it immediately after first sign-in** using the self-service endpoint (`PUT /api/v1/users/me/password`) or the admin user-management API (`PUT /api/v1/users/{id}`).
