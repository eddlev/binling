use crate::capsules::Capsule;
use crate::instructions::OpCode;
use serde::{Deserialize, Serialize};

// The Levin Lattice VM
#[derive(Serialize, Deserialize)]
pub struct LatticeVM {
    pub active_queue: Vec<Capsule>,
    pub next_queue: Vec<Capsule>,
    pub cycle_count: u64,
    pub registers: [i32; 4],
}

impl LatticeVM {
    pub fn new() -> Self {
        Self {
            active_queue: Vec::new(),
            next_queue: Vec::new(),
            cycle_count: 0,
            registers: [0; 4],
        }
    }

    // --- PERSISTENCE (CLI ONLY) ---
    // These functions strictly require File I/O, so we hide them from WASM.

    #[cfg(feature = "cli-mode")]
    pub fn save_world(&self, filename: &str) -> std::io::Result<()> {
        use std::fs::File;
        use std::io::Write;

        let encoded: Vec<u8> = bincode::serialize(&self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let mut file = File::create(filename)?;
        file.write_all(&encoded)?;
        Ok(())
    }

    #[cfg(feature = "cli-mode")]
    pub fn load_world(filename: &str) -> std::io::Result<Self> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(filename)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let decoded: Self = bincode::deserialize(&buffer)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        Ok(decoded)
    }

    // --- EXECUTION CORE (Available everywhere) ---

    pub fn activate(&mut self, capsule: Capsule) {
        self.next_queue.push(capsule);
    }

    pub fn next_cycle(&mut self) {
        self.active_queue = std::mem::take(&mut self.next_queue);
        self.cycle_count += 1;

        self.active_queue.sort_by(|a, b| {
            a.header
                .priority
                .cmp(&b.header.priority)
                .then(a.header.coord_z.cmp(&b.header.coord_z))
                .then(a.header.coord_y.cmp(&b.header.coord_y))
                .then(a.header.coord_x.cmp(&b.header.coord_x))
        });

        let mut birth_queue: Vec<Capsule> = Vec::new();

        for i in 0..self.active_queue.len() {
            let mut capsule = self.active_queue[i].clone();
            self.execute_capsule(&mut capsule, &mut birth_queue);
        }

        self.next_queue.append(&mut birth_queue);
    }

    fn execute_capsule(&mut self, capsule: &mut Capsule, birth_queue: &mut Vec<Capsule>) {
        let mut pc = 0;
        while pc < capsule.payload.len() {
            let op_byte = capsule.payload[pc];
            pc += 1;

            if let Some(op) = OpCode::from_u8(op_byte) {
                match op {
                    OpCode::NOOP => {}
                    OpCode::HALT => break,
                    OpCode::ADD => {
                        self.registers[0] = self.registers[0].wrapping_add(self.registers[1])
                    }
                    OpCode::SUB => {
                        self.registers[0] = self.registers[0].wrapping_sub(self.registers[1])
                    }
                    OpCode::INC => self.registers[0] = self.registers[0].wrapping_add(1),
                    OpCode::DEC => self.registers[0] = self.registers[0].wrapping_sub(1),
                    OpCode::LOG => {
                        // In WASM, println! goes to the browser console.
                        // In CLI, it goes to stdout.
                        // This is safe to keep.
                        println!(
                            "VM [Cycle {} | Cap {}]: R0 = {}",
                            self.cycle_count, capsule.header.capsule_id, self.registers[0]
                        );
                    }
                    OpCode::SPAWN => {
                        let mut daughter = capsule.clone();
                        daughter.header.capsule_id += 1000;
                        println!(
                            ">> [MITOSIS] Cap {} spawned Cap {}!",
                            capsule.header.capsule_id, daughter.header.capsule_id
                        );
                        birth_queue.push(daughter);
                    }
                }
            }
        }
    }

    pub fn is_void(&self) -> bool {
        self.active_queue.is_empty() && self.next_queue.is_empty()
    }
}
