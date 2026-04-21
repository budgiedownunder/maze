# `Maze.Interop.Tests` Assembly

## Introduction

The `Maze.Interop.Tests` .NET assembly incorporates the tests for the [`Maze.Interop`](../Maze.Interop/README.md) .NET assembly.

Tests cover the full `MazeInterop` surface: maze operations, maze generation, and `MazeGame`. Each test is defined once in an abstract base class and run against multiple concrete connector implementations. `DisableTestParallelization = true` is set at the assembly level — all tests run sequentially.

| Class | Connector | Platform |
|:------|:----------|:---------|
| `MazeInteropWasmtimeTest` | Wasmtime (loads .wasm from disk) | Cross-platform |
| `MazeInteropWasmtimeTestFromBytes` | Wasmtime (receives .wasm bytes from caller) | Cross-platform |
| `MazeInteropWasmerTest` | Wasmer (loads .wasm from disk) | Windows only |
| `MazeInteropWasmerTestFromBytes` | Wasmer (receives .wasm bytes from caller) | Windows only |
| `MazeInteropNativeConnectorTest` | Native P/Invoke | iOS only — manual |

`MazeInteropNativeConnectorTest` is compiled only when targeting iOS (`#if IOS`) and must be run manually on an iOS simulator or physical device.

## Getting Started

### Setup
To set up the build environment, run the following from the `Maze.Interop.Tests` directory:

```
dotnet restore
```

### Build
To build the `Maze.Interop.Tests` assembly, run the following from the `Maze.Interop.Tests` directory:

```
dotnet build
```

### Testing
To run the tests, run the following command from the `Maze.Interop.Tests` directory:

```
dotnet test
```
