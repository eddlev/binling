use serde::{Deserialize, Serialize};

// The Genetic Alphabet (Spec v0.1 Section 3)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum OpCode {
    // === 0x00: Lifecycle ===
    NOOP = 0x00, // Do nothing (Energy save)
    HALT = 0xFF, // Stop execution immediately

    // === 0x10: Arithmetic (The "Metabolism") ===
    ADD = 0x10, // Register[0] = Register[0] + Register[1]
    SUB = 0x11, // Register[0] = Register[0] - Register[1]
    INC = 0x12, // Register[0]++
    DEC = 0x13, // Register[0]--

    // === 0x20: I/O (The "Senses") ===
    LOG = 0x20, // Print Register[0] to CLI (Debug)
}

impl OpCode {
    // The Decoder: Raw Byte -> Meaning
    pub fn from_u8(byte: u8) -> Option<Self> {
        match byte {
            0x00 => Some(OpCode::NOOP),
            0xFF => Some(OpCode::HALT),
            0x10 => Some(OpCode::ADD),
            0x11 => Some(OpCode::SUB),
            0x12 => Some(OpCode::INC),
            0x13 => Some(OpCode::DEC),
            0x20 => Some(OpCode::LOG),
            _ => None, // "Junk DNA" (Unknown instruction)
        }
    }
}
