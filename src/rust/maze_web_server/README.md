# `maze_web_server` Crate

## Introduction

The `maze_web_server` crate is written in `Rust` and is a web server console application that hosts the `REST`-compliant `Maze Web API`.

It leverages the `Rust` library crates for calculation and generation ([`maze`](../maze/README.md)) and storage ([`storage`](../storage/README.md)). It then exposes them using [`actix`](https://actix.rs/) to serve the API and [`utoipa`](https://docs.rs/utoipa/latest/utoipa/) to publish it as an [`OpenAPI`](https://www.openapis.org/)-compliant interface for use in third party products such as [`Swagger`](https://swagger.io/). 

In addition to the API interfaces, it also supports the following documentation endpoints:

| EndPoint                  | Description
|:--------------------------|:------------
| `/api-docs/v1/rapidoc`    | [RapiDoc](https://rapidocweb.com/) 
| `/api-docs/v1/redoc`      | [ReDoc](https://redocly.com/)
| `/api-docs/v1/swagger-ui/`| [Swagger UI](https://swagger.io/tools/swagger-ui/)

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
|          | `auth_token`       | Text    | -          | `MAZE_WEB_SERVER_SECURITY_AUTH_TOKEN`
| Logging  | `log_dir`          | Text    | `logs`            | `MAZE_WEB_SERVER_LOGGING_LOG_DIR`
|          | `log_level`        | Text    | `info`            | `MAZE_WEB_SERVER_LOGGING_LOG_LEVEL`
|          | `log_file_prefix`  | Text    | `maze_web_server_`| `MAZE_WEB_SERVER_LOGGING_LOG_FILE_PREFIX`

These can also be set in a local configuration file called `config.toml` as follows

```toml
port = 8443

[security]
cert_file = "cert.pem"
key_file = "key.pem"

[logging]
log_dir = "logs"
log_level = "info"
```

Notes:

- Any environment variable values will take precedence over their corresponding configuration file values.
- `log_dir` is relative to the server working directory. Log files are named `{log_file_prefix}{YYYY-MM-DD}.log` and a new file is started each calendar day. Old log files are not deleted automatically.
- `log_file_prefix` is used verbatim — include any desired separator as the final character (e.g. `"maze_web_server_"` produces `maze_web_server_2026-04-09.log`, while `"my-app-"` produces `my-app-2026-04-09.log`).
- Valid `log_level` values are: `error`, `warn`, `info`, `debug`, `trace`.

> `auth_token` is a static API key used for privileged (admin) or service-to-service access via the `X-API-Key` request header. Regular users authenticate via the `POST /api/v1/login` endpoint, which returns a short-lived bearer token.

## Authentication

The server supports two authentication mechanisms:

| Mechanism | Header | Usage |
|:----------|:-------|:------|
| Static API key | `X-API-Key: <auth_token>` | Admin / service access; configured via `auth_token` |
| Bearer token | `Authorization: Bearer <token>` | Per-user login; token obtained via `POST /api/v1/login` |

The following endpoints manage user identity:

| Method | Path | Auth required | Description |
|:-------|:-----|:--------------|:------------|
| `POST` | `/api/v1/signup` | None | Register a new (non-admin) user account |
| `POST` | `/api/v1/login` | None | Sign in; returns a bearer token |
| `POST` | `/api/v1/logout` | Bearer | Invalidate the current bearer token |
| `GET` | `/api/v1/users/me` | Either | Return the signed-in user's profile |
| `PUT` | `/api/v1/users/me/profile` | Either | Update the signed-in user's profile (username, full name, email) |
| `PUT` | `/api/v1/users/me/password` | Either | Change the signed-in user's password |
| `DELETE` | `/api/v1/users/me` | Either | Delete the signed-in user's account and all their mazes |

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

### Default Admin Account

On first run, if no admin user exists in the data store, the server automatically creates one with the following credentials:

| Field | Value |
|:------|:------|
| Username | `admin` |
| Password | `Admin1!` |

> **Important:** The default password is intentionally simple. **Change it immediately after first login** using the self-service endpoint (`PUT /api/v1/users/me/password`) or the admin user-management API (`PUT /api/v1/users/{id}`).

The admin account is used with the bearer token mechanism (`POST /api/v1/login`) or, for service access, via the static API key configured in `auth_token`.