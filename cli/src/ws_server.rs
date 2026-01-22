use futures_util::{SinkExt, StreamExt};
use std::error::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::accept_async;

// Start the WebSocket Gateway
pub async fn start_ws_server(
    tx_vm_status: broadcast::Sender<String>,
) -> Result<(), Box<dyn Error>> {
    let addr = "127.0.0.1:8081"; // Web browsers will connect here
    let listener = TcpListener::bind(&addr).await?;
    println!("> [WS-NET] Gateway active on: ws://{}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let tx = tx_vm_status.clone();
        tokio::spawn(accept_connection(stream, tx));
    }

    Ok(())
}

async fn accept_connection(stream: TcpStream, tx: broadcast::Sender<String>) {
    if let Ok(ws_stream) = accept_async(stream).await {
        let (mut ws_sender, _ws_receiver) = ws_stream.split();

        // Subscribe to the VM's status updates
        let mut rx = tx.subscribe();

        // Loop: Whenever the VM shouts a status update, forward it to this browser
        while let Ok(status_msg) = rx.recv().await {
            let msg = tokio_tungstenite::tungstenite::Message::Text(status_msg);
            if ws_sender.send(msg).await.is_err() {
                break; // Browser disconnected
            }
        }
    }
}
