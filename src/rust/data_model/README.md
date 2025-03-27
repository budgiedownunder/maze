# `data_model` Crate

## Introduction

The `data_model` crate is written in `Rust` and defines the following data model objects:

- `Error` - represents a data model error
- `Maze` - represents a maze
- `MazeCellState` - represents an individual maze cell state
- `MazeDefinition` - represents a maze definition
- `MazePoint` - represents a point within a maze
- `User` - represents a user

## Getting Started

### Build
To build the `data_model` crate, run the following from within the `data_model` directory:
```
cargo build
```

### Testing
To test the `data_model` crate, run the following from within the `data_model` directory:
```
cargo test
```

### Benchmarking
No benchmarking tests are currently implemented for the crate

### Generating Documentation
To generate and view `Rust` documentation for the crate in your default browser, run the following from within the `data_model` directory:
```
cargo doc --open
```

### Handling UUID Generation

The crate uses [`uuid::Uuid::new_v4()`](https://docs.rs/uuid/latest/uuid/struct.Uuid.html#method.new_v4) to generate unique IDs where needed. This requires access to secure randomness via the [`getrandom`](https://docs.rs/getrandom) crate.

However, some WebAssembly targets (like `wasm32-unknown-unknown` used by Wasmtime or .NET) do not support randomness by default. To handle this cleanly, this crate supports a feature flag `uuid-disable-random` to conditionally disable UUID generation.

Internally, UUIDs are generated like this:

```rust
fn generate_uuid() -> uuid::Uuid {
    #[cfg(not(feature = "uuid-disable-random"))]
    {
        uuid::Uuid::new_v4()
    }

    #[cfg(feature = "uuid-disable-random")]
    {
        uuid::Uuid::nil()
    }
}
```

This ensures:
- ✅ Random UUIDs are used by default in supported environments
- ✅ A deterministic fallback (`Uuid::nil()`) is used when randomness is explicitly disabled