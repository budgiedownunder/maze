# `docfx` Project

## Introduction
The `docfx` project is responsible for the generation of HTML documentation for the `.NET` assemblies and `Rust` crates. It does this by:

1. First generating the content for the `.NET` assemblies in the `_site` sub-directory
2. Next generating the `Rust` documentation in the temporary sub-directory `rust-doc-tmp` using `cargo`   
3. Copying the `Rust` documentation files from `rust-doc-tmp` into  `_site/api/rust`
4. The `Rust` documentation pages are then presented in iFrame containers within the main documentation 

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
sh copy_files.sh
docfx docfx.json
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
sh copy_files.sh
docfx docfx.json --serve
```

The documentation can then be accessed in a browser at http://localhost:8080
