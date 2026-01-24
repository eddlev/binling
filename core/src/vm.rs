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
    pub pending_writes: Vec<(i16, i16, i16, u8)>,
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
        // TRINITY + ORACLE
        self.spawn_system_node(1, -12, 0, 0, 4, true);
        self.spawn_system_node(2, -12, 2, 0, 5, false);
        self.spawn_system_node(3, -12, -2, 0, 5, false);
        self.spawn_system_node(4, -11, 0, 0, 6, false);
        self.spawn_system_node(5, -10, 0, 0, 7, false);
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
            // We pass a clone of the payload to step_capsule so it can 'REPL' itself
            let dna_backup = capsule.payload.clone();
            self.step_capsule(&mut capsule, &snapshot, &mut birth_queue, &dna_backup);

            if capsule.header.capsule_id != 0 {
                self.next_queue.push(capsule);
            }
        }

        while let Some((tx, ty, tz, val)) = self.pending_writes.pop() {
            if let Some(target) = self.next_queue.iter_mut().find(|c| {
                c.header.coord_x == tx && c.header.coord_y == ty && c.header.coord_z == tz
            }) {
                target.payload = vec![val];
            }
        }

        self.next_queue.append(&mut birth_queue);
    }

    fn step_capsule(
        &mut self,
        capsule: &mut Capsule,
        snapshot: &Vec<Capsule>,
        birth_queue: &mut Vec<Capsule>,
        dna: &Vec<u8>,
    ) {
        // 1. ORACLE
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
        if capsule.header.capsule_id != 777
            && capsule.header.capsule_id != 999
            && capsule.header.capsule_id > 100
            && capsule.payload.is_empty()
        {
            return;
        }

        // 3. ETERNAL EXECUTION
        let mut ip = capsule.header.pad_len as usize;

        if ip < capsule.payload.len() {
            let op_byte = capsule.payload[ip];
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
                        if ip + 3 <= capsule.payload.len() {
                            let dx = capsule.payload[ip] as i8;
                            let dy = capsule.payload[ip + 1] as i8;
                            let dz = capsule.payload[ip + 2] as i8;
                            ip += 3;
                            let tx = capsule.header.coord_x + dx as i16;
                            let ty = capsule.header.coord_y + dy as i16;
                            let tz = capsule.header.coord_z + dz as i16;
                            let val = (self.registers[0] & 0xFF) as u8;
                            self.pending_writes.push((tx, ty, tz, val));
                            // println!("VM [STORE]: Queued write {} to ({},{},{})", val, tx, ty, tz);
                        }
                    }

                    OpCode::LOAD => {
                        if ip + 3 <= capsule.payload.len() {
                            let dx = capsule.payload[ip] as i8;
                            let dy = capsule.payload[ip + 1] as i8;
                            let dz = capsule.payload[ip + 2] as i8;
                            ip += 3;
                            let tx = capsule.header.coord_x + dx as i16;
                            let ty = capsule.header.coord_y + dy as i16;
                            let tz = capsule.header.coord_z + dz as i16;

                            if let Some(target) = snapshot.iter().find(|c| {
                                c.header.coord_x == tx
                                    && c.header.coord_y == ty
                                    && c.header.coord_z == tz
                            }) {
                                if !target.payload.is_empty() {
                                    self.registers[0] = target.payload[0] as i32;
                                    // println!("VM [LOAD]: Read {} from ({},{},{})", self.registers[0], tx, ty, tz);
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

                    // --- THE REPLICATOR (PHASE 4) ---
                    OpCode::REPL => {
                        // REPL [dx] [dy] [dz] -> Copies SELF to Target
                        if ip + 3 <= capsule.payload.len() {
                            let dx = capsule.payload[ip] as i8;
                            let dy = capsule.payload[ip + 1] as i8;
                            let dz = capsule.payload[ip + 2] as i8;
                            ip += 3;

                            let tx = capsule.header.coord_x + dx as i16;
                            let ty = capsule.header.coord_y + dy as i16;
                            let tz = capsule.header.coord_z + dz as i16;

                            // We need to write into the FUTURE (next_queue)
                            // But unlike STORE (which queues a byte), this writes a WHOLE PROGRAM
                            // We can do this directly here because we hold mutable ref to self.
                            // NOTE: This runs immediately, not at end of cycle.
                            // We find the target in next_queue and INFECT it.

                            // To avoid borrow checker hell (we are iterating active_queue elsewhere),
                            // we scan `birth_queue`? No. We need to push a NEW capsule to birth_queue?
                            // Or find the existing cell in next_queue?

                            // Simplest way: Birth a new capsule that OVERWRITES the location.
                            // The VM loop will sort it out (last one wins?).
                            // Better: We actually want to overwrite the existing cell structure.

                            // Let's use the 'birth_queue' to spawn a CLONE at that location.
                            let mut clone = capsule.clone();
                            clone.header.coord_x = tx;
                            clone.header.coord_y = ty;
                            clone.header.coord_z = tz;
                            clone.header.capsule_id = self.next_id; // Unique ID
                            self.next_id += 1;
                            clone.header.pad_len = 0; // RESET IP (Start fresh)
                            clone.payload = dna.clone(); // Copy the pure DNA

                            birth_queue.push(clone);
                            println!("VM [REPL]: Replicated to ({},{},{})", tx, ty, tz);
                        }
                    }

                    OpCode::SPAWN => {
                        // Full Logic Restored
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
        capsule.header.pad_len = ip as u32;
    }

    pub fn is_void(&self) -> bool {
        self.active_queue.is_empty() && self.next_queue.is_empty()
    }
}
