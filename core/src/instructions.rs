use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum OpCode {
    NOOP = 0,
    HALT = 1,
    // MATH
    ADD = 3,
    SUB = 4,
    INC = 5,
    DEC = 6,
    // I/O
    LOG = 7,
    SPAWN = 8,
    // MEMORY
    STORE = 9,
    LOAD = 10,
    // CONTROL
    JMP = 11,
    BEQ = 12,
    // LIFE
    REPL = 13, // Replicate (Viral Copy)
}

impl OpCode {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(OpCode::NOOP),
            1 => Some(OpCode::HALT),
            3 => Some(OpCode::ADD),
            4 => Some(OpCode::SUB),
            5 => Some(OpCode::INC),
            6 => Some(OpCode::DEC),
            7 => Some(OpCode::LOG),
            8 => Some(OpCode::SPAWN),
            9 => Some(OpCode::STORE),
            10 => Some(OpCode::LOAD),
            11 => Some(OpCode::JMP),
            12 => Some(OpCode::BEQ),
            13 => Some(OpCode::REPL),
            _ => None,
        }
    }
}
