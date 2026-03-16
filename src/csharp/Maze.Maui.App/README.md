# `Maze.Maui.App` Application

## Introduction

The `Maze.Maui.App` .NET application is a work-in-progress [MAUI](https://dotnet.microsoft.com/en-us/apps/maui) application.

At the moment, it allows the user to:

- Sign up for a new account
- Sign in and out of their account
- View their account profile and delete their account
- Load, edit, delete, rename and duplicate mazes
- Construct mazes containing start, finish and wall cells
- Attempt to solve mazes using the [`Maze.Api`](../Maze.Api/README.md) .NET assembly

It  has been tested on `Windows` desktop and `Android`/`iOS` devices. The screenshots below show it running on `Windows` desktop:  

| Design      | Solved 
|-------------|--------
|<img src = "Screenshots/windows-design.png" width="250"> | <img src = "Screenshots/windows-solved.png" width="250">

and these screenshots show it running on `Android` and `iOS` mobile devices:

| Android     | iOS 
|-------------|-------------
|<img src = "Screenshots/android-solved.png" width="250"> | <img src = "Screenshots/ios-solved.png" width="250">

This short animation clip demonstrates it being used on `Windows` to solve a more complex maze before introducing a blocking wall that then forces the solver to go via a different route on the next solve attempt:

<img src="Gifs/windows/solve-demo.gif" height="600">

and this one shows the design and solve processes being performed on an `iOS` device using extended (animated) selection:

<img src="Gifs/ios/solve-demo.gif" height="400">

## Navigation

The app uses a two-area navigation model:

- **Login area** (tab bar hidden): shown on first launch and after sign-out. Provides Sign In and Create Account actions.
- **Main area** (three tabs): shown after a successful sign-in.
  - **Mazes** — load, edit, delete, rename, duplicate and solve mazes
  - **Account** — view your profile, sign out, or delete your account
  - **About** — app information

If a bearer token from a previous session is stored, the app skips the Login page automatically on restart.

## Configuration

The app reads its settings from `Resources/Raw/appsettings.json` at startup:

| Setting | Type | Default | Description |
|:--------|:-----|:--------|:------------|
| `ApiRootUri` | Text | See below | Root URI of the `maze_web_server` REST API |
| `DisableStrictTLSCertificateValidation` | Boolean | `true` | Disables strict TLS certificate validation — set to `false` in production |

If `ApiRootUri` is not set, the app falls back to a platform-specific development default:

| Platform | Default `ApiRootUri` |
|:---------|:---------------------|
| Windows  | `https://localhost:8443/api/v1/` |
| iOS      | `https://localhost:8443/api/v1/` |
| Android  | `https://10.0.2.2:8443/api/v1/` (host machine loopback from the emulator) |

To point at a different server, add `ApiRootUri` to `appsettings.json`:

```json
{
  "ApiRootUri": "https://your-server/api/v1/"
}
```

## Keyboard Support
In addition to mouse/pointer support on the desktop, the following keyboard shortcuts are supported for selecting/editing cells and cell navigation:

**Editing:**

| Shortcut    | Description  |
|:------------|:-------------|
| `F`         | Set `Finish` |
| `S`         | Set `Start`  |
| `W`         | Set `Wall`   |
| `Delete`      | Clear selection   |

**Navigation and Selection:**

| Shortcut        | Description       
|:----------------|:------------------
| `Shift`         | Extend selection  
| `↓`             | Move down
| `←`             | Move left
| `→`             | Move right
| `↑`             | Move upwards
| `End`           | Jump to end of row
| `Home`          | Jump to start of row
| `Ctrl`+ `←`     | Jump to start column 
| `Ctrl`+ `→`     | Jump to end column 
| `Ctrl`+ `↑`     | Jump to top row 
| `Ctrl`+ `↓`     | Jump to bottom row 
| `Ctrl`+ `End`   | Jump to last cell 
| `Ctrl`+ `Home`  | Jump to first cell 

## Getting Started

### Setup
To setup the build environment, run the following from the `Maze.Maui.App` directory:

```
dotnet restore
```

### Build
To build the `Maze.Maui.App` application, run the following from the `Maze.Maui.App` directory:

```
dotnet build
```

### Publishing

To publish the `Maze.Maui.App` to your local machine, run the following from the `Maze.Maui.App` directory:

Windows:

```
publish-release-windows.bat
```

This should build and register the application with `Windows`. You should then be able to locate  the `Maze Maui App` in the Windows Apps list and launch it.

### Testing
Automated testing is not implemented yet

