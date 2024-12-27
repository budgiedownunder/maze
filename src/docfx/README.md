# `docfx` Project

## Introduction
The `docfx` project is responsible for the generation of HTML documentation for the `.NET`,  `JavaScript`, `Rust`, `Web` and `WebAssembly` APIs. It does this by:

1. Generating the content for the `.NET` assemblies in the `_site` sub-directory
2. Generating the `Rust` documentation in the temporary sub-directory `rust-doc-tmp` using `cargo`   
3. Copying the `Rust` documentation files from `rust-doc-tmp` into  `_site/api/rust`
4. Generating the `JavaScript` documentation in the temporary sub-directory `js-doc-tmp` using `cargo` and `wasm-bindgen`
5. Copying the `JavaScript` documentation files from `js-doc-tmp` into  `_site/api/js`
6. Generating the `OpenAPI` specification file `openapi.json` for the `maze_web_server` using the `maze_openapi_generator` console application
7. Running `redocly` with `openapi.json` to generate the `Maze REST API` documentation in the temporary sub-directory `web-doc-tmp/doc/maze_rest`
8. Copying the `Web API` documentation files from `web-doc-tmp` into  `_site/api/web`

Note: 

The `JavaScript API`, `Rust API`, `Web API` and `WebAssembly APi` documentation pages are displayed in `iFrame` containers within the main documentation. This allows the [`DocFX`](https://dotnet.github.io/docfx/) navigation bar and table of contents to be made consistently available for all API help topics.

## Setup
The project uses [`DocFX`](https://dotnet.github.io/docfx/) and [Redocly](https://redocly.com/) to generate the help content. These can be installed by running the following commands:
```
dotnet tool install -g docfx
npm install -g @redocly/cli
```

## Build
To generate the HTML documentation, run the following from the `docfx` directory:

Windows:
```
build_all.bat
```

This will generate the documentation under the `_site` sub-directory.

## Viewing
To view `HTML` documentation after building, run the following from the `docfx` directory:

Windows:
```
serve.bat
```

The documentation can then be accessed in a browser at http://localhost:8080
