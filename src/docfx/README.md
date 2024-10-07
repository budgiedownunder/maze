# `docfx` Project

## Introduction
The `docfx` project is responsible for the generation of HTML documentation for the `.NET` assemblies and `Rust` crates. It does this by:

1. First generating the content for the `.NET` assemblies in the `_site` sub-directory
2. Then, generating the `Rust` documentation in the temporary sub-directory `rust-doc-tmp` using `cargo`   
3. Finally, copying the `Rust` documentation files from `rust-doc-tmp` into  `_site/api/rust`

Note: 

The `Rust` documentation pages are displayed in `iFrame` containers within the main documentation. This allows the [`DocFX`](https://dotnet.github.io/docfx/) navigation bar and table of contents to be made consistently available for all `.NET` and `Rust` API help topics.

## Setup
The project uses [`DocFX`](https://dotnet.github.io/docfx/) to generate the help content. This can be installed by running the following command:
```
dotnet tool install -g docfx
```

## Build
To generate the HTML documentation, run the following from the `docfx` directory:

Windows:
```
build_all.bat
```

Linux/macOS:
```
sh build_all.sh
```

This will generate the documentation under the `_site` sub-directory.

## Viewing
To view `HTML` documentation after building, run the following from the `docfx` directory:

Windows:
```
serve.bat
```

Linux/macOS:
```
sh serve.sh
```

The documentation can then be accessed in a browser at http://localhost:8080
