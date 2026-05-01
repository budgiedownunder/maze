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
| Storage  | `storage.type`               | Text (`file` / `sql`) | `file` | `MAZE_WEB_SERVER_STORAGE_TYPE`
|          | `storage.file.data_dir`      | Text    | `data`  | `MAZE_WEB_SERVER_STORAGE_FILE_DATA_DIR`
|          | `storage.sql.driver`         | Text (`sqlite` / `postgres` / `mysql`) | `sqlite` | `MAZE_WEB_SERVER_STORAGE_SQL_DRIVER`
|          | `storage.sql.host`           | Text    | (empty) | `MAZE_WEB_SERVER_STORAGE_SQL_HOST`
|          | `storage.sql.port`           | Integer | `0`     | `MAZE_WEB_SERVER_STORAGE_SQL_PORT`
|          | `storage.sql.database`       | Text    | (empty) | `MAZE_WEB_SERVER_STORAGE_SQL_DATABASE`
|          | `storage.sql.username`       | Text    | (empty) | `MAZE_WEB_SERVER_STORAGE_SQL_USERNAME`
|          | `storage.sql.password`       | Text    | (env-var only — never read from config files) | `MAZE_WEB_SERVER_STORAGE_SQL_PASSWORD`
|          | `storage.sql.path`           | Text    | `maze.db` | `MAZE_WEB_SERVER_STORAGE_SQL_PATH`
|          | `storage.sql.max_connections` | Integer | `5`    | `MAZE_WEB_SERVER_STORAGE_SQL_MAX_CONNECTIONS`
|          | `storage.sql.auto_create_database` | Boolean | `false` | `MAZE_WEB_SERVER_STORAGE_SQL_AUTO_CREATE_DATABASE`
|          | `storage.sql.require_tls`    | Boolean | `false` | `MAZE_WEB_SERVER_STORAGE_SQL_REQUIRE_TLS`
|          | `storage.sql.ca_cert_path`   | Text    | (empty) | `MAZE_WEB_SERVER_STORAGE_SQL_CA_CERT_PATH`
|          | `storage.sql.connect_timeout_secs` | Integer | `10` | `MAZE_WEB_SERVER_STORAGE_SQL_CONNECT_TIMEOUT_SECS`
|          | `storage.sql.idle_timeout_secs` | Integer | `600` | `MAZE_WEB_SERVER_STORAGE_SQL_IDLE_TIMEOUT_SECS`
|          | `storage.sql.acquire_timeout_secs` | Integer | `30` | `MAZE_WEB_SERVER_STORAGE_SQL_ACQUIRE_TIMEOUT_SECS`

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

[storage]
# Backend selector: "file" (on-disk JSON layout) or "sql" (SQLite/Postgres/MySQL).
type = "file"

[storage.file]
# Directory under which user/maze data is stored, relative to the working
# directory or absolute.
data_dir = "data"

# ---- SQL backend ----
# To switch to a SQL backend, set type = "sql" above and uncomment the
# block below. Driver selection happens at runtime — one binary supports
# all three engines via SQLx's Any backend. The connection URL is
# assembled from these fields at startup. The password is *never*
# stored here — set MAZE_WEB_SERVER_STORAGE_SQL_PASSWORD instead
# (sqlite is exempt — it has no network user).
# [storage.sql]
# driver = "postgres"            # "postgres", "mysql", or "sqlite"
# host = "your-db-host"          # postgres / mysql only
# port = 5432                    # postgres / mysql only
# database = "your_database"     # postgres / mysql only
# username = "your_app_user"     # postgres / mysql only
# path = "your_database.db"      # sqlite only
# max_connections = 5
# auto_create_database = false   # sqlite + dev only — cloud creds rarely have the privilege
# require_tls = false            # set true for any host beyond localhost
# ca_cert_path = ""              # optional CA bundle for full TLS verification
# connect_timeout_secs = 10
# idle_timeout_secs = 600
# acquire_timeout_secs = 30

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

