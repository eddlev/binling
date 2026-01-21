use crate::capsules::Capsule;
use crate::instructions::OpCode;
// Remove unused imports if any, keeping it clean for now

// The Levin Lattice VM (Spec v0.1 Section 2)
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
        // We iterate through all active capsules and run their code
        // We use a separate index to avoid borrowing issues while mutating the VM
        for i in 0..self.active_queue.len() {
            // Clone the capsule to execute it (simplification for v0.1)
            let mut capsule = self.active_queue[i].clone();
            self.execute_capsule(&mut capsule);
            // In a real version, we might update the capsule state here
        }
    }

    // The Interpreter Loop (Spec v0.1 Section 3)
    fn execute_capsule(&mut self, capsule: &mut Capsule) {
        let mut pc = 0; // Program Counter (Instruction Pointer)

        while pc < capsule.payload.len() {
            let op_byte = capsule.payload[pc];
            pc += 1;

            // Decode Byte -> OpCode
            if let Some(op) = OpCode::from_u8(op_byte) {
                match op {
                    OpCode::NOOP => {}     // Do nothing
                    OpCode::HALT => break, // Stop executing this capsule

                    // Arithmetic (Metabolism)
                    OpCode::ADD => {
                        // R0 = R0 + R1
                        self.registers[0] = self.registers[0].wrapping_add(self.registers[1]);
                    }
                    OpCode::SUB => {
                        self.registers[0] = self.registers[0].wrapping_sub(self.registers[1]);
                    }
                    OpCode::INC => {
                        // R0++
                        self.registers[0] = self.registers[0].wrapping_add(1);
                    }
                    OpCode::DEC => {
                        // R0--
                        self.registers[0] = self.registers[0].wrapping_sub(1);
                    }

                    // I/O (Senses)
                    OpCode::LOG => {
                        println!(
                            "VM [Cycle {} | Cap {}]: R0 = {}",
                            self.cycle_count, capsule.header.capsule_id, self.registers[0]
                        );
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
