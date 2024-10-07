# C# Components 

## Introduction
The following `C#` (`.NET`) components are present:

| Folder | Component | Description |
|--------|-----------|--------------- |
| `src/csharp`| [`Maze.Wasm.Interop`](./src/csharp/Maze.Wasm.Interop/README.md) | .NET interop to `maze_wasm` web assembly|
|            | [`Maze.Wasm.Interop.Tests`](./src/csharp/Maze.Wasm.Interop/README.md) | .NET test library for [`Maze.Wasm.Interop`](./src/csharp/Maze.Wasm.Interop/README.md) |

## Getting Started

### Setup
To setup the build and test environment for `.NET`, you first need to install:

- [`.NET 8.0+`](https://dotnet.microsoft.com/en-us/download)

and then run the following from the `csharp` directory:

```
dotnet restore
```

### Build
To build all components, run the following from the `csharp` directory:

```
dotnet build
```

### Testing
To test all components, run the following from the `csharp` directory:

```
dotnet test
```

### Benchmarking
There are no benchmarking tests currently configured for these components.

### Generating Documentation

#### `.NET`-only 
To generate and view documentation for the project's `.NET` (`C#`) assemblies, with placeholder pages for where the `Rust` documentation would be, run the following from the `.src/docfx` directory:

Windows:

```
copy_files.bat
docfx docfx.json --serve
```

Linux/macOS:

```
sh copy_files.sh
docfx docfx.json --serve
```

#### Combined (`.NET` and `Rust`)
To generate and view combined documentation for the project's `.NET` (`C#`) assemblies and `Rust` crates, refer to the [README](../docfx/README.md) in the `docfx` project.
