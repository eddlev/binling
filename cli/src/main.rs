use binling_core::capsules::{Capsule, CapsuleHeader, SquareSpace};
use binling_core::vm::LatticeVM;
use serde_json::json;
use std::env;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::sync::broadcast;

mod ws_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== BinLing CLI v1.1 (Full I/O) ===");

    // 1. DETERMINE IDENTITY
    let args: Vec<String> = env::args().collect();
    let universe_id = if args.len() > 1 {
        args[1].clone()
    } else {
        "default".to_string()
    };

    let filename = format!("universe_{}.bin", universe_id);
    let interface_dir = "./interface";
    let input_file = format!("{}/oracle_in.txt", interface_dir);
    let output_file = format!("{}/oracle_out.txt", interface_dir);

    // Create Interface Directory
    if !Path::new(interface_dir).exists() {
        fs::create_dir(interface_dir)?;
        println!("> [SYSTEM] Created I/O Interface: {}", interface_dir);
    }

    // 2. Initialize VM
    let vm = match LatticeVM::load_world(&filename) {
        Ok(loaded_vm) => {
            println!(
                "> [VAULT] Universe Loaded. Cycle: {}",
                loaded_vm.cycle_count
            );
            Arc::new(Mutex::new(loaded_vm))
        }
        Err(_) => {
            println!("> [VAULT] New World Created.");
            Arc::new(Mutex::new(LatticeVM::new(universe_id.clone())))
        }
    };

    // 3. Setup Broadcast
    let (tx_status, _rx_status) = broadcast::channel(100);

    // 4. Start WS Server
    let tx_for_ws = tx_status.clone();
    tokio::spawn(async move {
        if let Err(e) = ws_server::start_ws_server(tx_for_ws).await {
            eprintln!("WS Server Error: {}", e);
        }
    });

    // 5. Start TCP Listener
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

    // 6. START THE ORACLE WATCHER (File System Bridge)
    let vm_for_oracle = vm.clone();
    let input_path = input_file.clone();

    tokio::spawn(async move {
        println!("> [ORACLE] Watching for input in: {}", input_path);
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

            if Path::new(&input_path).exists() {
                if let Ok(content) = fs::read_to_string(&input_path) {
                    if !content.trim().is_empty() {
                        println!(">> [ORACLE] Input Detected: '{}'", content.trim());

                        let payload = content.trim().as_bytes().to_vec(); // Trim whitespace

                        let capsule = Capsule {
                            header: CapsuleHeader {
                                magic: *b"BLE1",
                                version_major: 0,
                                version_minor: 1,
                                flags: 7,
                                capsule_id: 5, // Target: Oracle
                                ss_n: SquareSpace::SS64,
                                priority: 100,
                                coord_x: -10,
                                coord_y: 0,
                                coord_z: 0,
                                header_len: 0,
                                policy_len: 0,
                                payload_len: payload.len() as u32,
                                pad_len: 0,
                                dict_hash: [0; 32],
                                policy_core_hash: [0; 32],
                                capsule_hash: [0; 32],
                            },
                            policy_core: vec![],
                            payload: payload,
                        };

                        {
                            let mut locked_vm = vm_for_oracle.lock().unwrap();
                            locked_vm.activate(capsule);
                        }

                        let _ = fs::write(&input_path, "");
                    }
                }
            }
        }
    });

    println!("> [WS-NET] Gateway active on: ws://127.0.0.1:8081");
    println!("> [SYSTEM] Core Loop Running...");

    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));

    // 7. THE LOOP
    loop {
        interval.tick().await;

        {
            let mut vm = vm.lock().unwrap();

            // --- OUTPUT BRIDGE (The "Mouth" Writer) ---
            if !vm.output_buffer.is_empty() {
                for msg in vm.output_buffer.drain(..) {
                    println!("<< [ORACLE] Output Generated: '{}'", msg);
                    // Append to file (Log style)
                    use std::io::Write;
                    if let Ok(mut file) = fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&output_file)
                    {
                        if let Err(e) = writeln!(file, "{}", msg) {
                            eprintln!("!! [IO ERROR] Could not write output: {}", e);
                        }
                    }
                }
            }
            // ------------------------------------------

            // Cast u16 flag to u8
            let cell_data: Vec<(i32, i32, i32, u8)> = vm
                .next_queue
                .iter()
                .map(|c| {
                    (
                        c.header.coord_x as i32,
                        c.header.coord_y as i32,
                        c.header.coord_z as i32,
                        c.header.flags as u8,
                    )
                })
                .collect();

            let payload = json!({
                "cycle": vm.cycle_count,
                "active_count": vm.next_queue.len(),
                "cells": cell_data
            });

            let _ = tx_status.send(payload.to_string());

            if !vm.is_void() {
                vm.next_cycle();

                if vm.cycle_count % 50 == 0 {
                    match vm.save_world(&filename) {
                        Ok(_) => {}
                        Err(e) => eprintln!("!! [ERROR] Failed to save: {}", e),
                    }
                }
            }
        }
    }
}
