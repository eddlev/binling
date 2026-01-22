use crate::capsules::Capsule;
use serde::{Deserialize, Serialize};

// The Language of the Lattice Network
// Spec v0.1 Section 4: P2P Protocol

#[derive(Serialize, Deserialize, Debug)]
pub enum NetMessage {
    // 1. The Handshake (Initiation)
    // "Hi, I am Node X, running Version Y"
    Hello { version: String, node_id: u32 },

    // 2. The Handshake (Response)
    // "Welcome, I see you. I am running Version Z"
    Welcome { server_version: String },

    // 3. The Payload (Work)
    // "Here is a living Capsule. Take care of it."
    InjectCapsule(Capsule),
}
