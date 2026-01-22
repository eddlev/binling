use binling_core::capsules::Capsule;
use binling_core::vm::LatticeVM;
use std::path::Path;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== BinLing CLI v{} (Node) ===", binling_core::version());
    println!("Initializing Levin Lattice VM...");

    // 1. Create Channel
    let (tx, mut rx) = mpsc::channel::<Capsule>(100);

    // 2. Start Network (Background)
    let tx_for_network = tx.clone();
    tokio::spawn(async move {
        let addr = "127.0.0.1:4000";
        println!("> [NET] Binding to {}...", addr);
        let listener = TcpListener::bind(addr).await.expect("Failed to bind port");
        println!("> [NET] Listening for peers...");

        loop {
            let (mut socket, _peer_addr) = listener.accept().await.expect("Accept error");
            let tx_for_connection = tx_for_network.clone();

            tokio::spawn(async move {
                use binling_core::net::{recv_message, send_message, NetMessage};

                // Handshake
                if let Ok(NetMessage::Hello { .. }) = recv_message(&mut socket).await {
                    let _ = send_message(
                        &mut socket,
                        &NetMessage::Welcome {
                            server_version: "0.1.0".to_string(),
                        },
                    )
                    .await;
                }

                // Capsule Loop
                loop {
                    if let Ok(NetMessage::InjectCapsule(capsule)) = recv_message(&mut socket).await
                    {
                        println!(
                            "  >> [NET] Recv Capsule {}. Forwarding...",
                            capsule.header.capsule_id
                        );
                        if let Err(_) = tx_for_connection.send(capsule.clone()).await {
                            break;
                        }

                        // Receipt
                        let mut receipt = capsule;
                        receipt.header.capsule_id += 10000;
                        let _ =
                            send_message(&mut socket, &NetMessage::InjectCapsule(receipt)).await;
                    } else {
                        break;
                    }
                }
            });
        }
    });

    // 3. THE VAULT: Try to Load Existing World
    let mut vm;
    if Path::new("universe.bin").exists() {
        println!("> [VAULT] Found existing universe. Loading...");
        match LatticeVM::load_world("universe.bin") {
            Ok(loaded_vm) => {
                vm = loaded_vm;
                println!("> [VAULT] Success! Resuming from Cycle {}.", vm.cycle_count);
            }
            Err(e) => {
                println!("> [VAULT] Load failed ({}). Creating new Big Bang.", e);
                vm = LatticeVM::new();
            }
        }
    } else {
        println!("> [VAULT] No universe found. Creating new Big Bang.");
        vm = LatticeVM::new();
    }

    println!("> [VM] Core Online. Pulse set to 100ms.");
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));

    // 4. The Loop of Life (With Auto-Save)
    loop {
        tokio::select! {
            // Network Event
            maybe_capsule = rx.recv() => {
                match maybe_capsule {
                    Some(new_capsule) => {
                        println!("> [VM] INJECTING Capsule {}...", new_capsule.header.capsule_id);
                        vm.activate(new_capsule);
                    }
                    None => break,
                }
            }

            // Heartbeat Event
            _ = interval.tick() => {
                if !vm.is_void() {
                    vm.next_cycle();

                    // Stats Log (Every 10 cycles)
                    if vm.cycle_count % 10 == 0 {
                        println!("> [STATS] Cycle {}: Active Cells = {}", vm.cycle_count, vm.active_queue.len());
                    }

                    // AUTO-SAVE (Every 50 cycles / ~5 seconds)
                    if vm.cycle_count % 50 == 0 {
                        print!("> [VAULT] Saving Universe... ");
                        if let Err(e) = vm.save_world("universe.bin") {
                            println!("Failed: {}", e);
                        } else {
                            println!("Saved.");
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
