use binling_core::capsules::{Capsule, CapsuleHeader, SquareSpace};
use binling_core::instructions::OpCode;
use binling_core::vm::LatticeVM;
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== BinLing PHYSICS DIAGNOSTIC ===");

    // 1. Create a fresh VM
    let mut vm = LatticeVM::new();
    println!(
        "> [INIT] VM Created. Active Queue: {}",
        vm.active_queue.len()
    );

    // 2. Create the Kernel manually (No network involved)
    println!("> [GENESIS] creating Kernel Capsule...");
    let payload = vec![7u8; 10]; // 10 SPAWN instructions

    let kernel = Capsule {
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
            payload_len: payload.len() as u32,
            pad_len: 0,
            dict_hash: [0; 32],
            policy_core_hash: [0; 32],
            capsule_hash: [0; 32],
        },
        policy_core: vec![],
        payload,
    };

    // 3. Inject
    vm.activate(kernel);
    println!(
        "> [INJECT] Kernel injected. Next Queue: {}",
        vm.next_queue.len()
    );

    // 4. Run 5 Cycles manually
    for i in 1..=6 {
        println!("\n--- CYCLE {} ---", i);

        // Move from Next -> Active
        if vm.is_void() {
            println!("!! [CRITICAL ERROR] VM is VOID (Empty). Simulation died.");
            break;
        }

        vm.next_cycle();

        println!("> Status: Active Cells: {}", vm.active_queue.len());
        println!("> Status: Next Queue:   {}", vm.next_queue.len());

        // Print positions of all cells
        for (idx, cap) in vm.next_queue.iter().enumerate() {
            println!(
                "  [{}] ID: {} | Pos: ({}, {}, {}) | DNA: {} bytes",
                idx,
                cap.header.capsule_id,
                cap.header.coord_x,
                cap.header.coord_y,
                cap.header.coord_z,
                cap.payload.len()
            );
        }

        thread::sleep(Duration::from_millis(500));
    }

    println!("\n=== DIAGNOSTIC COMPLETE ===");
}
