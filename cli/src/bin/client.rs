use binling_core::capsules::{Capsule, CapsuleHeader, SquareSpace};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    println!("=== BinLing Client (Blindfire Mode) ===");

    // 1. Connect to the Server
    println!("> Connecting to Node at 127.0.0.1:4000...");
    let mut stream = TcpStream::connect("127.0.0.1:4000")?;

    // Set timeouts to prevent hanging
    stream.set_read_timeout(Some(Duration::from_millis(500)))?;
    stream.set_write_timeout(Some(Duration::from_millis(500)))?;

    println!("> [CONNECTED] Socket open. Bypassing handshake...");

    // 2. SKIP HANDSHAKE
    // We assume the server is listening.

    // 3. Construct the "Architect" Capsule
    println!("> [GENETICS] Constructing Architect Payload...");

    // 4,096 SPAWN instructions (0x07)
    // This forces the Kernel to spawn 1 brick per cycle for 4096 cycles.
    let payload = vec![7u8; 4096];

    let capsule = Capsule {
        header: CapsuleHeader {
            magic: *b"BLE1",
            version_major: 0,
            version_minor: 1,
            flags: 0,
            capsule_id: 777, // Kernel ID
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

    // 4. Serialize
    let encoded = bincode::serialize(&capsule).expect("Failed to serialize");

    // 5. Send Immediately with Force Push
    println!("> [SEND] Teleporting Capsule ({} bytes)...", encoded.len());
    stream.write_all(&encoded)?;
    stream.flush()?; // Ensure data leaves the buffer
    println!("> [SENT] Payload delivered.");

    // 6. Polite Pause
    // Give the server 100ms to ingest the data before we kill the connection.
    thread::sleep(Duration::from_millis(100));

    // Optional: Peek for response (don't block if none)
    let mut response_buffer = Vec::new();
    if let Ok(_) = stream.read_to_end(&mut response_buffer) {
        if !response_buffer.is_empty() {
            println!("> [ACK] Server acknowledged receipt.");
        }
    }

    println!("> [DONE] Disconnecting.");
    Ok(())
}
