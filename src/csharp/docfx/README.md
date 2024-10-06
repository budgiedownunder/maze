# `docfx` Project

## Introduction
The `docfx` project defines and manages the online documentation for the `.NET` components.

## Setup
The project uses [`DocFX`](https://dotnet.github.io/docfx/) to generate the help content. This can be installed by running the following command:
```
dotnet tool install -g docfx
```

## Build
To generate the documentation, run the following from the `docfx` directory:

Windows:
```
copy_files.bat
docfx docfx.json
```

Linux/macOS:
```
sh copy_files.sh
docfx docfx.json
```

This will generate the documentation under the `_site` sub-directory.

## Viewing
To generate and view `HTML` documentation, run the following from the `docfx` directory:

Windows:
```
copy_files.bat
docfx docfx.json --serve
```

Linux/macOS:
```
sh copy_files.sh
docfx docfx.json --serve
```

Once the build is complete, the documentation can then be accessed in a browser at http://localhost:8000
