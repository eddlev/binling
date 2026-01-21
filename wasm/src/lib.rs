use wasm_bindgen::prelude::*;

// This attribute exposes the function to JavaScript
#[wasm_bindgen]
pub fn version() -> String {
    format!("BinLing WASM Adapter v{}", binling_core::version())
}

// Placeholder for future encoder/decoder exposure
#[wasm_bindgen]
pub fn init_vm() -> Result<String, JsValue> {
    Ok("Lattice VM Initialized (Placeholder)".to_string())
}
