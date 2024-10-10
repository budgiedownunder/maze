# `MazeMauiApp` Application

## Introduction

The `MazeMauiApp` .NET application is a work-in-progress [MAUI](https://dotnet.microsoft.com/en-us/apps/maui) application. At the moment, 
it simply connects to the [`Maze.Api`](../Maze.Api/README.md) .NET assembly to prove that the connectivity through to the underlying Web Assembly
function. The plan is to develop it into a fully fledged application for defining and solving mazes.  

## Getting Started

### Setup
To setup the build environment, run the following from the `MazeMauiApp` directory:

```
dotnet restore
```

### Build
To build the `MazeMauiApp` application, run the following from the `MazeMauiApp` directory:


```
dotnet build
```

### Testing
Automated testing is not implemented yet