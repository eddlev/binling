use binling_core::capsules::{Capsule, CapsuleHeader, SquareSpace};
use binling_core::codec::LatticeCodec;
use binling_core::vm::LatticeVM;

fn main() {
    println!("=== BinLing CLI v{} ===", binling_core::version());
    println!("Initializing Levin Lattice VM...");

    // 1. Initialize the VM
    let mut vm = LatticeVM::new();
    println!(
        "> Cycle {}: VM Ready. Status: {}",
        vm.cycle_count,
        if vm.is_void() { "VOID" } else { "ACTIVE" }
    );

    // 2. Create a Dummy Capsule (The "Adam" Capsule)
    let dummy = Capsule {
        header: CapsuleHeader {
            magic: *b"BLE1",
            version_major: 0,
            version_minor: 1,
            flags: 0,
            ss_n: SquareSpace::SS64,
            priority: 10,
            header_len: 122,
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
        // GENETIC CODE:
        // 1. INC (R0 = 1)
        // 2. LOG (Print 1)
        // 3. SPAWN (Create a child)
        // 4. HALT (Mother dies)
        payload: vec![
            0x12, // INC
            0x20, // LOG
            0x30, // SPAWN
            0xFF, // HALT
        ],
    };

    // 3. Test the Codec (Serialization)
    println!("> Testing Codec (Encoding Capsule)...");
    match LatticeCodec::encode(&dummy) {
        Ok(bytes) => {
            println!("  [OK] Encoded size: {} bytes", bytes.len());
            println!(
                "  [OK] First 4 bytes: {:?} (Should be [66, 76, 69, 49])",
                &bytes[0..4]
            );
        }
        Err(e) => println!("  [ERR] Encoding failed: {}", e),
    }

    // 4. Activate the Capsule
    println!(
        "> Injecting Capsule ID {} into Next Queue...",
        dummy.header.capsule_id
    );
    vm.activate(dummy);

    // 5. Run the Simulation for 3 Cycles (The Loop of Life)
    println!("> Starting Life Simulation (3 Cycles)...");

    for _ in 0..3 {
        println!("\n--------------------------------");
        println!("> Stepping Cycle {}...", vm.cycle_count + 1);

        // Run the cycle
        vm.next_cycle();

        // Report
        println!("=== Cycle {} Summary ===", vm.cycle_count);
        println!("  Active Population: {} capsule(s)", vm.active_queue.len());
        println!("  Next Generation:   {} capsule(s)", vm.next_queue.len());

        // Print who is running
        if let Some(cap) = vm.active_queue.first() {
            println!("  [Trace] Currently Running: Cap {}", cap.header.capsule_id);
        }
    } // End of For Loop
}
