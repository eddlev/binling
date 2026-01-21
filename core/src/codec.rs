use crate::capsules::Capsule;
use anyhow::Result; // We need to add 'anyhow' to Cargo.toml later if not present, but for now we use standard Result

// The Codec Module (Spec v0.1 Section 5)
// Handles converting Capsules <-> Raw Bytes

pub struct LatticeCodec;

impl LatticeCodec {
    // Encode: Capsule -> Bytes
    pub fn encode(capsule: &Capsule) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(capsule)
    }

    // Decode: Bytes -> Capsule
    pub fn decode(data: &[u8]) -> Result<Capsule, bincode::Error> {
        bincode::deserialize(data)
    }

    // Helper: Verify Magic Bytes (BLE1)
    pub fn verify_header(data: &[u8]) -> bool {
        if data.len() < 4 {
            return false;
        }
        &data[0..4] == b"BLE1"
    }
}