[oauth.internal.providers.facebook]
enabled = false
display_name = "Facebook"
client_id = ""
client_secret_env = "MAZE_OAUTH_FACEBOOK_SECRET"
redirect_uri = "https://your-host:8443/api/v1/auth/oauth/facebook/callback"
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
- The `[storage]` section selects between the file-backed (`type = "file"`, the default) and SQL-backed (`type = "sql"`) implementations. The SQL backend supports SQLite, PostgreSQL, and MySQL via SQLx's `Any` driver — all three engines are compiled into the same binary; selection happens at runtime via `storage.sql.driver` and the connection details. See **Storage Backend** below for setup recipes per backend.

## Storage Backend

The server stores users, maze definitions, OAuth identities, and login tokens in a pluggable backend selected by `storage.type`.

### When to use which

| Backend | Best for | Setup |
|:--------|:---------|:------|
| `file` | Local dev, single-instance, zero infrastructure | None — server creates `data/` on first run |
| `sql` + `sqlite` | Local dev, single-instance with relational guarantees, low-traffic self-hosted production | None — `auto_create_database = true` creates the `.db` file on first run |
| `sql` + `postgres` | Networked / multi-instance production, cloud deployments | Operator pre-provisions the database and grants the app user (see below) |
| `sql` + `mysql` | Networked / multi-instance production, MySQL-shop deployments | Same operator pattern as PostgreSQL |

### Example configurations

Runnable starter configs are checked in alongside this README:

| File | Description |
|:-----|:------------|
| [`config.example.sqlite.toml`](./config.example.sqlite.toml) | SQLite — no infrastructure, file at `maze.db` |
| [`config.example.postgres.toml`](./config.example.postgres.toml) | PostgreSQL on `localhost` (Docker / LAN), TLS off |
| [`config.example.postgres-cloud.toml`](./config.example.postgres-cloud.toml) | Cloud-managed PostgreSQL (RDS / Cloud SQL / Azure DB), TLS required, longer timeouts, no auto-create |
| [`config.example.mysql.toml`](./config.example.mysql.toml) | MySQL on `localhost` (Docker / LAN), TLS off |

Copy the relevant file over `config.toml` (or merge its `[storage]` block in) and adjust hostnames, usernames, etc. Set `MAZE_WEB_SERVER_STORAGE_SQL_PASSWORD` in the environment before starting the server when using `postgres` or `mysql`.

### Two-phase migration model (PostgreSQL / MySQL)

Production databases are managed in two phases with distinct privileges. The application **never** runs `CREATE DATABASE` or `CREATE USER` against a production server — those are operator-only steps.

**Phase 1 — Operator (one-time, before the app first connects):**

1. Create the database server instance (Docker / managed cloud service / on-prem install).
2. Create the application's database.
3. Create an application user with `CREATE TABLE` rights inside the database, but no server-level admin rights.

PostgreSQL:
```sql
CREATE DATABASE your_database;
CREATE USER your_app_user WITH PASSWORD '<your_app_password>';
GRANT CONNECT ON DATABASE your_database TO your_app_user;
\c your_database
GRANT USAGE, CREATE ON SCHEMA public TO your_app_user;
```

MySQL:
```sql
CREATE DATABASE your_database CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
CREATE USER 'your_app_user'@'%' IDENTIFIED BY '<your_app_password>';
GRANT CREATE, ALTER, DROP, INDEX, REFERENCES, SELECT, INSERT, UPDATE, DELETE
    ON your_database.* TO 'your_app_user'@'%';
FLUSH PRIVILEGES;
```

The app user gets `CREATE TABLE` rights inside the database but cannot create or drop other databases on the same server. Replace `your_database`, `your_app_user`, and `<your_app_password>` with your own values.

**Phase 2 — Application startup (every deployment):**

4. App connects with the app-user credentials to the pre-existing database.
5. SQLx applies any pending migrations from `storage/migrations/` automatically — this is when `CREATE TABLE` statements run.
6. SQLx tracks applied migrations in its own `_sqlx_migrations` table, so subsequent restarts skip migrations that have already been applied.

