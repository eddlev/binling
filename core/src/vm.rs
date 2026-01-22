use crate::capsules::Capsule;
use crate::instructions::OpCode;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LatticeVM {
    pub active_queue: Vec<Capsule>,
    pub next_queue: Vec<Capsule>,
    pub cycle_count: u64,
    pub registers: [i32; 4],
    pub next_id: u32,
}

impl LatticeVM {
    pub fn new() -> Self {
        Self {
            active_queue: Vec::new(),
            next_queue: Vec::new(),
            cycle_count: 0,
            registers: [0; 4],
            next_id: 1,
        }
    }

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

    pub fn activate(&mut self, capsule: Capsule) {
        self.next_queue.push(capsule);
    }

    pub fn next_cycle(&mut self) {
        self.active_queue = std::mem::take(&mut self.next_queue);
        self.cycle_count += 1;

        // Sort for deterministic execution
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

            // EXECUTE ONE STEP
            self.step_capsule(&mut capsule, &mut birth_queue);

            // PERSISTENCE:
            // We always put the capsule back into the universe unless it explicitly HALTs.
            // This ensures the "Structure" remains solid.
            self.next_queue.push(capsule);
        }

        self.next_queue.append(&mut birth_queue);
    }

    // Changed from "execute_capsule" (loop) to "step_capsule" (single tick)
    fn step_capsule(&mut self, capsule: &mut Capsule, birth_queue: &mut Vec<Capsule>) {
        // If payload is empty, the capsule is idle (Static Block)
        if capsule.payload.is_empty() {
            return;
        }

        // CONSUME TAPE: Eat the first byte
        let op_byte = capsule.payload.remove(0);

        if let Some(op) = OpCode::from_u8(op_byte) {
            match op {
                OpCode::NOOP => {}
                // HALT: We don't implement "death" here yet, effectively NOOP for now.
                // If we wanted death, we would simply NOT push it back to next_queue in next_cycle.
                OpCode::HALT => {}
                OpCode::ADD => {
                    self.registers[0] = self.registers[0].wrapping_add(self.registers[1])
                }
                OpCode::SUB => {
                    self.registers[0] = self.registers[0].wrapping_sub(self.registers[1])
                }
                OpCode::INC => self.registers[0] = self.registers[0].wrapping_add(1),
                OpCode::DEC => self.registers[0] = self.registers[0].wrapping_sub(1),
                OpCode::LOG => {
                    println!(
                        "VM [Cycle {}]: R0 = {}",
                        self.cycle_count, self.registers[0]
                    );
                }
                OpCode::SPAWN => {
                    if capsule.header.capsule_id != 0 && capsule.header.capsule_id != 777 {
                        // Regular bricks don't reproduce
                    } else {
                        // Create Child
                        let mut child = capsule.clone();

                        // IMPORTANT: Child is born with empty mind (Static Brick)
                        child.payload.clear();

                        // Assign Identity
                        child.header.capsule_id = self.next_id;
                        self.next_id += 1;

                        // Calculate Position (16-Cube)
                        let grid_size: i32 = 16;
                        let index = child.header.capsule_id as i32;
                        let x = index % grid_size;
                        let y = (index / grid_size) % grid_size;
                        let z = index / (grid_size * grid_size);
                        let offset = grid_size / 2;

                        child.header.coord_x = (x - offset) as i16;
                        child.header.coord_y = (y - offset) as i16;
                        child.header.coord_z = (z - offset) as i16;
                        child.header.priority = 0;

                        birth_queue.push(child);

                        if index % 256 == 0 {
                            println!("> [ARCHITECT] Layer Z={} Complete.", z - offset);
                        }
                    }
                }
            }
        }
    }

    pub fn is_void(&self) -> bool {
        self.active_queue.is_empty() && self.next_queue.is_empty()
    }
}
