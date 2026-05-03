# `Maze.Maui.Controls` Assembly

## Introduction

The `Maze.Maui.Controls` .NET assembly is written in `C#` and provides custom MAUI control classes and definitions for:

- [Icon](./Pointer/Icon.cs)
- [InteractiveGrid](./InteractiveGrid/Grid.cs)
- [Keyboard](./Keyboard/Keyboard.cs)
- [Pointer](./Pointer/Pointer.cs)

## Getting Started

### Setup
To setup the build environment, run the following from the `Maze.Maui.Controls` directory:

```
dotnet restore
```

### Build
To build the `Maze.Maui.Controls` assembly, run the following from the `Maze.Maui.Controls` directory:

```
dotnet build
```

### Testing
Automated testing is not implemented yet

### Linting
To verify code formatting and analyzer rules, run the following from the `Maze.Maui.Controls` directory:

```
dotnet format --verify-no-changes --severity info
```

To autofix violations, run:

```
dotnet format --severity info
```

The expected output is zero errors and zero warnings.
