use binling_core::capsules::Capsule;
use binling_core::vm::LatticeVM;
use serde_json::json;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::sync::broadcast;

mod ws_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== BinLing CLI v0.1.0 (Node) ===");

    // 1. Initialize VM
    let vm = match LatticeVM::load_world("universe.bin") {
        Ok(loaded_vm) => {
            println!("> [VAULT] Universe Loaded.");
            Arc::new(Mutex::new(loaded_vm))
        }
        Err(_) => {
            println!("> [VAULT] New World Created.");
            Arc::new(Mutex::new(LatticeVM::new()))
        }
    };

    // 2. Setup Broadcast
    let (tx_status, _rx_status) = broadcast::channel(100); // Increased buffer size

    // 3. Start WS Server
    let tx_for_ws = tx_status.clone();
    tokio::spawn(async move {
        if let Err(e) = ws_server::start_ws_server(tx_for_ws).await {
            eprintln!("WS Server Error: {}", e);
        }
    });

    // 4. Start TCP Listener (Capsule Receiver)
    let listener = TcpListener::bind("127.0.0.1:4000").await?;
    println!("> [NET] TCP Node listening on 127.0.0.1:4000...");

    let vm_for_net = vm.clone();
    tokio::spawn(async move {
        loop {
            if let Ok((mut socket, _)) = listener.accept().await {
                let vm_clone = vm_for_net.clone();
                tokio::spawn(async move {
                    let mut buffer = Vec::new();
                    if let Ok(_) = socket.read_to_end(&mut buffer).await {
                        if !buffer.is_empty() {
                            if let Ok(c) = bincode::deserialize::<Capsule>(&buffer) {
                                println!(
                                    ">> [NET] Recv Capsule {}. Forwarding...",
                                    c.header.capsule_id
                                );
                                let mut locked_vm = vm_clone.lock().unwrap();
                                locked_vm.activate(c);
                            }
                        }
                    }
                });
            }
        }
    });

    println!("> [WS-NET] Gateway active on: ws://127.0.0.1:8081");
    println!("> [SYSTEM] Core Loop Running...");

    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));

    // 5. THE FIXED LOOP
    loop {
        // A. SLEEP FIRST (Releases lock implicitly because we haven't grabbed it yet)
        interval.tick().await;

        // B. LOCK & COMPUTE
        {
            let mut vm = vm.lock().unwrap();

            // Broadcast every cycle for smooth visuals
            if true {
                // Use next_queue to show the "Resting State" of the universe
                let cell_data: Vec<(i32, i32, i32)> = vm
                    .next_queue
                    .iter()
                    .map(|c| {
                        (
                            c.header.coord_x as i32,
                            c.header.coord_y as i32,
                            c.header.coord_z as i32,
                        )
                    })
                    .collect();

                let payload = json!({
                    "cycle": vm.cycle_count,
                    "active_count": vm.next_queue.len(),
                    "cells": cell_data
                });

                // We ignore errors if nobody is listening
                let _ = tx_status.send(payload.to_string());
            }

            // Advance Physics
            if !vm.is_void() {
                vm.next_cycle();
                if vm.cycle_count % 50 == 0 {
                    println!(
                        "> [STATS] Cycle {}: Active Cells = {}",
                        vm.cycle_count,
                        vm.next_queue.len()
                    );
                }
            }
        } // Lock is released here immediately
    }
}
