use crate::capsules::Capsule;
use crate::instructions::OpCode;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Read, Write};

// The Levin Lattice VM (Spec v0.1 Section 2)
// Now with Persistence capabilities!
#[derive(Serialize, Deserialize)] // <--- NEW: Allows the whole VM to be saved
pub struct LatticeVM {
    // The Active Queue: Capsules executing in the CURRENT cycle
    pub active_queue: Vec<Capsule>,

    // The Next Queue: Capsules scheduled for the NEXT cycle
    pub next_queue: Vec<Capsule>,

    // Global Clock
    pub cycle_count: u64,

    // General Purpose Registers (R0, R1, R2, R3)
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

    // --- PERSISTENCE (The Vault) ---

    // Save the entire universe to a binary file
    pub fn save_world(&self, filename: &str) -> io::Result<()> {
        // 1. Serialize memory to bytes
        // We use bincode for speed and compactness
        let encoded: Vec<u8> =
            bincode::serialize(&self).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // 2. Write to disk
        let mut file = File::create(filename)?;
        file.write_all(&encoded)?;
        Ok(())
    }

    // Load a universe from a binary file
    pub fn load_world(filename: &str) -> io::Result<Self> {
        // 1. Read from disk
        let mut file = File::open(filename)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // 2. Deserialize bytes to memory
        let decoded: Self =
            bincode::deserialize(&buffer).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(decoded)
    }

    // --- EXECUTION CORE ---

    // Spec v0.1 Section 2.4: Activation Primitive
    pub fn activate(&mut self, capsule: Capsule) {
        self.next_queue.push(capsule);
    }

    // Spec v0.1 Section 2.5: Queue Swap, Sort, AND EXECUTE
    pub fn next_cycle(&mut self) {
        // 1. Swap Queues
        self.active_queue = std::mem::take(&mut self.next_queue);

        // 2. Increment Clock
        self.cycle_count += 1;

        // 3. Deterministic Sort
        self.active_queue.sort_by(|a, b| {
            a.header
                .priority
                .cmp(&b.header.priority)
                .then(a.header.coord_z.cmp(&b.header.coord_z))
                .then(a.header.coord_y.cmp(&b.header.coord_y))
                .then(a.header.coord_x.cmp(&b.header.coord_x))
        });

        // 4. EXECUTE (The "Brain")
        let mut birth_queue: Vec<Capsule> = Vec::new();

        for i in 0..self.active_queue.len() {
            let mut capsule = self.active_queue[i].clone();
            self.execute_capsule(&mut capsule, &mut birth_queue);
        }

        // 5. Move babies to the Next Queue
        self.next_queue.append(&mut birth_queue);
    }

    // The Interpreter Loop
    fn execute_capsule(&mut self, capsule: &mut Capsule, birth_queue: &mut Vec<Capsule>) {
        let mut pc = 0; // Program Counter

        while pc < capsule.payload.len() {
            let op_byte = capsule.payload[pc];
            pc += 1;

            if let Some(op) = OpCode::from_u8(op_byte) {
                match op {
                    OpCode::NOOP => {}
                    OpCode::HALT => break,

                    // Metabolism (Math)
                    OpCode::ADD => {
                        self.registers[0] = self.registers[0].wrapping_add(self.registers[1])
                    }
                    OpCode::SUB => {
                        self.registers[0] = self.registers[0].wrapping_sub(self.registers[1])
                    }
                    OpCode::INC => self.registers[0] = self.registers[0].wrapping_add(1),
                    OpCode::DEC => self.registers[0] = self.registers[0].wrapping_sub(1),

                    // Senses (Output)
                    OpCode::LOG => {
                        println!(
                            "VM [Cycle {} | Cap {}]: R0 = {}",
                            self.cycle_count, capsule.header.capsule_id, self.registers[0]
                        );
                    }

                    // REPRODUCTION (Mitosis)
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
