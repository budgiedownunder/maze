# maze-project

## Table of Contents
- [Introduction](#introduction)
- [Getting Started](#getting-started)
- [Documentation](docs/README.md)
- [Contributing](#contributing)
- [License](#license)

## Introduction
This is an experimental project that I have created for exploring various programming languages, technologies and language-to-language integration. At its core, it contains a set of tools and libraries for managing and solving mazes that are then utilised in various application scenarios.

The following components are present:

| Folder | Component | Description |
|--------|-----------|--------------- |
| `src/csharp` | [`Maze.Wasm.Interop`](./src/csharp/Maze.Wasm.Interop/README.md) | .NET interop to `maze_wasm` web assembly|
|            | [`Maze.Wasm.Interop.Tests`](./src/csharp/Maze.Wasm.Interop/README.md) | .NET test library for [`Maze.Wasm.Interop`](./src/csharp/Maze.Wasm.Interop/README.md) |
| `src/rust` | [`maze`](./src/rust/maze/README.md) | maze definition and calculation library|
|            | [`maze_console`](./src/rust/maze_console/README.md) | maze console application |
|            | [`maze_wasm`](./src/rust/maze_wasm/README.md) | maze web assembly library |
|            | [`storage`](./src/rust/storage/README.md) | maze storage library |
|            | [`utils`](./src/rust/utils/README.md) | utilities library |

## Getting Started

### Setup
To setup the build and test environment, you first need to install:

- [`.NET 8.0+`](https://dotnet.microsoft.com/en-us/download)
- [`Rust`](https://www.rust-lang.org/tools/install)
- [`Node.js`](https://nodejs.org/en/learn/getting-started/how-to-install-nodejs) 

and then:

```
cd src/rust
cargo install wasm-pack
cd maze_wasm/tests/js
npm install
cd ../csharp
dotnet restore
```
### Build
To build all project components:
```
cd src/rust
cargo build
cd maze_wasm
wasm-pack build --target web --features "wasm-bindgen"
cargo build --target wasm32-unknown-unknown --release
cd ../../csharp
dotnet build
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

#### 3. .NET Components
To test all .NET components:
```
cd src/charp
dotnet test
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
To generate and view documentation for individual `.NET` assemblies, refer to the `README.md` file for each assembly.

## Contributing
At this stage, this project is not accepting contributions.

## License
This software is licensed under the [MIT License](./LICENSE)
