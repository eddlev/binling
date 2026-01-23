use binling_core::capsules::{Capsule, CapsuleHeader, SquareSpace};
use binling_core::vm::LatticeVM;
use wasm_bindgen::prelude::*;

// This attribute makes the function run when the WASM module loads
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This allows Rust panic messages to appear in the browser console
    console_error_panic_hook::set_once();
    Ok(())
}

// This struct will be exported to JavaScript class "WebLattice"
#[wasm_bindgen]
pub struct WebLattice {
    vm: LatticeVM,
}

#[wasm_bindgen]
impl WebLattice {
    // Constructor: JS calls "new WebLattice()"
    #[wasm_bindgen(constructor)]
    pub fn new() -> WebLattice {
        // --- FIX IS HERE ---
        // We now provide a default name for the browser instance
        let mut vm = LatticeVM::new("web-local".to_string());
        // -------------------

        // Let's inject a "Genesis Capsule" just like in the CLI
        let genesis = Capsule {
            header: CapsuleHeader {
                magic: *b"BLE1",
                version_major: 0,
                version_minor: 1,
                flags: 0,
                capsule_id: 777,
                ss_n: SquareSpace::SS64,
                priority: 10,
                coord_x: 0,
                coord_y: 0,
                coord_z: 0,
                header_len: 122,
                policy_len: 0,
                payload_len: 10,
                pad_len: 0,
                dict_hash: [0; 32],
                policy_core_hash: [0; 32],
                capsule_hash: [0; 32],
            },
            policy_core: vec![],
            payload: vec![7u8; 4096], // 4096 SPAWN commands
        };

        vm.activate(genesis);

        WebLattice { vm }
    }

    pub fn tick(&mut self) -> String {
        if !self.vm.is_void() {
            self.vm.next_cycle();
        }

        // We return a JSON string to JS (Simple serialization)
        // Note: For high performance, we would use shared memory, but this is fine for v1.
        let cell_data: Vec<(i32, i32, i32, u8)> = self
            .vm
            .next_queue
            .iter()
            .map(|c| {
                (
                    c.header.coord_x as i32,
                    c.header.coord_y as i32,
                    c.header.coord_z as i32,
                    c.header.flags as u8,
                )
            })
            .collect();

        serde_json::to_string(&cell_data).unwrap_or("[]".to_string())
    }

    pub fn get_cycle(&self) -> u64 {
        self.vm.cycle_count
    }

    pub fn get_count(&self) -> usize {
        self.vm.next_queue.len()
    }
}
