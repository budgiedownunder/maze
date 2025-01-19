# `maze_openapi_generator` Crate

## Introduction

The `maze_openapi_generator` crate is written in `Rust` and is a console application that generates the `openapi.json` file that is used for generating static help content for the `Maze Web API` documentation.


## Getting Started

### Build
To build the `maze_openapi_generator` application, run the following from within the `maze_openapi_generator` directory:
```
cargo build
```

### Run
To run the `maze_openapi_generator` application, run the following from within the `maze_openapi_generator` directory:
```
cargo run
```
If successful, this will generate the `openapi.json` file in the working directory with output similar to the following:

```
Running Maze API OpenAPI generator...
Maze API OpenAPI specification sucessfully generated as file:
/path/to/working/directory/openapi.json
```

### Checking for Errors/Warnings in `openapi.json`
To check for errors or warnings in the generated `openapi.json` file, you need to ensure that the [Redocly](https://redocly.com/) CLI is installed. If it isn't, you can install it by running the following command:  

```
npm install -g @redocly/cli
```

Then run the following `lint` command from within the `maze_openapi_generator` directory:

```
redocly lint openapi.json --config ../../docfx/redocly.yaml
```

If there are no errors or warnings, the output should appear similar to the following:
```
validating openapi.json...
openapi.json: validated in 7ms

Woohoo! Your API description is valid. 🎉
```
You should always ensure that any changes you make to the endpoint definitions in `maze_web_server` do not introduce any errors or warnings in their associated `OpenAPI` specifications. 

### Testing
To test the `maze_openapi_generator` crate, run the following from within the `maze_openapi_generator` directory:
```
cargo test
```

### Benchmarking
No benchmarking tests are currently implemented for the crate

### Generating Documentation
To generate and view `Rust` documentation for the crate in your default browser, run the following from within the `maze_openapi_generator` directory:
```
cargo doc --open
```