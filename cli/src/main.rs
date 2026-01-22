use binling_core::capsules::{Capsule, CapsuleHeader, SquareSpace};
use binling_core::vm::LatticeVM;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== BinLing CLI v{} (Node) ===", binling_core::version());
    println!("Initializing Levin Lattice VM...");

    // 1. Initialize the VM
    let mut vm = LatticeVM::new();

    // Boot Check (Warm-up)
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
    vm.next_cycle();
    println!("> [Boot Check] VM warm-up complete.\n");

    // --- PHASE 2 - NETWORKING ---

    // 2. Define the Port
    let addr = "127.0.0.1:4000";
    println!("> Binding to Lattice Network on {}...", addr);

    // 3. Open the Socket
    let listener = TcpListener::bind(addr).await?;
    println!("> [LISTENING] Node is ready. Waiting for peers...");

    // 4. The Server Loop
    loop {
        let (mut socket, peer_addr) = listener.accept().await?;
        println!("> [NEW CONNECTION] Peer joined from: {}", peer_addr);

        tokio::spawn(async move {
            use binling_core::net::{recv_message, send_message, NetMessage};

            println!("  [HANDSHAKE] Waiting for Hello from {}...", peer_addr);

            // 1. Attempt to receive a message
            match recv_message(&mut socket).await {
                Ok(NetMessage::Hello { version, node_id }) => {
                    println!(
                        "  [HANDSHAKE] Received HELLO from Node {} (v{})",
                        node_id, version
                    );

                    // 2. Send Welcome Back
                    let reply = NetMessage::Welcome {
                        server_version: binling_core::version().to_string(), // <-- Fixed type error
                    };

                    if let Err(e) = send_message(&mut socket, &reply).await {
                        println!("  [ERR] Failed to send Welcome: {}", e);
                    } else {
                        println!("  [HANDSHAKE] Sent WELCOME. Connection established.");
                    }
                }
                Ok(msg) => {
                    println!("  [ERR] Expected Hello, got {:?} from {}", msg, peer_addr);
                }
                Err(e) => {
                    println!("  [ERR] Connection failed or closed: {}", e);
                }
            }
        });
    }
}
