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

### Benchmarking
No benchmarking tests are currently implemented for the crate

### Generating Documentation
To generate and view `Rust` documentation for the crate in your default browser, run the following from within the `maze_web_server` directory:
```
cargo doc --open
```