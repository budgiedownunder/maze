# `Maze.Api` Assembly

## Introduction

The `Maze.Api` .NET assembly is written in `C#` and provides clean .NET classes for use by client applications. It wraps the Web Assembly interop functionality provided by [`Maze.Interop`](../Maze.Interop/README.md), allowing developers to be able to work with `Maze` and `MazeGame` objects without needing to be aware of the interop occuring beneath.

The assembly supports:

- Creating and editing mazes (cells, walls, start/finish positions, row/column insertion and deletion)
- Generating mazes automatically via `Maze.Generate()` with configurable dimensions, seed, start/finish positions, minimum spine length, and generation algorithm
- Solving mazes via `Maze.Solve()`
- Serialising and deserialising mazes to/from JSON
- Managing maze game sessions with `MazeGame` objects 

Use `Maze` to create, generate, solve, and serialise mazes:

```csharp
using (Maze maze = new Maze(10, 5)) {
    Console.WriteLine($"Maze contains: {maze.RowCount} row(s) x {maze.ColCount} column(s)");
}

using (Maze maze = Maze.Generate(new Maze.GenerationOptions { RowCount = 11, ColCount = 11, Seed = 42 })) {
    Console.WriteLine($"Generated: {maze.RowCount} row(s) x {maze.ColCount} column(s)");
}
```

Use `MazeGame` to run an interactive game session against a maze definition, tracking player position, direction, and visited cells:

```csharp
string definitionJson = /* {"grid":[...]} portion of a maze */;

using (MazeGame game = MazeGame.Create(definitionJson)) {
    Console.WriteLine($"Start: row {game.PlayerRow}, col {game.PlayerCol}");

    MazeGameMoveResult result = game.MovePlayer(MazeGameDirection.Right);
    Console.WriteLine($"Move right: {result}");  // Moved, Blocked, or Complete

    Console.WriteLine($"Visited {game.VisitedCells.Count} cell(s), complete: {game.IsComplete}");
}
```

## Getting Started

### Setup
To set up the build environment, run the following from the `Maze.Api` directory:

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
