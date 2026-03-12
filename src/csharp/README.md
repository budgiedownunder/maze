# C# Components 

## Introduction
The following `C#` (`.NET`) components are present:

| Folder      | Component                                                   | Description 
|--------------|------------------------------------------------------------|---------------
| `src/csharp` | [`Maze.Api`](./Maze.Api/README.md)                         | .NET API that sits above  [`Maze.Wasm.Interop`](./Maze.Wasm.Interop/README.md)
|              | [`Maze.Api.Tests`](./Maze.Api.Tests/README.md)             | Unit tests for [`Maze.Api`](./Maze.Api/README.md)
|              | [`Maze.Maui.App`](./Maze.Maui.App/README.md)               | Maze [MAUI](https://dotnet.microsoft.com/md) application
|              | [`Maze.Maui.Controls`](./Maze.Maui.Controls/README.md)     | Maze [MAUI](https://dotnet.microsoft.com/en-usen-us/apps/maui) custom controls              
|              | [`Maze.Maui.Services`](./Maze.Maui.Services/README.md)     | Maze [MAUI](https://dotnet.microsoft.com/en-usen-us/apps/maui) custom services              
|              | [`Maze.Wasm.Interop`](./Maze.Wasm.Interop/README.md)       | .NET interop to `maze_wasm` web assembly
|              | [`Maze.Wasm.Interop.Tests`](./Maze.Wasm.Interop/README.md) | .NET test library for [`Maze.Wasm.Interop`](./Maze.Wasm.Interop/README.md) 

## Getting Started

### Setup
To setup the build and test environment for `.NET`, you first need to install:

- [`.NET 10.0+`](https://dotnet.microsoft.com/en-us/download)

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

Before testing, you must also ensure that the Rust WebAssembly file `maze_wasm.wasm` has been built for the `wasm32-unknown-unknown` target. This is required for it to be used outside of JavaScript, as described in the `maze_wasm` [README](../rust/maze_wasm/README.md).

If this is not done, the WebAssembly will fail to load in `.NET` via `Wasmtime` and errors such as:
```
Error while importing "__wbindgen_placeholder__"."__wbindgen_describe": unknown import
```
will be thrown.

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

#### Combined (`.NET` and `Rust`)
To generate and view combined documentation for the project's `.NET` (`C#`) assemblies and `Rust` crates, refer to the [README](../docfx/README.md) in the `docfx` project.



 
