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

    // 2. Create a Dummy Capsule
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
        payload: vec![],
    };

    // 3. Test the Codec (Serialization)
    println!("> Testing Codec (Encoding Capsule)...");
    match LatticeCodec::encode(&dummy) {
        Ok(bytes) => {
            println!("  [OK] Encoded size: {} bytes", bytes.len());
            // ASCII 66=B, 76=L, 69=E, 49=1
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

    // 5. Step the Cycle
    println!("> Stepping Cycle...");
    vm.next_cycle();

    // 6. Verify State
    println!("=== Cycle {} Summary ===", vm.cycle_count);
    println!("Active Queue: {} capsule(s)", vm.active_queue.len());

    if let Some(cap) = vm.active_queue.first() {
        println!("> Executing Capsule ID: {}", cap.header.capsule_id);
    }
}
