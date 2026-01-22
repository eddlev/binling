mod ws_server; // Import the new module

use binling_core::capsules::Capsule;
use binling_core::vm::LatticeVM;
use std::path::Path;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc}; // We use both channel types now

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== BinLing CLI v{} (Node) ===", binling_core::version());
    println!("Initializing Levin Lattice VM...");

    // --- CHANNEL 1: INPUT (TCP -> VM) ---
    // MPSC: Multiple producers (clients), Single consumer (VM)
    let (tx_input, mut rx_input) = mpsc::channel::<Capsule>(100);

    // --- CHANNEL 2: OUTPUT (VM -> WebSockets) ---
    // Broadcast: Single producer (VM), Multiple consumers (browsers)
    let (tx_status, _) = broadcast::channel::<String>(16);

    // --- SYSTEM 1: TCP SERVER (Port 4000) ---
    let tx_for_tcp = tx_input.clone();
    tokio::spawn(async move {
        let addr = "127.0.0.1:4000";
        println!("> [NET] TCP Node listening on {}...", addr);
        let listener = TcpListener::bind(addr).await.expect("Failed to bind TCP");

        loop {
            let (mut socket, _) = listener.accept().await.expect("Accept error");
            let tx = tx_for_tcp.clone();

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

                        // Send to VM
                        if let Err(_) = tx.send(capsule.clone()).await {
                            break;
                        }

                        // Send Receipt (The Boomerang)
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

    // --- SYSTEM 2: WEBSOCKET GATEWAY (Port 8081) ---
    let tx_for_ws = tx_status.clone();
    tokio::spawn(async move {
        if let Err(e) = ws_server::start_ws_server(tx_for_ws).await {
            println!("> [ERR] WebSocket Server failed: {}", e);
        }
    });

    // --- SYSTEM 3: THE VM (The Heart) ---
    let mut vm;
    if Path::new("universe.bin").exists() {
        match LatticeVM::load_world("universe.bin") {
            Ok(v) => {
                vm = v;
                println!("> [VAULT] World Loaded. Cycle {}.", vm.cycle_count);
            }
            Err(_) => {
                vm = LatticeVM::new();
                println!("> [VAULT] Load failed. New World.");
            }
        }
    } else {
        println!("> [VAULT] New World Created.");
        vm = LatticeVM::new();
    }

    println!("> [VM] Core Online. Pulse 100ms.");
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));

    // The Infinite Loop
    loop {
        tokio::select! {
            // EVENT A: Capsule Arrived via TCP
            maybe_capsule = rx_input.recv() => {
                if let Some(c) = maybe_capsule {
                    println!("> [VM] INJECTING Capsule {}...", c.header.capsule_id);
                    vm.activate(c);
                }
            }

            // EVENT B: Heartbeat Tick
            _ = interval.tick() => {
                if !vm.is_void() {
                    vm.next_cycle();

                    // 1. Broadcast Status to WebSockets (Every 5 cycles)
                    if vm.cycle_count % 5 == 0 {
                        let status = format!("Cycle:{}|Active:{}", vm.cycle_count, vm.active_queue.len());
                        // Send but don't crash if no browsers are connected
                        let _ = tx_status.send(status);
                    }

                    // 2. Stats Log (Every 50 cycles)
                    if vm.cycle_count % 50 == 0 {
                        println!("> [STATS] Cycle {}: Active Cells = {}", vm.cycle_count, vm.active_queue.len());
                    }

                    // 3. Auto-Save (Every 50 cycles)
                    if vm.cycle_count % 50 == 0 {
                        let _ = vm.save_world("universe.bin");
                    }
                }
            }
        }
    }
}
