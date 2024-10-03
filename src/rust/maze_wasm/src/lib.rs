pub mod wasm_common;

// Export 'wasm' module if not in wasm-bindgen mode
#[cfg(not(feature = "wasm-bindgen"))]
pub mod wasm;

// Export 'wasm_bindgen' module if in wasm-bindgen mode
#[cfg(feature = "wasm-bindgen")]
pub mod wasm_bindgen;