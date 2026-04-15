# Rust Crates


## Introduction
The following `Rust` crates are present:

| Folder | Crate | Description
|--------|-----------|---------------
| `src/rust` | [`maze`](./maze/README.md) | Maze definition and calculation library
|            | [`maze_c`](./maze_c/README.md) | Maze C API library
|            | [`maze_console`](./maze_console/README.md) | Maze console application
|            | [`maze_openapi_generator`](./maze_openapi_generator/README.md) | Maze OpenAPI generator console application
|            | [`maze_wasm`](./maze_wasm/README.md) | Maze WebAssembly API library
|            | [`maze_web_server`](./maze_web_server/README.md) | Maze web server console application
|            | [`storage`](./storage/README.md) | Maze storage library
|            | [`utils`](./utils/README.md) | Utilities library

## Getting Started

### Setup
To setup the build and test environment, you first need to install:

- [`Rust`](https://www.rust-lang.org/tools/install)
- [`Node.js`](https://nodejs.org/en/learn/getting-started/how-to-install-nodejs) 

and then:

```
cd src/rust
cargo install wasm-pack
cd maze_wasm/tests/js
npm install
```

### Build
To build all project crates and JavaScript APIs:

```
cd src/rust
cargo build
cd maze_wasm
wasm-pack build --target web -- --features "wasm-bindgen"
cargo build --target wasm32-unknown-unknown --release --features "wasm-lite"
```

### Run

To run the `maze_console` application:

```
cd src/rust/maze_console
cargo run
```

To run the `maze_web_server` application:

```
cd src/rust/maze_web_server
cargo run
```

To run the `maze_openapi_generator` application:

```
cd src/rust/maze_openapi_generator
cargo run
```

### Testing
#### 1. Rust Crates
To test all `Rust` crates:

```
cd src/rust
cargo test --locked -p auth
cargo test --locked -p data_model
cargo test --locked -p maze --features generation
cargo test --locked -p maze_c -- --test-threads=1
cargo test --locked -p maze_console -- --test-threads=1
cargo test --locked -p storage -- --test-threads=1
cargo test --locked -p maze_wasm
cargo test --locked -p maze_web_server
cargo test --locked -p maze_openapi_generator
cargo test --locked -p utils
```

#### 2. JavaScript APIs
To test the `maze_wasm.js` JavaScript API:

```
cd src/rust/maze_wasm/tests/js
npm run test_api
npm run test_help_examples
```

### Linting
To run Clippy across all crates and targets:

```
cd src/rust
cargo clippy --all-targets
```

Expected: zero errors, zero warnings.

### Benchmarking
To run benchmark tests (which are currently only configured for the `maze` crate):

```
cd src/rust
cargo bench -p maze
```

### Generating Documentation
To generate and view documentation for the project's `Rust` crates in your default browser (with all external dependency crates), run:

```
cd src/rust
cargo doc --open
```

To generate and view documentation for the project's `Rust` crates in your default browser (without any external dependency crates), run:

```
cd src/rust
cargo doc --no-deps --open
```
