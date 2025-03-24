# `maze_wasm` Crate

## Introduction

The `maze_wasm` crate is written in `Rust` and defines the Web Assembly library for defining and solving mazes in consumer applications that support Web Assembly (WASM).

The crate uses `wasm-pack` to generate a JavaScript API wrapper `maze_wasm.js` to the WASM, and uses `cargo` to build the general Web Assembly `maze_wasm.wasm` for use outside of JavaScript.

## Getting Started

### Setup
To setup the build and test environment, run the following from the `maze_wasm` directory:

```
cargo install wasm-pack
cd tests/js
npm install
```

### Build
To build the `maze_wasm` crate and related resources, you need to run commands from within the `maze_wasm` directory.

To build the JavaScript API wrapper in the `./pkg` sub-directory, run:

```
wasm-pack build --target web --features "wasm-bindgen"
```

To build the general Web Assembly `maze_wasm.wasm` (for use outside of JavaScript), run:

```
cargo build --target wasm32-unknown-unknown --release --no-default-features --features "uuid-disable-random"
```

This will generate the release package for `maze_wasm.wasm` in the following directory:

`./src/rust/target/wasm32-unknown-unknown/release`

### Testing
To test the `maze_wasm` crate and the JavaScript API wrapper, run the following from within the `maze_wasm` directory:
```
cargo test --features "uuid-disable-random"
cd tests/js
npm run test_api
npm run test_help_examples
```

### Benchmarking
No benchmarking tests are currently implemented for the crate

### Generating Documentation
To generate and view `Rust` documentation for the crate in your default browser, run the following from within the `maze_wasm` directory depending on what type of build you require the documentation for.

To generate documentation for the JavaScript API (corresponding to the `wasm-bindgen` feature):
```
cargo doc --features "wasm-bindgen" --open
```

To generate documentation for the generalised Web Assembly API:
```
cargo doc --open
```

## WebAssembly Target Compatibility

This crate supports both **JavaScript/WebAssembly** builds and **general-purpose WebAssembly** builds for use in non-JS environments such as Wasmtime, .NET, or other native hosts, with the `uuid-disable-random` feature flag used to disable randomness in those environments that do not support it.
