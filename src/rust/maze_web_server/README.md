# `maze_web_server` Crate

## Introduction

The `maze_web_server` crate is written in `Rust` and is a web server console application that hosts the `REST`-compliant `Maze Web API`.

It leverages the `Rust` library crates for calculation ([`maze`](../maze/README.md)) and storage ([`storage`](../storage/README.md)). It then exposes them using [`actix`](https://actix.rs/) to serve the API and [`utoipa`](https://docs.rs/utoipa/latest/utoipa/) to publish it as an [`OpenAPI`](https://www.openapis.org/)-compliant interface for use in third party products such as [`Swagger`](https://swagger.io/). 

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

These curremtly have a 365 day expiry of `18-JAN-2025`. Hence, they will need to be renewed after this time has elapsed by using tools such as `openssl` or, for production, 
a trusted Certificate Authority (e.g. Let's Encrypt).

The following command will regenerate these files using `openssl`:

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
| Global   | `port`       | Integer | `8443`           | `MAZE_WEB_SERVER_PORT`
| Security | `cert_file`  | Text    | `cert.pem`       | `MAZE_WEB_SERVER_SECURITY_CERT_FILE`
|          | `key_file`   | Text    | `key.pem`        | `MAZE_WEB_SERVER_SECURITY_KEY_FILE`
|          | `auth_token` | Text    |  -               | `MAZE_WEB_SERVER_SECURITY_AUTH_TOKEN`

These can also be set in a local configuration file called `config.toml` as follows

``` 
port = 8443

[security]
cert_file = "cert.pem"
key_file = "key.pem"
auth_token = "0595C1D2-6341-44BF-BB34-C2E350A8AD72"
```

Note:

Any environment variable values will take precedence over their corresponding configuration file values.