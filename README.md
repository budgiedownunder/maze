# maze-project

## Table of Contents
- [Introduction](#introduction)
- [Getting Started](#getting-started)
- [Documentation](docs/README.md)
- [Contributing](#contributing)
- [License](#license)

## Introduction
This is an experimental project that has been created for exploring various programming languages, technologies and language-to-language integration. At its core, it contains a set of tools and libraries for managing and solving mazes that are then utilised in various application scenarios.

The following components are present:

| Folder | Component | Description |
|--------|-----------|--------------- |
| `src/rust` | [`maze`](./src/rust/maze/README.md) | maze definition and calculation library|
|            | [`maze_console`](./src/rust/maze_console/README.md) | maze console application |
|            | [`maze_wasm`](./src/rust/maze_wasm/README.md) | maze web assembly library |
|            | [`storage`](./src/rust/storage/README.md) | maze storage library |
|            | [`utils`](./src/rust/utils/README.md) | utilties library |

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
wasm-pack build --target web
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
To generate and view `Rust` (crate) documentation in your default browser:
```
cd src/rust
cargo doc --open
```

## Contributing
At this stage, this project is not accepting contributions.

## License
This software is licensed under the [MIT License](./LICENSE)
