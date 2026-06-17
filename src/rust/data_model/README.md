# `data_model` Crate

## Introduction

The `data_model` crate is written in `Rust` and defines the following data model objects:

- `CellEntity` - one entity occupying a cell, with its (optional) override characteristics (an `EnemyOverride`, `HealthOverride`, `KeyOverride`, `DoorOverride`, `WallOverride` or `TreasureOverride`); a cell holds a list of these, capped at one for now
- `DoorOverride` - non-default characteristics for a door cell (`door_style`)
- `DoorStyle` - visual open-animation rig for a door cell (`swing` / `slide` / `portcullis` / `dissolve`)
- `EnemyOverride` - non-default characteristics for an enemy cell (`enemy_type`, `damage`, `move_period_ms`)
- `EnemyType` - visual rig for an enemy cell (`goblin` / `ghost`)
- `Error` - represents a data model error
- `HealthOverride` - non-default characteristics for a health-pickup cell (`health_style`, `heal_amount`)
- `HealthStyle` - visual rig for a health-pickup cell (`heart` / `potion`)
- `KeyHolderStyle` - visual rig for a key-holder cell (`pedestal` / `chest` / `floating_key`)
- `KeyOverride` - non-default characteristics for a key-holder cell (`key_holder`)
- `Maze` - represents a maze
- `MazeCellState` - represents an individual maze cell state
- `MazeDefinition` - represents a maze definition (a character grid plus an optional sparse map of per-cell overrides)
- `MazePoint` - represents a point within a maze
- `OAuthIdentity` - represents a link between a user and an external OAuth provider
- `TreasureOverride` - non-default characteristics for a treasure cell (`style`, `rarity`, `value`)
- `TreasureRarity` - generation-frequency tier for a treasure cell, also the source of the default reward value (`common` / `uncommon` / `rare`)
- `TreasureStyle` - visual style for a treasure cell (`silver` / `gold` / `diamonds` / `jewels`)
- `User` - represents a user (with one or more associated `UserEmail`s)
- `UserEmail` - represents an email address attached to a user, with primary and verification flags
- `UserLogin` - represents a user login
- `WallOverride` - non-default characteristics for a wall cell (`wall_type`)
- `WallType` - visual type for a wall cell (`brick` / `dressed_stone` / `wood` / `cobblestone` solid textures, or `water` / `lava` / `iron_fence`); shares its vocabulary with the per-maze `wall_type` launch setting

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

However, some WebAssembly targets (like `wasm32-unknown-unknown` used by Wasmtime or .NET) do not support randomness or `Utc::now()` by default. To handle this cleanly, this crate supports a feature flag `wasm-lite` to conditionally disable that functionality and return a `nil` value instead.

Internally, UUIDs are generated like this:

```rust
fn generate_uuid() -> uuid::Uuid {
    #[cfg(not(feature = "wasm-lite"))]
    {
        uuid::Uuid::new_v4()
    }

    #[cfg(feature = "wasm-lite")]
    {
        uuid::Uuid::nil()
    }
}
```

This ensures:
- ✅ Random UUIDs are used by default in supported environments
- ✅ A deterministic fallback (`Uuid::nil()`) is used when randomness is explicitly disabled