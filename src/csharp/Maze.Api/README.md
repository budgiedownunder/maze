# `Maze.Api` Assembly

## Introduction

The `Maze.Api` .NET assembly is written in `C#` and provides clean .NET classes for use by client applications. It wraps the Web Assembly interop functionality provided by [`Maze.Wasm.Interop`](../Maze.Wasm.Interop/README.md), allowing developers to be able to work with `Maze` objects without needing to be aware of the interop occuring beneath.   

```csharp
using (Maze maze = new Maze(10, 5)) {
    Console.WriteLine($"Maze contains: {maze.RowCount} row(s) x {maze.ColCount} column(s)");
}
```

## Getting Started

### Setup
To setup the build environment, run the following from the `Maze.Api` directory:

```
dotnet restore
```

### Build
To build the `Maze.Api` assembly, run the following from the `Maze.Api` directory:


```
dotnet build
```

### Testing
Testing can be performed via the [`Maze.Api.Tests`](../Maze.Api.Tests/README.md) assembly.
