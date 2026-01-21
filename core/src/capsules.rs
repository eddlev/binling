use serde::{Deserialize, Serialize};

// The fixed set of allowed Cube sizes (Spec v0.1 Section 4.2)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum SquareSpace {
    SS8 = 8,
    SS16 = 16,
    SS32 = 32,
    SS64 = 64,
    SS128 = 128,
}

// The normative Fixed Header (Spec v0.1 Section 6.1)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapsuleHeader {
    pub magic: [u8; 4],         // "BLE1"
    pub version_major: u8,      // 0
    pub version_minor: u8,      // 1
    pub flags: u16,             // Fail-closed, Verify-required
    pub ss_n: SquareSpace,      // Cube Size
    pub priority: u8,           // 0-255
    
    // Lengths
    pub header_len: u16,        // Fixed + Policy
    pub policy_len: u16,        // Length of Q0-Q2
    pub payload_len: u32,       // Length of Q3+
    pub pad_len: u32,           // Zero padding count
    
    // Coordinates (Lattice Position)
    pub coord_x: i16,
    pub coord_y: i16,
    pub coord_z: i16,
    
    // Identity
    pub capsule_id: u32,
    
    // Integrity (Spec v0.1 Section 8)
    pub dict_hash: [u8; 32],        // Opcode Table Hash
    pub policy_core_hash: [u8; 32], // Q0-Q2 Hash
    pub capsule_hash: [u8; 32],     // Full Integrity Hash
}

// The complete Capsule structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capsule {
    pub header: CapsuleHeader,
    pub policy_core: Vec<u8>,   // Canonical Q0-Q2 bytes
    pub payload: Vec<u8>,       // ASCII Instruction Stream
    // Note: Padding is generated during serialization, not stored here.
}

impl Capsule {
    // Helper to calculate total capacity (N^3)
    pub fn capacity(&self) -> u32 {
        let n = self.header.ss_n as u32;
        n * n * n
    }
}
