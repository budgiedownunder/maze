pub mod wasm_common;
#[cfg(not(feature = "wasm-bindgen"))]
pub mod wasm;

#[cfg(feature = "wasm-bindgen")]
pub mod wasm_bindgen;