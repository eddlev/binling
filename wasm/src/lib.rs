use binling_core::capsules::{Capsule, CapsuleHeader, SquareSpace};
use binling_core::codec::LatticeCodec;
use wasm_bindgen::prelude::*;

// 1. Expose the Version to JS
#[wasm_bindgen]
pub fn lib_version() -> String {
    binling_core::version().to_string()
}

// 2. A JS-friendly wrapper to create a Test Capsule and get its bytes
// (In a real app, we would pass arguments for x, y, z, etc.)
#[wasm_bindgen]
pub fn js_create_dummy_capsule(id: u32) -> Vec<u8> {
    // Construct the Rust Struct
    let cap = Capsule {
        header: CapsuleHeader {
            magic: *b"BLE1",
            version_major: 0,
            version_minor: 1,
            flags: 0,
            ss_n: SquareSpace::SS64,
            priority: 1,
            header_len: 0,
            policy_len: 0,
            payload_len: 0,
            pad_len: 0,
            coord_x: 10,
            coord_y: 20,
            coord_z: 30,
            capsule_id: id,
            dict_hash: [0; 32],
            policy_core_hash: [0; 32],
            capsule_hash: [0; 32],
        },
        policy_core: vec![],
        payload: vec![0xAA, 0xBB, 0xCC], // Dummy payload
    };

    // Encode to Bytes (WASM will return this as a Uint8Array to JS)
    LatticeCodec::encode(&cap).unwrap_or(vec![])
}

// 3. A JS-friendly wrapper to Decode bytes and check the ID
#[wasm_bindgen]
pub fn js_inspect_capsule_id(data: &[u8]) -> i32 {
    match LatticeCodec::decode(data) {
        Ok(cap) => cap.header.capsule_id as i32,
        Err(_) => -1, // Error Code
    }
}
