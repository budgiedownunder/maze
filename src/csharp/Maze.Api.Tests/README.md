# `Maze.Api.Tests` Assembly

## Introduction

The `Maze.Api.Tests` .NET assembly incorporates the tests for the [`Maze.Api`](../Maze.Api/README.md) .NET assembly.

Tests cover the three public types: `Maze`, `Solution`, and `MazeGame`.

Each test is defined once in an abstract base class and run against both Wasmtime and Wasmer connector variants (Static and NonStatic interop), so the reported test count is a multiple of the number of active concrete fixture classes. `DisableTestParallelization = true` is set at the assembly level — all tests run sequentially.

## Getting Started

### Setup
To set up the build environment, run the following from the `Maze.Api.Tests` directory:

```
dotnet restore
```

### Build
To build the `Maze.Api.Tests` assembly, run the following from the `Maze.Api.Tests` directory:

```
dotnet build
```

### Testing
To run the tests, run the following command from the `Maze.Api.Tests` directory:

```
dotnet test
```

### Linting
To verify code formatting and analyzer rules, run the following from the `Maze.Api.Tests` directory:

```
dotnet format --verify-no-changes --severity info
```

To autofix violations, run:

```
dotnet format --severity info
```

The expected output is zero errors and zero warnings.
