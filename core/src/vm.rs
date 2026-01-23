use crate::capsules::{Capsule, CapsuleHeader, SquareSpace};
use crate::instructions::OpCode;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LatticeVM {
    pub active_queue: Vec<Capsule>,
    pub next_queue: Vec<Capsule>,
    pub cycle_count: u64,
    pub registers: [i32; 4],
    pub next_id: u32,
    pub universe_id: String,
    pub output_buffer: Vec<String>,
}

impl LatticeVM {
    pub fn new(id: String) -> Self {
        let mut vm = Self {
            active_queue: Vec::new(),
            next_queue: Vec::new(),
            cycle_count: 0,
            registers: [0; 4],
            next_id: 1000,
            universe_id: id,
            output_buffer: Vec::new(),
        };

        vm.genesis();
        vm
    }

    pub fn genesis(&mut self) {
        // TRINITY + ORACLE + SCEPTER
        self.spawn_system_node(1, -12, 0, 0, 4, true); // Queen
        self.spawn_system_node(2, -12, 2, 0, 5, false); // Prince A
        self.spawn_system_node(3, -12, -2, 0, 5, false); // Prince B
        self.spawn_system_node(4, -11, 0, 0, 6, false); // Scepter
        self.spawn_system_node(5, -10, 0, 0, 7, false); // Oracle

        println!("> [MACF] Trinity Protocol & Oracle Initiated.");
    }

    fn spawn_system_node(&mut self, id: u32, x: i16, y: i16, z: i16, flag: u16, active: bool) {
        let payload = if active { vec![7u8; 4096] } else { vec![] };

        let cap = Capsule {
            header: CapsuleHeader {
                magic: *b"BLE1",
                version_major: 0,
                version_minor: 1,
                flags: flag,
                capsule_id: id,
                ss_n: SquareSpace::SS64,
                priority: 0,
                coord_x: x,
                coord_y: y,
                coord_z: z,
                header_len: 0,
                policy_len: 0,
                payload_len: payload.len() as u32,
                pad_len: 0,
                dict_hash: [0; 32],
                policy_core_hash: [0; 32],
                capsule_hash: [0; 32],
            },
            policy_core: vec![],
            payload,
        };
        self.next_queue.push(cap);
    }

    // ... (Save/Load unchanged) ...
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

        self.active_queue.sort_by(|a, b| {
            a.header
                .coord_z
                .cmp(&b.header.coord_z)
                .then(a.header.coord_y.cmp(&b.header.coord_y))
                .then(a.header.coord_x.cmp(&b.header.coord_x))
        });

        let mut birth_queue: Vec<Capsule> = Vec::new();

        for i in 0..self.active_queue.len() {
            let mut capsule = self.active_queue[i].clone();

            // EXECUTE LOGIC
            self.step_capsule(&mut capsule, &mut birth_queue);

            // --- GARBAGE COLLECTION (METABOLISM) ---
            // If the ID became 0 (Void), it means the capsule died/was consumed.
            // We do NOT push it to the next_queue. It ceases to exist.
            if capsule.header.capsule_id != 0 {
                self.next_queue.push(capsule);
            }
        }

        self.next_queue.append(&mut birth_queue);
    }

    fn step_capsule(&mut self, capsule: &mut Capsule, birth_queue: &mut Vec<Capsule>) {
        // 1. ORACLE LOGIC (DIGESTION)
        if capsule.header.capsule_id == 5 {
            if !capsule.payload.is_empty() {
                if let Ok(msg) = String::from_utf8(capsule.payload.clone()) {
                    let response: String = msg.chars().rev().collect();
                    println!("VM [Oracle]: Digested '{}' -> '{}'", msg, response);
                    self.output_buffer.push(response);
                }
                // --- THE KILL SWITCH ---
                // We mark this capsule as "Void" (ID 0).
                // The main loop will see ID 0 and delete it.
                capsule.header.capsule_id = 0;
            }
            return;
        }

        // 2. SYSTEM CHECK
        if capsule.header.flags >= 5 && capsule.header.flags <= 7 {
            return;
        }

        // 3. DEAD BRICK CHECK
        if capsule.header.capsule_id != 777
            && capsule.header.capsule_id > 100
            && capsule.payload.is_empty()
        {
            return;
        }

        // 4. STANDARD EXECUTION
        if !capsule.payload.is_empty() {
            let op_byte = capsule.payload.remove(0);

            if let Some(op) = OpCode::from_u8(op_byte) {
                match op {
                    OpCode::NOOP => {}
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
                        if capsule.header.capsule_id == 777 || capsule.header.capsule_id == 1 {
                            let mut child = capsule.clone();
                            child.payload.clear();
                            child.header.capsule_id = self.next_id;

                            let grid_size: i32 = 16;
                            let index = (self.next_id - 1000) as i32;
                            self.next_id += 1;

                            let x = index % grid_size;
                            let y = (index / grid_size) % grid_size;
                            let z = index / (grid_size * grid_size);
                            let offset = grid_size / 2;

                            child.header.coord_x = (x - offset) as i16;
                            child.header.coord_y = (y - offset) as i16;
                            child.header.coord_z = (z - offset) as i16;
                            child.header.priority = 0;

                            let dist = ((x - offset).abs()
                                + (y - offset).abs()
                                + (z - offset).abs()) as u32;
                            if dist < 4 {
                                child.header.flags = 1;
                            } else if dist < 8 {
                                child.header.flags = 2;
                            } else {
                                child.header.flags = 3;
                            }

                            birth_queue.push(child);
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
