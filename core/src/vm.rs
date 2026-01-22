use crate::capsules::Capsule;
use crate::instructions::OpCode;

// The Levin Lattice VM (Spec v0.1 Section 2)
// 1 implementation
pub struct LatticeVM {
    // The Active Queue: Capsules executing in the CURRENT cycle
    pub active_queue: Vec<Capsule>,

    // The Next Queue: Capsules scheduled for the NEXT cycle
    pub next_queue: Vec<Capsule>,

    // Global Clock
    pub cycle_count: u64,

    // NEW: 4 General Purpose Registers (R0, R1, R2, R3)
    // This is the VM's "Short Term Memory"
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
        // We create a "Birth Queue" to hold babies created during this cycle
        // This avoids fighting the Borrow Checker over 'self.next_queue'
        let mut birth_queue: Vec<Capsule> = Vec::new();

        for i in 0..self.active_queue.len() {
            // Clone the capsule to execute it
            let mut capsule = self.active_queue[i].clone();

            // Pass the birth_queue into the execution environment
            self.execute_capsule(&mut capsule, &mut birth_queue);
        }

        // 5. Move babies to the Next Queue (End of Cycle)
        self.next_queue.append(&mut birth_queue);
    }

    // The Interpreter Loop (Now with Reproduction capabilities)
    // NOTICE: We now accept 'birth_queue' as an argument
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
                        // A. Clone the Mother
                        let mut daughter = capsule.clone();

                        // B. Mutate Identity (New ID = Old ID + 1000 for visibility)
                        daughter.header.capsule_id += 1000;

                        // C. Log the miracle
                        println!(
                            ">> [MITOSIS] Cap {} spawned Cap {}!",
                            capsule.header.capsule_id, daughter.header.capsule_id
                        );

                        // D. Place into the Birth Queue
                        birth_queue.push(daughter);
                    }
                }
            }
        }
    }

    // Helper to see if we hit the VOID boundary
    pub fn is_void(&self) -> bool {
        self.active_queue.is_empty() && self.next_queue.is_empty()
    }
}
