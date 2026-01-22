use binling_core::capsules::{Capsule, CapsuleHeader, SquareSpace};
use binling_core::vm::LatticeVM;
use tokio::net::TcpListener;
use tokio::sync::mpsc; // <--- NEW: The Channel Tool

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== BinLing CLI v{} (Node) ===", binling_core::version());
    println!("Initializing Levin Lattice VM...");

    // 1. Create the Communication Channel
    // (tx = Transmitter, rx = Receiver)
    // Buffer size 100 means we can hold 100 capsules in the "Mailbox" before we stop accepting more.
    let (tx, mut rx) = mpsc::channel::<Capsule>(100);

    // 2. Start the Network Listener in the BACKGROUND
    // We clone the transmitter 'tx' so the background task can send mail to the main thread.
    let tx_for_network = tx.clone();

    tokio::spawn(async move {
        let addr = "127.0.0.1:4000";
        println!("> [NET] Binding to {}...", addr);
        let listener = TcpListener::bind(addr).await.expect("Failed to bind port");
        println!("> [NET] Listening for peers...");

        loop {
            let (mut socket, peer_addr) = listener.accept().await.expect("Accept error");
            // Clone the transmitter again for THIS specific connection
            let tx_for_connection = tx_for_network.clone();

            tokio::spawn(async move {
                use binling_core::net::{recv_message, send_message, NetMessage};

                // Handshake (Simplified for brevity)
                if let Ok(NetMessage::Hello { .. }) = recv_message(&mut socket).await {
                    let _ = send_message(
                        &mut socket,
                        &NetMessage::Welcome {
                            server_version: "0.1.0".to_string(),
                        },
                    )
                    .await;
                }

                // Listen for Capsules
                loop {
                    if let Ok(NetMessage::InjectCapsule(capsule)) = recv_message(&mut socket).await
                    {
                        println!(
                            "  >> [NET] Recv Capsule {}. Forwarding to VM...",
                            capsule.header.capsule_id
                        );

                        // 1. Send to VM (Internal)
                        if let Err(_) = tx_for_connection.send(capsule.clone()).await {
                            // Note: We use .clone() now because we need to keep a copy to send back!
                            println!("  >> [ERR] VM is dead/closed.");
                            break;
                        }

                        // 2. Send Receipt back to Client (External) - THE BOOMERANG
                        // We will modify the ID to show it was processed
                        let mut receipt = capsule; // Move the original variable here
                        receipt.header.capsule_id += 10000; // Flag it as processed

                        println!(
                            "  >> [NET] Sending Receipt (Cap {}) back to peer...",
                            receipt.header.capsule_id
                        );
                        if let Err(e) =
                            send_message(&mut socket, &NetMessage::InjectCapsule(receipt)).await
                        {
                            println!("  >> [ERR] Failed to send receipt: {}", e);
                        }
                    } else {
                        break; // Disconnected
                    }
                }
            });
        }
    });

    // 3. The Main VM Loop (The Consumer)
    // This runs on the main thread and owns the VM data.
    let mut vm = LatticeVM::new();
    println!("> [VM] Core Online. Waiting for capsules from network...");

    // We cycle forever. In a real engine, this would be a high-speed loop.
    // For now, we will wait for mail, then run a cycle.
    while let Some(new_capsule) = rx.recv().await {
        println!(
            "> [VM] Mail received! Loading Capsule {}...",
            new_capsule.header.capsule_id
        );

        // A. Inject
        vm.activate(new_capsule);

        // B. Run a Cycle
        println!("> [VM] Running Cycle...");
        vm.next_cycle();

        // C. Report
        println!(
            "> [VM] Cycle Complete. Active Cells: {}",
            vm.active_queue.len()
        );
    }

    Ok(())
}
