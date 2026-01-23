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
}

impl LatticeVM {
    pub fn new(id: String) -> Self {
        let mut vm = Self {
            active_queue: Vec::new(),
            next_queue: Vec::new(),
            cycle_count: 0,
            registers: [0; 4],
            next_id: 1000, // Reserve 0-999 for System
            universe_id: id,
        };

        // AUTO-GENESIS
        vm.genesis();
        vm
    }

    pub fn genesis(&mut self) {
        // --- MACF PROTOCOL: THE TRINITY ---
        // MOVED TO "DEEP NEGATIVE SPACE" (X = -12) to avoid collision with User Cube (-8 to +7)

        // 1. THE QUEEN (Active) - Gold
        self.spawn_system_node(1, -12, 0, 0, 4, true);

        // 2. PRINCE A (Standby) - Silver (Above)
        self.spawn_system_node(2, -12, 2, 0, 5, false);

        // 3. PRINCE B (Standby) - Silver (Below)
        self.spawn_system_node(3, -12, -2, 0, 5, false);

        // 4. THE SCEPTER (Shared Memory) - Cyan (Front)
        self.spawn_system_node(4, -11, 0, 0, 6, false);

        println!("> [MACF] Trinity Protocol Initiated in Deep Negative Space (-12).");
    }

    fn spawn_system_node(&mut self, id: u32, x: i16, y: i16, z: i16, flag: u16, active: bool) {
        let payload = if active { vec![7u8; 100] } else { vec![] };
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

    // ... (Save/Load functions unchanged) ...
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
                .coord_z
                .cmp(&b.header.coord_z)
                .then(a.header.coord_y.cmp(&b.header.coord_y))
                .then(a.header.coord_x.cmp(&b.header.coord_x))
        });

        let mut birth_queue: Vec<Capsule> = Vec::new();

        for i in 0..self.active_queue.len() {
            let mut capsule = self.active_queue[i].clone();
            self.step_capsule(&mut capsule, &mut birth_queue);
            self.next_queue.push(capsule);
        }

        self.next_queue.append(&mut birth_queue);
    }

    fn step_capsule(&mut self, capsule: &mut Capsule, birth_queue: &mut Vec<Capsule>) {
        // SYSTEM CHECK (Standby Nodes Sleep)
        if capsule.header.flags == 5 || capsule.header.flags == 6 {
            return;
        }

        // Standard Logic (Dead Bricks Sleep)
        if capsule.header.capsule_id != 777
            && capsule.header.capsule_id > 100
            && capsule.payload.is_empty()
        {
            return;
        }

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
                        if capsule.header.capsule_id == 777 {
                            let mut child = capsule.clone();
                            child.payload.clear();
                            child.header.capsule_id = self.next_id;

                            // --- FIX: NORMALIZE INDEX ---
                            // We subtract 1000 so the first user cell maps to Index 0.
                            // This ensures the cube starts filling from the true corner.
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

                            // COLOR GENE
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
