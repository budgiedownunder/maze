# maze-project

## Table of Contents
- [Introduction](#introduction)
- [Getting Started](#getting-started)
- [Documentation](docs/README.md)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgements](#acknowledgements)

## Introduction
This is an experimental project that has been created for exploring various programming languages and technologies. At its core, it contains a set of tools and libraries for managing and solving mazes that are then utilised in various application scenarios. 

## Getting Started

### Build
To build all (currently `Rust`) project components:
```
cd src/rust
cargo build
```
### Run
To run the `maze_cli` application:
```
cd src/rust/maze_cli
cargo run
```

### Testing
To test all (currently `Rust`) components:
```
cd src/rust
cargo test
```
or, if you also wish to capture and display any `stdout` output:
```
cd src/rust
cargo test -- --nocapture
```
### Generating Documentation
To generate and view Rust (crate) documentation in your default browser:
```
cd src/rust
cargo doc --open
```

## Contributing
At this stage, this project is not accepting contributions.

## License

## Acknowledgements