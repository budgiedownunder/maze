# Introduction

The `maze-project` is an experimental project that has been created for exploring various programming languages, technologies and language-to-language integration. At its core, it contains a set of tools and libraries for managing and solving mazes that are then utilised in various application scenarios.

The following components are present:

| Language | Folder | Component | Description |
|----------|--------|-----------|--------------- |
| C#       | `src/csharp` | [`Maze.Wasm.Interop`](/api/net/Maze.Wasm.Interop.html) | .NET interop to `maze_wasm` web assembly |
|          |            | [`Maze.Wasm.Interop.Tests`](/api/net/Maze.Wasm.Interop.Tests.html) | .NET test library for [`Maze.Wasm.Interop`](/api/net/Maze.Wasm.Interop.html) |
| Rust     | `src/rust` | `maze` | maze definition and calculation library |
|          |            | `maze_console` | maze console application |
|          |            | `maze_wasm` | maze web assembly library |
|          |            | `storage` | maze storage library |
|          |            | `utils` | utilities library |

