use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum OpCode {
    NOOP = 0,
    HALT = 1,
    ADD = 2,
    SUB = 3,
    INC = 4,
    DEC = 5,
    LOG = 6,
    SPAWN = 7, // <--- CRITICAL: Must be 7 to match Client
}

impl OpCode {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(OpCode::NOOP),
            1 => Some(OpCode::HALT),
            2 => Some(OpCode::ADD),
            3 => Some(OpCode::SUB),
            4 => Some(OpCode::INC),
            5 => Some(OpCode::DEC),
            6 => Some(OpCode::LOG),
            7 => Some(OpCode::SPAWN),
            _ => None,
        }
    }
}
