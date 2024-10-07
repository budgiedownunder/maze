# Rust Components


## Introduction
The following `Rust` components are present:

| Folder | Component | Description |
|--------|-----------|--------------- |
| `src/rust` | [`maze`](./src/rust/maze/README.md) | maze definition and calculation library|
|            | [`maze_console`](./src/rust/maze_console/README.md) | maze console application |
|            | [`maze_wasm`](./src/rust/maze_wasm/README.md) | maze web assembly library |
|            | [`storage`](./src/rust/storage/README.md) | maze storage library |
|            | [`utils`](./src/rust/utils/README.md) | utilities library |

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
To build all project components:

```
cd src/rust
cargo build
cd maze_wasm
wasm-pack build --target web --features "wasm-bindgen"
cargo build --target wasm32-unknown-unknown --release
```

### Run

To run the `maze_console` application:

```
cd src/rust/maze_console
cargo run
```

### Testing
#### 1. Rust Components
To test all `Rust` components:

```
cd src/rust
cargo test -p maze
cargo test -p maze_console -- --test-threads=1
cargo test -p storage -- --test-threads=1
cargo test -p maze_wasm
```

#### 2. JavaScript APIs
To test the `maze_wasm.js` JavaScript API:

```
cd src/rust/maze_wasm/tests/js
npm run test_api
npm run test_help_examples
```

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
