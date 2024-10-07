# Introduction

The `maze-project` is an experimental project that has been created for exploring various programming languages, technologies and language-to-language integration. At its core, it contains a set of tools and libraries for managing and solving mazes that are then utilised in various application scenarios.

The following components are present:

| Language | Folder | Component | Description |
|----------|--------|-----------|--------------- |
| C#       | `src/csharp` | [`Maze.Wasm.Interop`](../api/net/Maze.Wasm.Interop.html) | .NET interop to `maze_wasm` web assembly |
|          |            | [`Maze.Wasm.Interop.Tests`](../api/net/Maze.Wasm.Interop.Tests.html) | .NET test library for [`Maze.Wasm.Interop`](../api/net/Maze.Wasm.Interop.html) |
| Rust     | `src/rust` | [`maze`](../api/rust/maze/view_content.md) | maze definition and calculation library |
|          |            | [`maze_console`](../api/rust/maze_console/view_content.md) | maze console application |
|          |            | [`maze_wasm`](../api/rust/maze_wasm/view_content.md) | maze web assembly library |
|          |            | [`storage`](../api/rust/storage/view_content.md) | maze storage library |
|          |            | [`utils`](../api/rust/utils/view_content.md) | utilities library |

