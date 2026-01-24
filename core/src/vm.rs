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
    #[serde(skip)]
    pub pending_writes: Vec<(i16, i16, i16, usize, u8)>,
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
            pending_writes: Vec::new(),
        };
        vm.genesis();
        vm
    }

    pub fn genesis(&mut self) {
        println!("> [INIT] Constructing Star Fortress Architecture...");
        self.spawn_node(1, 0, 0, 0, 1);
        self.spawn_node(5, -10, 0, 0, 7);

        let mut struct_id = 100;
        let arm_length = 8;
        for i in 1..=arm_length {
            self.spawn_node(struct_id, i, 0, 0, 2);
            struct_id += 1;
            self.spawn_node(struct_id, -i, 0, 0, 2);
            struct_id += 1;
            self.spawn_node(struct_id, 0, i, 0, 2);
            struct_id += 1;
            self.spawn_node(struct_id, 0, -i, 0, 2);
            struct_id += 1;
            self.spawn_node(struct_id, 0, 0, i, 2);
            struct_id += 1;
            self.spawn_node(struct_id, 0, 0, -i, 2);
            struct_id += 1;
        }
        println!(
            "> [SYSTEM] Star Fortress Online. Nodes: {}",
            self.next_queue.len()
        );
    }

    fn spawn_node(&mut self, id: u32, x: i16, y: i16, z: i16, flag: u16) {
        let payload = vec![0u8; 64];
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
        let snapshot = self.active_queue.clone();

        for i in 0..self.active_queue.len() {
            let mut capsule = self.active_queue[i].clone();
            // REMOVED: dna_backup logic. We want live mutation.
            self.step_capsule(&mut capsule, &snapshot, &mut birth_queue);
            if capsule.header.capsule_id != 0 {
                self.next_queue.push(capsule);
            }
        }

        while let Some((tx, ty, tz, idx, val)) = self.pending_writes.pop() {
            if let Some(target) = self.next_queue.iter_mut().find(|c| {
                c.header.coord_x == tx && c.header.coord_y == ty && c.header.coord_z == tz
            }) {
                if target.payload.len() <= idx {
                    target.payload.resize(idx + 1, 0);
                }
                target.payload[idx] = val;
            }
        }
        self.next_queue.append(&mut birth_queue);
    }

    // REMOVED: dna argument
    fn step_capsule(
        &mut self,
        capsule: &mut Capsule,
        snapshot: &Vec<Capsule>,
        birth_queue: &mut Vec<Capsule>,
    ) {
        if capsule.header.capsule_id == 5 {
            if !capsule.payload.is_empty() {
                if let Ok(msg) = String::from_utf8(capsule.payload.clone()) {
                    let response: String = msg.chars().rev().collect();
                    self.output_buffer.push(response);
                }
                capsule.header.capsule_id = 0;
            }
            return;
        }

        if capsule.header.flags >= 5 && capsule.header.flags <= 7 {
            return;
        }
        if capsule.header.capsule_id > 200
            && capsule.header.capsule_id != 999
            && capsule.payload.is_empty()
        {
            return;
        }

        let mut ip = capsule.header.pad_len as usize;

        if ip < capsule.payload.len() {
            let op_byte = capsule.payload[ip];
            if op_byte == 0 {
                return;
            }

            ip += 1;

            if let Some(op) = OpCode::from_u8(op_byte) {
                match op {
                    OpCode::NOOP => {}
                    OpCode::HALT => {
                        capsule.payload.clear();
                        ip = 0;
                    }
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

                    OpCode::STORE => {
                        if ip + 4 <= capsule.payload.len() {
                            let dx = capsule.payload[ip] as i8;
                            let dy = capsule.payload[ip + 1] as i8;
                            let dz = capsule.payload[ip + 2] as i8;
                            let idx = capsule.payload[ip + 3] as usize;
                            ip += 4;

                            let tx = capsule.header.coord_x + dx as i16;
                            let ty = capsule.header.coord_y + dy as i16;
                            let tz = capsule.header.coord_z + dz as i16;
                            let val = (self.registers[0] & 0xFF) as u8;

                            // --- NEW: IMMEDIATE LOCAL WRITE ---
                            if dx == 0 && dy == 0 && dz == 0 {
                                if capsule.payload.len() <= idx {
                                    capsule.payload.resize(idx + 1, 0);
                                }
                                capsule.payload[idx] = val;
                            }
                            // ----------------------------------

                            self.pending_writes.push((tx, ty, tz, idx, val));
                        }
                    }

                    OpCode::LOAD => {
                        if ip + 4 <= capsule.payload.len() {
                            let dx = capsule.payload[ip] as i8;
                            let dy = capsule.payload[ip + 1] as i8;
                            let dz = capsule.payload[ip + 2] as i8;
                            let idx = capsule.payload[ip + 3] as usize;
                            ip += 4;

                            let tx = capsule.header.coord_x + dx as i16;
                            let ty = capsule.header.coord_y + dy as i16;
                            let tz = capsule.header.coord_z + dz as i16;

                            if let Some(target) = snapshot.iter().find(|c| {
                                c.header.coord_x == tx
                                    && c.header.coord_y == ty
                                    && c.header.coord_z == tz
                            }) {
                                if idx < target.payload.len() {
                                    self.registers[0] = target.payload[idx] as i32;
                                } else {
                                    self.registers[0] = 0;
                                }
                            }
                        }
                    }

                    OpCode::JMP => {
                        if ip < capsule.payload.len() {
                            let target_idx = capsule.payload[ip] as usize;
                            ip = target_idx;
                        }
                    }

                    OpCode::BEQ => {
                        if ip + 2 <= capsule.payload.len() {
                            let check_val = capsule.payload[ip] as i32;
                            let target_idx = capsule.payload[ip + 1] as usize;
                            ip += 2;
                            if self.registers[0] == check_val {
                                ip = target_idx;
                            }
                        }
                    }

                    OpCode::REPL => {
                        if ip + 3 <= capsule.payload.len() {
                            let dx = capsule.payload[ip] as i8;
                            let dy = capsule.payload[ip + 1] as i8;
                            let dz = capsule.payload[ip + 2] as i8;
                            ip += 3;

                            let tx = capsule.header.coord_x + dx as i16;
                            let ty = capsule.header.coord_y + dy as i16;
                            let tz = capsule.header.coord_z + dz as i16;

                            let mut clone = capsule.clone();
                            clone.header.coord_x = tx;
                            clone.header.coord_y = ty;
                            clone.header.coord_z = tz;
                            clone.header.capsule_id = self.next_id;
                            self.next_id += 1;
                            clone.header.pad_len = 0;
                            // CHANGED: Use the CURRENT payload, not the old backup
                            clone.payload = capsule.payload.clone();

                            birth_queue.push(clone);
                            println!("VM [REPL]: Replicated to ({},{},{})", tx, ty, tz);
                        }
                    }

                    OpCode::SPAWN => { /* Omitted */ }
                }
            }
        }
        capsule.header.pad_len = ip as u32;
    }

    pub fn is_void(&self) -> bool {
        self.active_queue.is_empty() && self.next_queue.is_empty()
    }
}
