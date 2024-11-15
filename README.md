# maze-project

## Table of Contents
- [Introduction](#introduction)
- [Getting Started](#getting-started)
- [Documentation](docs/README.md)
- [Contributing](#contributing)
- [License](#license)

## Introduction
This is an experimental project that has been created for exploring various programming languages, technologies and language-to-language integration. At its core, it contains a set of tools and libraries for managing and solving mazes that are then utilised in various application scenarios.

At this stage, the following areas are covered:

- Creating library crates in `Rust`
- Creating a `Rust` console application ([`maze_console`](./src/rust/maze_console/README.md)) that leverages `Rust` library crates for calculation ([`maze`](./src/rust/maze/README.md)) and storage ([`storage`](./src/rust/storage/README.md))
- Implementing automated unit and mock testing (dependency injection) in `Rust` 
- Automating `Rust` documentation-generation with `cargo doc`
- Web Assembly implementation and generation (`wasm32` and `wasm-bindgen`) in `Rust` ([`maze_wasm`](./src/rust/maze_wasm/README.md))
- Generating JavaScript APIs from `Rust` crates (`wasm-pack`)
- Automating JavaScript API testing in `node.js` (`chai`, `mocha`)
- Implementing a `.NET` to Web Assembly ([`maze_wasm`](./src/rust/maze_wasm/README.md)) interop library ([`Maze.Wasm.Interop`](./src/csharp/Maze.Wasm.Interop/README.md)) in `C#`
- Implementing automated `.NET` API testing with `xUnit` ([`Maze.Wasm.Interop.Tests`](./src/csharp/Maze.Wasm.Interop.Tests/README.md))
- Implementing a `C#` [MAUI](https://dotnet.microsoft.com/en-us/apps/maui) application ([`Mazer.Maui.App`](./src/csharp/Maze.Maui.App/README.md)) that utilises an underlying Web Assembly interop library ([`Maze.Wasm.Interop`](./src/csharp/Maze.Wasm.Interop/README.md)) via a wrapper API ([`Maze.Api`](./src/csharp/Maze.Api/README.md))
- Automating `C#` API documentation generation with `DocFX`
- Combining `C#` and `Rust` documentation into a single HTML help system with use of `iFrame` containers
- Architecture diagramming using `PlantUML` ([`architecture.puml`](./docs/diagrams/architecture.puml))
- Automating image generation workflows using GitHub Actions ([`generate-png-from-puml.yml`](./.github/workflows/generate-png-from-puml.yml))
- Automating build and testing workflows using GitHub Actions ([`build-and-test-components-multi-os.yml`](./.github/workflows/build-and-test-components-multi-os.yml))
- Automating GitHub Pages asset generation and deployment [`build-and-deploy-to-github-pages`](./.github/workflows/build-and-deploy-to-github-pages.yml)

The following components are present:

| Folder                         | Component                                                                     | Description
|--------------------------------|-------------------------------------------------------------------------------|---------------
| `.github/workflows`            | `*.yml`                                                                       | GitHub Action workflow files
| `docs`                         | [`README.md`](./docs/README.md)                                               | Project overview documentation
| `research/alorithms/excel`     | `maze-algorithms.xls`                                                         | Excel workbook containing maze algorithms
| `src`                          | [`docfx`](./src/docfx/README.md)                                              | HTML help generation|
| `src/csharp`                   | [`Maze.Api`](./src/csharp/Maze.Api/README.md)                                 | .NET API that sits above  [`Maze.Wasm.Interop`](./src/csharp/Maze.Wasm.Interop/README.md)
|                                | [`Maze.Api.Tests`](./src/csharp/Maze.Api.Tests/README.md)                     | Unit tests for [`Maze.Api`](./src/csharp/Maze.Api/README.md)
|                                | [`Maze.Maui.App`](./src/csharp/Maze.Maui.App/README.md)                       | Maze [MAUI](https://dotnet.microsoft.com/en-us/apps/maui) application
|                                | [`Maze.Wasm.Interop`](./src/csharp/Maze.Wasm.Interop/README.md)               | .NET interop to `maze_wasm` web assembly|
|                                | [`Maze.Wasm.Interop.Tests`](./src/csharp/Maze.Wasm.Interop/README.md)         | .NET test library for [`Maze.Wasm.Interop`](./src/csharp/Maze.Wasm.Interop/README.md) |
| `src/rust`                     | [`maze`](./src/rust/maze/README.md)                                           | Maze definition and calculation library|
|                                | [`maze_console`](./src/rust/maze_console/README.md)                           | Maze console application
|                                | [`maze_wasm`](./src/rust/maze_wasm/README.md)                                 | Maze web assembly library |
|                                | [`storage`](./src/rust/storage/README.md)                                     | Maze storage library |
|                                | [`utils`](./src/rust/utils/README.md)                                         | Utilities library |

## Getting Started

### Setup
To setup the build and test environment, you first need to install:

- [`.NET 8.0+`](https://dotnet.microsoft.com/en-us/download)
- [`Rust`](https://www.rust-lang.org/tools/install)
- [`Node.js`](https://nodejs.org/en/learn/getting-started/how-to-install-nodejs) 

To setup the `C#` build environment, refer to the [README](src/csharp/README.md) in the `csharp` directory.

To setup the `Rust` build environment, refer to the [README](src/rust/README.md) in the `rust` directory.

### Build

- To build the `C#` (`.NET`) APIs, refer to the [README](src/csharp/README.md) in the `csharp` directory.

- To build the `Rust` crates, refer to the [README](src/rust/README.md) in the `rust` directory.

### Generating Documentation
- To generate combined documentation for the `.NET` APIs and `Rust` crates, refer to the [README](src/docfx/README.md) in the `docfx` project.

- To generate documentation just for the `.NET` APIs, refer to the [README](src/csharp/README.md) in the `csharp` directory.

- To generate documentation just for the `Rust` crates, refer to the [README](src/rust/README.md) in the `rust` directory.

## Contributing
At this stage, this project is not accepting contributions.

## License
This software is licensed under the [MIT License](./LICENSE)