**Schema changes** (future migrations) ship as new `0002_*.sql` files alongside `0001_initial.sql` and apply automatically on the next deploy. The same binary runs against dev/staging/prod — only the connection config differs.

**`auto_create_database = true`** is for local dev / SQLite only. PostgreSQL and MySQL cloud credentials typically lack the server-level `CREATEDB`/`CREATE` privilege required for it to work, and managed databases are usually pre-provisioned by IaC (Terraform / CloudFormation / Bicep) anyway.

### TLS

`require_tls = true` enforces TLS for the connection. The URL gets driver-appropriate query parameters appended at startup:

| Driver | Without `ca_cert_path` | With `ca_cert_path` |
|:-------|:-----------------------|:--------------------|
| `postgres` | `?sslmode=require` (TLS used, cert not verified) | `?sslmode=verify-full&sslrootcert=<path>` (full verification) |
| `mysql` | `?ssl-mode=REQUIRED` | `?ssl-mode=VERIFY_CA&ssl-ca=<path>` |
| `sqlite` | (ignored — no network) | (ignored) |

For cloud-managed databases, `ca_cert_path` should point at the provider's CA bundle (e.g. `rds-global-bundle.pem` for AWS RDS).

#### PostgreSQL TLS — local Docker recipe

The default `postgres:16` image ships with `ssl = off`. To enable TLS for a local TLS smoke-test you need to run the container with TLS enabled and a cert mounted. From a host with OpenSSL available:

```bash
# Generate a self-signed cert/key pair (one-off)
openssl req -x509 -nodes -newkey rsa:2048 \
    -keyout postgres-server.key -out postgres-server.crt \
    -days 365 -subj "/CN=localhost"

# Run postgres with TLS enabled and the cert mounted
docker run --name maze-postgres-tls \
    -e POSTGRES_PASSWORD=pw -p 5432:5432 \
    -v "$(pwd)/postgres-server.crt:/var/lib/postgresql/server.crt:ro" \
    -v "$(pwd)/postgres-server.key:/var/lib/postgresql/server.key:ro" \
    -d postgres:16 \
    -c ssl=on \
    -c ssl_cert_file=/var/lib/postgresql/server.crt \
    -c ssl_key_file=/var/lib/postgresql/server.key
```

(On Windows, replace `$(pwd)` with the absolute Windows path to your cert files.)

Then set `require_tls = true` in `config.toml` and start the server. Verify TLS is actually being used by querying the live connection state:

```bash
docker exec -it maze-postgres-tls psql -U postgres -d your_database \
    -c "SELECT datname, usename, ssl, version FROM pg_stat_ssl JOIN pg_stat_activity USING(pid) WHERE usename = 'your_app_user';"
```

`ssl = t` and a `version` of `TLSv1.2` or `TLSv1.3` confirms the pool's connections are encrypted.

#### MySQL TLS — already on by default

The `mysql:8` Docker image enables TLS automatically with an auto-generated self-signed cert. Set `require_tls = true` in `config.toml`, start the server, and verify per-connection TLS state via:

```bash
docker exec -it maze-mysql mysql -uroot -ppw \
    -e "SELECT processlist_user, processlist_host, connection_type FROM performance_schema.threads WHERE processlist_user = 'your_app_user';"
```

`connection_type = SSL/TLS` per pool connection confirms TLS is in use.


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
| `GET` | `/api/v1/users/me` | Either | Return the signed-in user's profile (includes `email`, `emails`, and `has_password` — see below) |
| `PUT` | `/api/v1/users/me/profile` | Either | Update the signed-in user's username and full name. **Email is no longer mutable here** — use `/api/v1/users/me/emails` instead. Sending an `email` field returns `400 Bad Request` |
| `PUT` | `/api/v1/users/me/password` | Either | **Sets or changes** the signed-in user's password — the same endpoint handles both flows (see **Password set-or-change** below) |
| `DELETE` | `/api/v1/users/me` | Either | Delete the signed-in user's account and all their mazes |
| `GET` | `/api/v1/users/me/emails` | Either | List the signed-in user's email addresses with primary/verified status |
| `POST` | `/api/v1/users/me/emails` | Either | Add a new email row (created `verified = true` for now; once email-send-support ships, this becomes `verified = false` until the user clicks the verify link) |
| `DELETE` | `/api/v1/users/me/emails/{email}` | Either | Remove an email; rejects with 409 if the address is the user's only email or their primary |
| `PUT` | `/api/v1/users/me/emails/{email}/primary` | Either | Promote an email to primary; rejects with 409 if the target is unverified |
| `POST` | `/api/v1/users/me/emails/{email}/verify` | Either | **Stub** — returns `501 Not Implemented` until the email-verification flow ships |

