# Introduction

The `maze-project` is an experimental project that has been created for exploring various programming languages, technologies and language-to-language integration. At its core, it contains a set of tools and libraries for managing and solving mazes that are then utilised in various application scenarios.

The following components are present:

| Language | Folder       | Component                                                  | Description   |
|----------|--------------|------------------------------------------------------------|---------------|
| C#       | `src/csharp` | [`Maze.Api`](xref:Maze.Api)                                | .NET API that sits above  [`Maze.Wasm.Interop`](xref:Maze.Wasm.Interop)
|          |              | [`Maze.Api.Tests`](xref:Maze.Api.Tests)                    | Unit tests for [`Maze.Api`](xref:Maze.Api)
|          |              | [`Maze.Wasm.Interop`](xref:Maze.Wasm.Interop)              | .NET interop to [`maze_wasm`](../api/rust/maze_wasm/view_content.md) web assembly
|          |              | [`Maze.Wasm.Interop.Tests`](xref:Maze.Wasm.Interop.Tests)  | .NET test library for [`Maze.Wasm.Interop`](xref:Maze.Wasm.Interop)
|          |              | [`MazerMauiApp`](xref:MazeMauiApp)                         | Maze [MAUI](https://dotnet.microsoft.com/en-us/apps/maui) application
| Rust     | `src/rust`   | [`maze`](../api/rust/maze/view_content.md)                 | Maze definition and calculation library
|          |              | [`maze_console`](../api/rust/maze_console/view_content.md) | Maze console application
|          |              | [`maze_wasm`](../api/rust/maze_wasm/view_content.md)       | Maze Web Assembly library
|          |              | [`storage`](../api/rust/storage/view_content.md)           | Maze storage library
|          |              | [`utils`](../api/rust/utils/view_content.md)               | Utilities library


