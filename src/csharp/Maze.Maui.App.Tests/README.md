# `Maze.Maui.App.Tests` Assembly

## Introduction

The `Maze.Maui.App.Tests` .NET assembly incorporates unit tests for the [`Maze.Maui.App`](../Maze.Maui.App/README.md) MAUI application's ViewModels.

The project deliberately targets plain `net10.0` rather than a MAUI TFM. ViewModel source files from `Maze.Maui.App` are pulled in directly via `<Compile Include="..." Link="..." />` references in the `.csproj` rather than through an assembly reference. This avoids the WindowsAppSDK auto-initializer (which throws inside an unpackaged xUnit test host) while still exercising the real ViewModel code.

The trade-off: any ViewModel under test must remain free of MAUI runtime types (`Shell.Current`, `Application.Current`, etc.). Where a ViewModel needs MAUI runtime behaviour, it is abstracted behind an interface (e.g. `INavigationService`) that the tests mock with [Moq](https://github.com/devlooped/moq).

Tests currently cover:

| Class | What it tests |
|:------|:--------------|
| `AccountViewModelTests` | Profile load/save, dirty-tracking, password message handler, delete-account confirmation |
| `ChangePasswordViewModelTests` | Password Change/Set branching, messenger send, back-navigation |
| `EmailAddressesViewModelTests` | Email list management — add, set primary, remove, primary-row guards |
| `LoginViewModelTests` | Credentials sign-in (validation, 401, navigation), OAuth flow, session restore, OAuth provider sync |
| `MazesViewModelTests` | Maze list load + sort, name uniqueness, add/remove/invalidate, rename / duplicate / delete dialog flows, message handlers, navigation routing |
| `MazeViewModelTests` | SaveMaze new-vs-existing branches, RefreshMaze confirmation, dirty/CanSave/CanRefresh transitions, command-to-event routing, IsTouchOnlyDevice delegation |
| `SignUpViewModelTests` | CanSignUp guards, password match + complexity rules, 409 conflict, error-clearing partials, OAuth refresh |

When adding a new ViewModel test, also add the corresponding source-file link entry to `Maze.Maui.App.Tests.csproj`.

## Getting Started

### Setup
To set up the build environment, run the following from the `Maze.Maui.App.Tests` directory:

```
dotnet restore
```

### Build
To build the `Maze.Maui.App.Tests` assembly, run the following from the `Maze.Maui.App.Tests` directory:

```
dotnet build
```

### Testing
To run the tests, run the following command from the `Maze.Maui.App.Tests` directory:

```
dotnet test
```

### Linting
To verify code formatting and analyzer rules, run the following from the `Maze.Maui.App.Tests` directory:

```
dotnet format --verify-no-changes --severity info
```

To autofix violations, run:

```
dotnet format --severity info
```

The expected output is zero errors and zero warnings.
