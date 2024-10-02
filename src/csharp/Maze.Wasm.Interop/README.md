# `Maze.Wasm.Interop` Assembly

## Introduction

The `Maze.Wasm.Interop` .NET assembly is written in `C#` and provides interop between .NET applications and the `maze_wasm.wasm` Web Assembly. Its purpose is to allow the Web Assembly's functionality to be used from with .NET applications, without needing to be concerned with the underlying .NET to Web Assembly interop specifics.

It exposes a singleton instance of a `MazeWasmInterop` object, which can be accessed via:

```csharp
var instance = MazeWasmInterop.GetInstance(string wasmPath);
```

where `wasmPath` is the path to the `maze_wasm` Web Assembly. The `GetInstance()` function enforces the singleton instance and, on initialisation, loads the `maze_wasm` Web Assembly and any required function entry points within it. If any functions are found to be missing, an exception will be thrown. 

Once the instance is obtained, the caller can execute methods to create and interact with `maze_wasm` objects . It is important that any object pointers that are returned to the caller are released using the appropriate `MazeWasmInterop` method. For example, to create a new maze, resize it to 10 rows by 5 columns and display the number of rows and columns, the following code can be used:

```csharp
var instance = MazeWasmInterop.GetInstance("/path/to/maze_wasm.wasm");
UInt32 mazeWasmPtr = CreateNewMazeWasm(0, 0);
interop.MazeWasmResize(mazeWasmPtr, 10, 5);
var rowCount = interop.MazeWasmGetRowCount(mazeWasmPtr);
var colCount = interop.MazeWasmGetColCount(mazeWasmPtr);
Console.WriteLine($"New dimensions = {rowCount} rows x {colCount} columns");
interop.FreeMazeWasm(mazeWasmPtr);
```

Notice that this code uses `FreeMazeWasm()` to release the `MazeWasm` pointer after it is no longer needed. If this is not done, a memory leak will occur within the `maze_wasm` Web Assembly.

## Getting Started

### Setup
To setup the build environment, run the following from the `Maze.Wasm.Interop` directory:

```
dotnet restore
```

### Build
To build the `Maze.Wasm.Interop` assembly, run the following from the `Maze.Wasm.Interop` directory:


```
dotnet build
```

### Testing
Testing can be performed via the [`Maze.Wasm.Interop.Tests`](../Maze.Wasm.Interop.Tests/README.md) project.

### Generating Documentation
To generate and view `HTML` documentation for the assembly, run the following from the `Maze.Wasm.Interop` directory:
```
docfx docfx.json --serve
```

This will generate the help which can then be accessed in a browser at http://localhost:8000
