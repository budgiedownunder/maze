# maze

[![Build & Test](https://github.com/budgiedownunder/maze/actions/workflows/build-and-test-components-multi-os.yml/badge.svg)](https://github.com/budgiedownunder/maze/actions/workflows/build-and-test-components-multi-os.yml)
[![Deploy Docs](https://github.com/budgiedownunder/maze/actions/workflows/build-and-deploy-to-github-pages.yml/badge.svg)](https://github.com/budgiedownunder/maze/actions/workflows/build-and-deploy-to-github-pages.yml)
[![Generate Diagrams](https://github.com/budgiedownunder/maze/actions/workflows/generate-png-from-puml.yml/badge.svg)](https://github.com/budgiedownunder/maze/actions/workflows/generate-png-from-puml.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![Documentation](https://img.shields.io/badge/docs-GitHub%20Pages-blue)](https://budgiedownunder.github.io/maze/)

A multi-language experimental project exploring **Rust**, **C# (.NET 10)**, **React.js**, **TypeScript**, **WebAssembly**, and the **Bevy** game engine. Built around maze generation, solving, and both 2D and **first-person 3D** gameplay — with collectible keys & doors, real-time chasing enemies, player health, health pickups, collectible treasure, scoring with leaderboards, and a browsable library of user-authored & curated 3D games (shareable, grouped into arcade & campaign collections, with daily challenges) — it demonstrates library crates, REST APIs, WASM bindings, OpenAPI, a cross-platform MAUI app, a React.js SPA, Node.js-based API and E2E testing, architecture diagramming with PlantUML, documentation generation with DocFX, and automated CI/CD across Windows, macOS, and Linux.

<img src="./src/react/maze_web_server/screenshots/web-3d-intro.png" width="600">

## Table of Contents
- [Introduction](#introduction)
- [Overview](#overview)
- [Components](#components)
- [Architecture](#architecture)
- [Screenshots](#screenshots)
- [Getting Started](#getting-started)
- [Contributing](#contributing)
- [License](#license)

## Introduction

> **Status:** Experimental — this project exists to explore language interoperability and is actively maintained but not intended for production use.

This is an experimental project that has been created for exploring various programming languages, technologies and language-to-language integration. At its core, it contains a set of tools and libraries for managing and solving mazes that are then utilised in various application scenarios.

## Overview

The project spans a set of `Rust` library crates and the interop layers, applications, APIs and tooling built on top of them, grouped by area below.

### Core libraries & console

- A simple `Rust` console application ([`maze_console`](./src/rust/maze_console/README.md)) that leverages `Rust` library crates for maze management/generate/solve ([`maze`](./src/rust/maze/README.md)) and storage ([`storage`](./src/rust/storage/README.md))
- Automated unit and mock testing (dependency injection)
- Pluggable storage backends in `Rust` ([`storage`](./src/rust/storage/README.md)) — a file-on-disk default plus a SQL-backed implementation supporting `SQLite`, `PostgreSQL`, and `MySQL` via a single portable schema (built on [`SQLx`](https://crates.io/crates/sqlx)'s `Any` driver with automatic migrations)
- Pluggable communications (e.g. email) via the [`comms`](./src/rust/comms/README.md) crate

### Interop & WebAssembly

- Web Assembly libraries (`wasm32` and `wasm-bindgen`) written in `Rust` for maze generation/calculation ([`maze_wasm`](./src/rust/maze_wasm/README.md)) and 3D game visualisation ([`maze_game_bevy_wasm`](./src/rust/maze_game_bevy_wasm/README.md))
- `.NET` to Web Assembly ([`maze_wasm`](./src/rust/maze_wasm/README.md)) interop library ([`Maze.Interop`](./src/csharp/Maze.Interop/README.md)) in `C#` that supports [Wasmtime](https://docs.wasmtime.dev/) (for `Windows` and `Android`), [Wasmer](https://wasmer.io/) (for `Android` and `iOS` simulator), and a native C library via [`maze_c`](./src/rust/maze_c/README.md) (for `iOS` simulator and physical device)
- JavaScript API generation from `Rust` crates (`wasm-pack`)
- Automated JavaScript API testing in `node.js` (`chai`, `mocha`)
- Automated `.NET` API testing with `xUnit` ([`Maze.Interop.Tests`](./src/csharp/Maze.Interop.Tests/README.md))

### Web server & REST API

- `Rust` web server application ([`maze_web_server`](./src/rust/maze_web_server/README.md)):
  - Leverages the `Rust` library crates for calculation/gameplay ([`maze`](./src/rust/maze/README.md)) and storage ([`storage`](./src/rust/storage/README.md)), with a choice of file-on-disk or SQL-backed persistence (selected at runtime) and exposes them as a `REST`ful Web API
  - Uses [`actix`](https://actix.rs/) to serve the `HTTPS` API and [`utoipa`](https://docs.rs/utoipa/latest/utoipa/) to publish it as an [`OpenAPI`](https://www.openapis.org/)-compliant interface for use in third party products such as [`Swagger`](https://swagger.io/)
  - Supports interactive documentation in the form of [RapiDoc](https://github.com/rapi-doc/RapiDoc), [Redoc](https://redocly.com/redoc) and [Swagger UI](https://swagger.io/tools/swagger-ui/)
  - Serves a React Single Page Application (SPA) from a configurable static directory
  - Supports OAuth / OIDC sign-in via a pluggable `OAuthConnector` (Google, GitHub and Facebook built-in)
  - Provides email-driven password-reset and email-verification flows, orchestrated through the [`comms`](./src/rust/comms/README.md) crate's pluggable email provider system (currently: Mailgun, SMTP+OAuth2 → Microsoft/GMail) + logging

### Applications (web & mobile)

- `React`/`TypeScript` web frontend ([`maze_web_server`](./src/react/maze_web_server/README.md)) — a browser-based UI for the `maze_web_server` REST API:
  - Maze management — create, edit, generate, solve, and walk-solution animation
  - 2D gameplay (in-browser via the [`maze_wasm`](./src/rust/maze_wasm/README.md) WebAssembly module) — auto-collected keys & doors, real-time enemies, player HP with health pickups, collectible treasure, leaderboards, and a pause menu
  - 3D gameplay (in-browser via the [`maze_game_bevy_wasm`](./src/rust/maze_game_bevy_wasm/README.md) WebAssembly module) — the first-person Bevy 3D maze game, launched from the 3D-game library
  - A 3D-game workshop — author shareable 3D games (create, edit, reshuffle, duplicate, preview, add an image thumbnail), grouped into free-choice arcade or ordered campaign collections and browsed across Featured / My Games / Shared / Community, plus daily challenges that rotate their layout and leaderboard each day
  - Account & authentication — OAuth sign-in (Google, GitHub, Facebook) when enabled; self-management (sign-up, sign-in, edit profile, set a profile avatar, manage/verify email addresses, change/forgot/reset password, delete account)
  - Testing with [Vitest](https://vitest.dev/), [React Testing Library](https://testing-library.com/), [Mock Service Worker](https://mswjs.io/), and [Playwright](https://playwright.dev/)
- `C#` [MAUI](https://dotnet.microsoft.com/en-us/apps/maui) application ([`Maze.Maui.App`](./src/csharp/Maze.Maui.App/README.md)) — built on the Web Assembly interop library ([`Maze.Interop`](./src/csharp/Maze.Interop/README.md)) via a wrapper API ([`Maze.Api`](./src/csharp/Maze.Api/README.md)):
  - Maze management and 2D gameplay — create, save, delete, rename, edit, generate, solve, walk solution, and play (keys & doors, real-time enemies, player HP + health pickups, collectible treasure, leaderboards, and a pause menu)
  - 3D-game library — browse and play 3D games (Featured / My Games / Shared / Community), daily challenges, and campaign collections
  - Account & authentication — OAuth sign-in (Google, GitHub, Facebook) when enabled; self-management (sign-up, sign-in, edit profile, set a profile avatar, manage/verify email addresses, change/forgot password, delete account)

### 3D game (Bevy)

- A first-person **3D maze game** in `Rust` using the [`Bevy`](https://bevyengine.org/) engine ([`maze_game_bevy`](./src/rust/maze_game_bevy/README.md)) — PBR rendering, procedural textures, wall decorations, minimap, camera tilt, gold-leaf rain on win, rain + lightning on lose, dead-end artifacts, auto-collected keys & doors, real-time chasing enemies, a heart-based HP HUD with health pickups, collectible treasure, multi-level stacked runs with ladder / portal level transitions, and touch support + D-pad for mobile
- A cross-platform WebAssembly build of the `Bevy` game ([`maze_game_bevy_wasm`](./src/rust/maze_game_bevy_wasm/README.md)) — served at `/game/` by the Rust web server, launched from the React SPA, and embedded in the MAUI app via a `WebView`

### Tooling, documentation & CI/CD

- Automated `Rust` documentation-generation with `cargo doc`
- Automated `C#` API documentation generation with `DocFX`
- Combined `C#` and `Rust` documentation in a single HTML help system with use of `iFrame` containers
- Architecture diagramming using `PlantUML` ([`architecture.puml`](./docs/diagrams/architecture.puml))
- Automated image generation workflows using GitHub Actions ([`generate-png-from-puml.yml`](./.github/workflows/generate-png-from-puml.yml))
- Automated build and testing workflows using GitHub Actions ([`build-and-test-components-multi-os.yml`](./.github/workflows/build-and-test-components-multi-os.yml))
- Automated GitHub Pages asset generation and deployment [`build-and-deploy-to-github-pages`](./.github/workflows/build-and-deploy-to-github-pages.yml)

## Components

The following components are present:

| Folder                         | Component                                                                     | Description
|--------------------------------|-------------------------------------------------------------------------------|---------------
| `.github/workflows`            | `*.yml`                                                                       | GitHub Action workflow files
| `docs`                         | [`README.md`](./docs/README.md)                                               | Project overview documentation
| `research/algorithms/excel`     | `maze-algorithms.xls`                                                         | Excel workbook containing maze algorithms
| `src`                          | [`docfx`](./src/docfx/README.md)                                              | HTML help generation
| `src/csharp`                   | [`Maze.Api`](./src/csharp/Maze.Api/README.md)                                 | .NET API that sits above  [`Maze.Interop`](./src/csharp/Maze.Interop/README.md)
|                                | [`Maze.Api.Tests`](./src/csharp/Maze.Api.Tests/README.md)                     | Unit tests for [`Maze.Api`](./src/csharp/Maze.Api/README.md)
|                                | [`Maze.Maui.App`](./src/csharp/Maze.Maui.App/README.md)                       | Maze [MAUI](https://dotnet.microsoft.com/en-us/apps/maui) application
|                                | [`Maze.Maui.Controls`](./src/csharp/Maze.Maui.Controls/README.md)             | Custom [MAUI](https://dotnet.microsoft.com/en-us/apps/maui) controls and definitions
|                                | [`Maze.Maui.Services`](./src/csharp/Maze.Maui.Services/README.md)             | Custom [MAUI](https://dotnet.microsoft.com/en-us/apps/maui) services
|                                | [`Maze.Interop`](./src/csharp/Maze.Interop/README.md)               | .NET interop to `maze_wasm` web assembly
|                                | [`Maze.Interop.Tests`](./src/csharp/Maze.Interop/README.md)         | .NET test library for [`Maze.Interop`](./src/csharp/Maze.Interop/README.md)
| `src/graphics`                 | [`graphics`](./src/graphics/README.md)                                        | Source graphic assets (sprites, animation frames)
| `src/react`                    | [`maze_web_server`](./src/react/maze_web_server/README.md)                    | React SPA frontend for Rust `maze_web_server`
| `src/rust`                     | [`auth`](./src/rust/auth/README.md)                                           | Authentication library
|                                | [`comms`](./src/rust/comms/README.md)                                         | Outbound communications library (Email)
|                                | [`data_model`](./src/rust/data_model/README.md)                               | Data model library
|                                | [`maze`](./src/rust/maze/README.md)                                           | Maze definition, calculation, and gameplay engine library
|                                | [`maze_c`](./src/rust/maze_c/README.md)                                       | Maze C API library
|                                | [`maze_console`](./src/rust/maze_console/README.md)                           | Maze console application
|                                | [`maze_game_bevy`](./src/rust/maze_game_bevy/README.md)                       | Bevy-based first-person 3D maze game (library + native binary)
|                                | [`maze_game_bevy_wasm`](./src/rust/maze_game_bevy_wasm/README.md)             | WASM wrapper of `maze_game_bevy` for browser deployment
|                                | [`maze_openapi_generator`](./src/rust/maze_openapi_generator/README.md)       | Maze OpenAPI generator console application
|                                | [`maze_wasm`](./src/rust/maze_wasm/README.md)                                 | Maze WebAssembly API library
|                                | [`maze_web_server`](./src/rust/maze_web_server/README.md)                     | Maze web server console application
|                                | [`storage`](./src/rust/storage/README.md)                                     | Maze storage library
|                                | [`utils`](./src/rust/utils/README.md)                                         | Utilities library

## Architecture

![Architecture Diagram](./docs/diagrams/architecture.png)

> See [`docs/diagrams/architecture.puml`](./docs/diagrams/architecture.puml) for the PlantUML source.

## Screenshots

### MAUI app (Windows, iOS, Android)

The Maze MAUI application running on Windows, iOS, and Android.

| | Windows | iOS | Android |
|---|---------|-----|---------|
| **Home Page** | <img src="./src/csharp/Maze.Maui.App/Screenshots/windows-home.png" width="250"> | <img src="./src/csharp/Maze.Maui.App/Screenshots/ios-home.png" width="250"> | <img src="./src/csharp/Maze.Maui.App/Screenshots/android-home.png" width="250"> |
| **Solved** | <img src="./src/csharp/Maze.Maui.App/Screenshots/windows-solved.png" width="250"> | <img src="./src/csharp/Maze.Maui.App/Screenshots/ios-solved.png" width="250"> | <img src="./src/csharp/Maze.Maui.App/Screenshots/android-solved.png" width="250"> |
| **Walk Solution** | <img src="./src/csharp/Maze.Maui.App/Screenshots/windows-walk.png" width="250"> | <img src="./src/csharp/Maze.Maui.App/Screenshots/ios-walk.png" width="250"> | <img src="./src/csharp/Maze.Maui.App/Screenshots/android-walk.png" width="250"> |
| **2D Game** | <img src="./src/csharp/Maze.Maui.App/Screenshots/windows-game.png" width="250"> | <img src="./src/csharp/Maze.Maui.App/Screenshots/ios-game.png" width="250"> | <img src="./src/csharp/Maze.Maui.App/Screenshots/android-game.png" width="250"> |
| **3D Games** (browse) | <img src="./src/csharp/Maze.Maui.App/Screenshots/windows-3d-games.png" width="250"> | <img src="./src/csharp/Maze.Maui.App/Screenshots/ios-3d-games.png" width="250"> | <img src="./src/csharp/Maze.Maui.App/Screenshots/android-3d-games.png" width="250"> |
| **3D Game** | <img src="./src/csharp/Maze.Maui.App/Screenshots/windows-3d-game.png" width="250"> | <img src="./src/csharp/Maze.Maui.App/Screenshots/ios-3d-game.png" width="250"> | <img src="./src/csharp/Maze.Maui.App/Screenshots/android-3d-game.png" width="250"> |

### Web UI

The React SPA running in a desktop browser.

**Home Page**

The home page, allowing the user to jump into today's daily challenge, browse and play 3D games, create their own 3D games and mazes, or view the leaderboards.

<img src="./src/react/maze_web_server/screenshots/web-home.png" width="600">

**Maze List**

The mazes list page, showing the user's mazes. 

<img src="./src/react/maze_web_server/screenshots/web-mazes.png" width="600">

**Maze Editor**

The maze editor, defining a health cell:

<img src="./src/react/maze_web_server/screenshots/web-editing.png" width="600">

The same maze solved:

<img src="./src/react/maze_web_server/screenshots/web-solved.png" width="600">

**Walk Solution**

The maze editor animating a step-by-step walk of the solution path.

<img src="./src/react/maze_web_server/screenshots/web-walk.gif" width="600">

**Maze Game**

Playing a maze — the player navigates using keyboard or D-pad, collecting keys to open doors and evading real-time enemies, with a heart-based HP HUD (health pickups restore HP) and visited cells marked as they are left.

<img src="./src/react/maze_web_server/screenshots/web-game.gif" width="600">

**3D Games**

Beyond their own mazes, players browse a library of first-person 3D games — **Featured**, **My Games**, **Shared** with them, and the wider **Community** — and can play any of them or view its leaderboard.

<img src="./src/react/maze_web_server/screenshots/web-3d-games.png" width="600">

<img src="./src/react/maze_web_server/screenshots/web-3d-games-featured.png" width="600">

They also author their own in the workshop: create, edit, reshuffle the layout, duplicate, preview, share, and give each an image thumbnail.

<img src="./src/react/maze_web_server/screenshots/web-3d-game-editor.png" width="600">

Games can be grouped into **collections**, played either as a free-choice **arcade** or an ordered **campaign** (levels unlock as you clear them):

<img src="./src/react/maze_web_server/screenshots/web-3d-game-collection.png" width="600">

**3D Maze Game**

Playing a maze in first-person 3D — chasing enemies, collectible keys & doors, and a heart-based HP HUD with health pickups — the Bevy engine runs entirely in-browser via WebAssembly.

<img src="./src/react/maze_web_server/screenshots/web-3d-game.gif" width="600">

A run can also span **multiple stacked levels**: each interim level's finish is a ladder or portal up to the next, carrying your score, health, and collected items forward, until the final level's gold finish orb completes the run. Upper levels can taper to smaller, see-through footprints so the stack reads as a tower from below.

A multi-level game at sunset, with lava as walls:

<img src="./src/react/maze_web_server/screenshots/web-3d-multilevel.png" width="600">

A night game, but this time with water as walls:

<img src="./src/react/maze_web_server/screenshots/web-3d-multilevel-2.png" width="600">

**Leaderboards**

Per-maze and per-3D-game leaderboards rank completed runs by fastest time or highest score, showing each player's name and highlighting your own runs. A board can be reset to empty by its owner (of the maze or the game) or by an administrator.

<img src="./src/react/maze_web_server/screenshots/web-leaderboards.png" width="600">

A **daily-challenge** game keeps a separate board for each day (UTC); a date picker jumps between the days that have runs:

<img src="./src/react/maze_web_server/screenshots/web-daily-challenge.png" width="600">

## API

The Rust [`maze_web_server`](./src/rust/maze_web_server/README.md#introduction) implements a rich, RESTful Web API supporting interactive documentation in the form of [RapiDoc](https://github.com/rapi-doc/RapiDoc), [Redoc](https://redocly.com/redoc) and [Swagger UI](https://swagger.io/tools/swagger-ui/)

<img src="./src/rust/maze_web_server/screenshots/api/swagger.png" width="600">

## Getting Started

### Setup
To setup the build and test environment, you first need to install:

- [`Rust`](https://www.rust-lang.org/tools/install) (latest stable)
- [`Node.js 24+`](https://nodejs.org/en/learn/getting-started/how-to-install-nodejs)
- [`.NET 10.0+`](https://dotnet.microsoft.com/en-us/download)

To setup the `Rust` build environment, refer to the [README](src/rust/README.md) in the `rust` directory.

To setup the `React` build environment, refer to the [README](src/react/README.md) in the `react` directory.

To setup the `C#` build environment, refer to the [README](src/csharp/README.md) in the `csharp` directory.

### Build

- To build the `Rust` crates, refer to the [README](src/rust/README.md) in the `rust` directory.

- To build the `React` web frontend, refer to the [README](src/react/README.md) in the `react` directory.

- To build the `C#` (`.NET`) APIs, refer to the [README](src/csharp/README.md) in the `csharp` directory.

### Generating Documentation
- To generate combined documentation for the `.NET` APIs and `Rust` crates, refer to the [README](src/docfx/README.md) in the `docfx` project.

- To generate documentation just for the `.NET` APIs, refer to the [README](src/csharp/README.md) in the `csharp` directory.

- To generate documentation just for the `Rust` crates, refer to the [README](src/rust/README.md) in the `rust` directory.

The combined output is deployed automatically to [GitHub Pages](https://budgiedownunder.github.io/maze/) on every push to `main`.

## Contributing
At this stage, this project is not accepting contributions.

## License
This software is licensed under the [MIT License](./LICENSE)
