use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

pub async fn start_ws_server(
    tx: broadcast::Sender<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = "127.0.0.1:8081";

    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => {
            println!("> [WS-NET] SUCCESS: WebSocket listening on ws://{}", addr);
            l
        }
        Err(e) => {
            eprintln!(
                "!! [WS-NET] CRITICAL FAILURE: Could not bind to {}: {}",
                addr, e
            );
            return Err(Box::new(e));
        }
    };

    while let Ok((stream, _)) = listener.accept().await {
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            // 1. Handshake
            let ws_stream = match accept_async(stream).await {
                Ok(ws) => ws,
                Err(e) => {
                    eprintln!("> [WS-NET] Handshake Error: {}", e);
                    return;
                }
            };

            // 2. Split the stream safely
            let (mut write, mut read) = ws_stream.split();

            // 3. The Dual Loop (Talk AND Listen)
            loop {
                tokio::select! {
                    // JOB A: Send Data to Browser
                    msg = rx.recv() => {
                        if let Ok(payload) = msg {
                            if write.send(Message::Text(payload)).await.is_err() {
                                break; // Browser disconnected
                            }
                        }
                    }
                    // JOB B: Keep Connection Alive (Drain incoming Pings)
                    // If we don't do this, the connection rots and resets.
                    incoming = read.next() => {
                        match incoming {
                            Some(Ok(_)) => {}, // Keep alive
                            Some(Err(_)) | None => break, // Connection died
                        }
                    }
                }
            }
        });
    }

    Ok(())
}
