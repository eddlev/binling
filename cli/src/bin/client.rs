use binling_core::net::{recv_message, send_message, NetMessage};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = "127.0.0.1:4000";
    println!("=== BinLing Client Test ===");
    println!("> Connecting to Node at {}...", addr);

    // 1. Connect to the Server
    let mut socket = TcpStream::connect(addr).await?;
    println!("> [CONNECTED] Socket open.");

    // 2. Say Hello
    let hello = NetMessage::Hello {
        version: "0.1.0".to_string(),
        node_id: 999, // We are Node 999
    };

    println!("> [SEND] Sending Handshake: {:?}", hello);
    send_message(&mut socket, &hello).await?;

    // 3. Wait for Welcome
    println!("> [WAIT] Waiting for reply...");
    match recv_message(&mut socket).await {
        Ok(NetMessage::Welcome { server_version }) => {
            println!("> [SUCCESS] Handshake Complete!");
            println!(
                "> Server says: 'Welcome! I am running version {}'",
                server_version
            );
        }
        Ok(msg) => {
            println!("> [ERR] Unexpected response: {:?}", msg);
        }
        Err(e) => {
            println!("> [ERR] Failed to receive reply: {}", e);
        }
    }

    Ok(())
}
