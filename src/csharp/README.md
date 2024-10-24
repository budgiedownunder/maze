# C# Components 

## Introduction
The following `C#` (`.NET`) components are present:

| Folder      | Component                                                   | Description 
|--------------|------------------------------------------------------------|---------------
| `src/csharp` | [`Maze.Api`](./Maze.Api/README.md)                         | .NET API that sits above  [`Maze.Wasm.Interop`](./Maze.Wasm.Interop/README.md)
|              | [`Maze.Api.Tests`](./Maze.Api.Tests/README.md)             | Unit tests for [`Maze.Api`](./Maze.Api/README.md)
|              | [`Maze.Maui.App`](./Maze.Maui.App/README.md)               | Maze [MAUI](https://dotnet.microsoft.com/en-us/apps/maui) application|              
|              | [`Maze.Wasm.Interop`](./Maze.Wasm.Interop/README.md)       | .NET interop to `maze_wasm` web assembly|
|              | [`Maze.Wasm.Interop.Tests`](./Maze.Wasm.Interop/README.md) | .NET test library for [`Maze.Wasm.Interop`](./Maze.Wasm.Interop/README.md) 

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

Windows:

```
dotnet build Build-Windows.sln
```

Non-Windows:

```
dotnet build Build-Non-Windows.sln
```

### Testing
To test all components, run the following from the `csharp` directory:

Windows:

```
dotnet test Build-Windows.sln
```

Non-Windows:

```
dotnet test Build-Non-Windows.sln
```

### Benchmarking
There are no benchmarking tests currently configured for these components.

### Generating Documentation

#### `.NET`-only 
To generate and view documentation for the project's `.NET` (`C#`) assemblies, with placeholder pages for where the `Rust` documentation would be, run the following from the `./src/docfx` directory:

Windows:

```
build_all.bat
serve.bat
```

Linux/macOS:

```
sh build_all.sh
sh serve.sh
```

#### Combined (`.NET` and `Rust`)
To generate and view combined documentation for the project's `.NET` (`C#`) assemblies and `Rust` crates, refer to the [README](../docfx/README.md) in the `docfx` project.
