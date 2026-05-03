# `Maze.Maui.Services` Assembly

## Introduction

The `Maze.Maui.Services` .NET assembly is written in `C#` and provides custom service functionality to insulate the calling code from platform specifics, such as determining the current device capabilities.

## Getting Started

### Setup
To setup the build environment, run the following from the `Maze.Maui.Services` directory:

```
dotnet restore
```

### Build
To build the `Maze.Maui.Services` assembly, run the following from the `Maze.Maui.Services` directory:

```
dotnet build
```

### Testing
Automated testing is not implemented yet

### Linting
To verify code formatting and analyzer rules, run the following from the `Maze.Maui.Services` directory:

```
dotnet format --verify-no-changes --severity info
```

To autofix violations, run:

```
dotnet format --severity info
```

The expected output is zero errors and zero warnings.
