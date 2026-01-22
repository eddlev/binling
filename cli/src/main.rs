use binling_core::capsules::{Capsule, CapsuleHeader, SquareSpace};
use binling_core::codec::LatticeCodec;
use binling_core::vm::LatticeVM;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener; // <--- NEW: Networking tools

#[tokio::main] // <--- NEW: This macro starts the Async Runtime
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== BinLing CLI v{} (Node) ===", binling_core::version());
    println!("Initializing Levin Lattice VM...");

    // 1. Initialize the VM
    let mut vm = LatticeVM::new();

    // --- (Keep the Simulation for sanity check) ---
    // We create a quick dummy capsule just to prove the engine works on boot
    let dummy = Capsule {
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
        payload: vec![0x12, 0x20, 0xFF], // INC, LOG, HALT
    };
    vm.activate(dummy);
    vm.next_cycle(); // Run once to warm up
    println!("> [Boot Check] VM warm-up complete.\n");

    // --- NEW: PHASE 2 - NETWORKING ---

    // 2. Define the Port
    let addr = "127.0.0.1:4000";
    println!("> Binding to Lattice Network on {}...", addr);

    // 3. Open the Socket (The Listener)
    let listener = TcpListener::bind(addr).await?;
    println!("> [LISTENING] Node is ready. Waiting for peers...");

    // 4. The Server Loop (Infinite)
    loop {
        // Wait for a new connection (this pauses execution until someone connects)
        let (mut socket, peer_addr) = listener.accept().await?;
        println!("> [NEW CONNECTION] Peer joined from: {}", peer_addr);

        // Spawn a background task to handle this specific connection
        // (This allows us to handle multiple peers at once)
        tokio::spawn(async move {
            let mut buf = [0; 1024];

            // In a real app, we would read the Handshake here.
            // For now, just read whatever they send and print it.
            match socket.read(&mut buf).await {
                Ok(n) if n == 0 => return, // Connection closed
                Ok(n) => {
                    println!("  [RECV] Received {} bytes from peer.", n);
                    // TODO: Decode NetMessage here
                }
                Err(e) => println!("  [ERR] Socket error: {}", e),
            }
        });
    }
}
