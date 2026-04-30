//! Shared helpers for the integration-test crates in `tests/`.
//!
//! Each `tests/*.rs` file is compiled as its own crate by Cargo, so a true
//! `pub` module exported from the lib would clutter the public API. Putting
//! the shared code in `tests/common/mod.rs` is the standard Rust convention:
//! every test file does `mod common;` and accesses `common::store_contract::*`.

pub mod store_contract;
