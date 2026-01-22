use crate::capsules::Capsule;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

// The Language of the Lattice Network
#[derive(Serialize, Deserialize, Debug)]
pub enum NetMessage {
    // 1. The Handshake (Initiation)
    Hello { version: String, node_id: u32 },

    // 2. The Handshake (Response)
    Welcome { server_version: String },

    // 3. The Payload (Work)
    InjectCapsule(Capsule),
}

// --- NETWORK I/O HELPERS ---

// UPDATE: Added "+ Send + Sync" to the return type
pub async fn send_message(
    socket: &mut TcpStream,
    msg: &NetMessage,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // 1. Serialize message to bytes
    let bytes = bincode::serialize(msg)?;

    // 2. Write the length of the message (4 bytes) prefix
    let len = bytes.len() as u32;
    socket.write_all(&len.to_be_bytes()).await?;

    // 3. Write the actual data
    socket.write_all(&bytes).await?;

    Ok(())
}

// UPDATE: Added "+ Send + Sync" to the return type
pub async fn recv_message(
    socket: &mut TcpStream,
) -> Result<NetMessage, Box<dyn Error + Send + Sync>> {
    // 1. Read the length prefix (4 bytes)
    let mut len_buf = [0u8; 4];
    socket.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;

    // 2. Read the exact number of bytes for the message
    let mut buf = vec![0u8; len];
    socket.read_exact(&mut buf).await?;

    // 3. Deserialize bytes -> NetMessage
    let msg = bincode::deserialize(&buf)?;
    Ok(msg)
}
