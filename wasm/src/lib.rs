use binling_core::capsules::{Capsule, CapsuleHeader, SquareSpace};
use binling_core::vm::LatticeVM;
use wasm_bindgen::prelude::*;

// This attribute makes the function run when the WASM module loads
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This allows Rust panic messages to appear in the browser console (very helpful!)
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
        let mut vm = LatticeVM::new();

        // Let's inject a "Genesis Capsule" just like in the CLI
        let genesis = Capsule {
            header: CapsuleHeader {
                magic: *b"BLE1",
                version_major: 0,
                version_minor: 1,
                flags: 0,
                ss_n: SquareSpace::SS64,
                priority: 10,
                header_len: 0,
                policy_len: 0,
                payload_len: 0,
                pad_len: 0,
                coord_x: 0,
                coord_y: 0,
                coord_z: 0,
                capsule_id: 1,
                dict_hash: [0; 32],
                policy_core_hash: [0; 32],
                capsule_hash: [0; 32],
            },
            policy_core: vec![],
            payload: vec![0x12, 0x30, 0xFF], // INC, SPAWN, HALT
        };

        vm.activate(genesis);

        WebLattice { vm }
    }

    // JS calls "lattice.tick()"
    pub fn tick(&mut self) {
        if !self.vm.is_void() {
            self.vm.next_cycle();
        }
    }

    // JS calls "lattice.get_status()" to get a string report
    pub fn get_status(&self) -> String {
        format!(
            "Cycle: {} | Active Cells: {} | Next Gen: {}",
            self.vm.cycle_count,
            self.vm.active_queue.len(),
            self.vm.next_queue.len()
        )
    }

    // JS calls "lattice.get_active_cell_count()"
    pub fn get_active_cell_count(&self) -> usize {
        self.vm.active_queue.len()
    }
}
