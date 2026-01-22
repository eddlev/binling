use binling_core::capsules::{Capsule, CapsuleHeader, SquareSpace};
use binling_core::net::{recv_message, send_message, NetMessage};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = "127.0.0.1:4000";
    println!("=== BinLing Client (Teleporter) ===");
    println!("> Connecting to Node at {}...", addr);

    // 1. Connect
    let mut socket = TcpStream::connect(addr).await?;
    println!("> [CONNECTED] Socket open.");

    // 2. Handshake
    let hello = NetMessage::Hello {
        version: "0.1.0".to_string(),
        node_id: 999,
    };
    send_message(&mut socket, &hello).await?;

    // 3. Wait for Welcome
    match recv_message(&mut socket).await {
        Ok(NetMessage::Welcome { .. }) => println!("> [SUCCESS] Handshake Verified."),
        _ => panic!("> [ERR] Server was rude."),
    }

    // 4. PREPARE THE CARGO (A Capsule)
    println!("> [GENETICS] Constructing Capsule...");
    let cargo = Capsule {
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
            coord_x: 50,
            coord_y: 50,
            coord_z: 50,     // Destination Coordinates
            capsule_id: 777, // Lucky Number
            dict_hash: [0; 32],
            policy_core_hash: [0; 32],
            capsule_hash: [0; 32],
        },
        policy_core: vec![],
        payload: vec![0x12, 0x20, 0x30, 0xFF], // The "Life" Code (INC, LOG, SPAWN, HALT)
    };

    // 5. TELEPORT
    println!(
        "> [SEND] Teleporting Capsule ID {}...",
        cargo.header.capsule_id
    );
    let msg = NetMessage::InjectCapsule(cargo);
    send_message(&mut socket, &msg).await?;

    // --- NEW: WAIT FOR RECEIPT ---
    println!("> [WAIT] Waiting for confirmation from Server...");

    match recv_message(&mut socket).await {
        Ok(NetMessage::InjectCapsule(receipt)) => {
            println!("\n> [BOOMERANG] SERVER RETURNED CAPSULE!");
            println!("> [CHECK] Original ID: 777");
            println!("> [CHECK] Returned ID: {}", receipt.header.capsule_id);

            if receipt.header.capsule_id == 10777 {
                println!("> [SUCCESS] Full Round-Trip Verified.");
            } else {
                println!("> [WARN] ID mismatch.");
            }
        }
        Ok(msg) => println!("> [ERR] Unexpected response: {:?}", msg),
        Err(e) => println!("> [ERR] Server disconnected: {}", e),
    }

    println!("> [DONE] Disconnecting.");
    Ok(())
}
