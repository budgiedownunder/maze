# `maze_wasm` Crate

## Introduction

The `maze_wasm` crate is written in `Rust` and defines the Web Assembly library for defining and solving mazes in consumer applications that support Web Assembly (WASM).

The crate uses `wasm-pack` to generate a JavaScript API wrapper `maze_wasm.js` to the WASM.

## Getting Started

### Setup
To setup the build and test environment, run the following from the `maze_wasm` directory:

```
cargo install wasm-pack
cd tests/js
npm install
```

### Build
To build the `maze_wasm` crate, run the following from within the `maze_wasm` directory:
```
wasm-pack build --target web
```

### Testing
To test the `maze_wasm` crate and the JavaScript API wrapper, run the following from within the `maze_wasm` directory:
```
cargo test
cd tests/js
npm run test_api
npm run test_help_examples
```

### Benchmarking
No benchmarking tests are currently implemented for the crate

### Generating Documentation
To generate and view `Rust` documentation for the crate in your default browser, run the following from within the `maze_wasm` directory:
```
cargo doc --open
```