In addition, `GET /api/v1/features` returns an `oauth_providers` array describing the canonical name and human-readable display name of each provider currently enabled — clients render one button per entry.

The full API reference (including maze and admin-user endpoints) is available interactively via the documentation endpoints listed above.

### `GET /api/v1/users/me` shape

The response is a `UserItem` carrying:

- `id`, `is_admin`, `username`, `full_name`
- `email` — the **primary** email address (legacy single-field shape, preserved for backwards-compat)
- `emails` — the full list of email rows: `{ email, is_primary, verified, verified_at }` per row. Always at least one row; exactly one is `is_primary`
- `has_password` — `true` if a password is set, `false` for OAuth-only users who haven't yet added a password. Front-ends use this to choose between the "Change Password" and "Set Password" UI variants

### Password set-or-change

`PUT /api/v1/users/me/password` is a single endpoint that handles both setting an initial password (OAuth-only users adding a password as a second login method) and changing an existing one. The body shape is:

```json
{ "current_password": "...", "new_password": "..." }
```

`current_password` is **optional**, with branching driven by the user's existing state (which the client reads from `has_password` on `GET /me`):

| User state                  | Required body                                | Behaviour                                     |
|:----------------------------|:---------------------------------------------|:----------------------------------------------|
| `has_password = true`       | `current_password` + `new_password`          | Verify `current_password`, then rotate        |
| `has_password = false`      | `new_password` only (omit `current_password`)| Set initial password                          |

Mismatched shapes return `400 Bad Request`:
- Sending `current_password` to a user who doesn't have one yet (the "set" path)
- Omitting `current_password` for a user who does (the "change" path)

A wrong `current_password` on the change path returns `401 Unauthorized`.

### Password Requirements

The following password complexity rules apply when creating an account (`POST /api/v1/signup`) or setting/changing a password (`PUT /api/v1/users/me/password`):

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
| **Facebook** | [Facebook for Developers](https://developers.facebook.com) → My Apps → Create App → "Authenticate and request data from users" → add the **Facebook Login** product | Under Facebook Login → Settings, add the value of `redirect_uri` to **Valid OAuth Redirect URIs** (HTTPS only outside `localhost`). Copy App ID into `client_id`; the App Secret goes in the env var. The Facebook account must have an email on file (declining the `email` scope at consent results in `email_not_verified`). The app must be in **Live** mode (with privacy policy + terms URLs) for non-developer users to sign in — in dev mode only roles you explicitly add (Admin / Developer / Tester) can authenticate. Facebook does not expose an `email_verified` flag, so we treat the email as verified whenever it is present (matches Auth0/Clerk default). |

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

> **Mismatched-email edge case (deliberately unsolved in v1):** if a user's existing password account uses a different email than their OAuth provider account, branch (2) cannot see the connection and falls through to branch (3), creating a duplicate user. A future "Linked accounts" UI in My Account will provide an explicit-link path that side-steps this.

### Default Admin Account

On first run, if no admin user exists in the data store, the server automatically creates one with the following credentials:

| Field | Value |
|:------|:------|
| Username | `admin` |
| Email | `admin@maze.local` |
| Password | `Admin1!` |

Sign in using the **email address** and password. The username is used for display purposes only.

> **Important:** The default password is intentionally simple. **Change it immediately after first sign-in** using the self-service endpoint (`PUT /api/v1/users/me/password`) or the admin user-management API (`PUT /api/v1/users/{id}`).
