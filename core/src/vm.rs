use crate::capsules::Capsule;
// Remove unused imports if any, keeping it clean for now

// The Levin Lattice VM (Spec v0.1 Section 2)
pub struct LatticeVM {
    // The Active Queue: Capsules executing in the CURRENT cycle
    pub active_queue: Vec<Capsule>,

    // The Next Queue: Capsules scheduled for the NEXT cycle
    pub next_queue: Vec<Capsule>,

    // Global Clock
    pub cycle_count: u64,
}

impl LatticeVM {
    pub fn new() -> Self {
        Self {
            active_queue: Vec::new(),
            next_queue: Vec::new(),
            cycle_count: 0,
        }
    }

    // Spec v0.1 Section 2.4: Activation Primitive
    // Places a capsule into the Next Queue (never the current one)
    pub fn activate(&mut self, capsule: Capsule) {
        self.next_queue.push(capsule);
    }

    // Spec v0.1 Section 2.5: Queue Swap & Sort
    // Moves Next -> Active, increments cycle, and sorts by Priority
    pub fn next_cycle(&mut self) {
        // 1. Swap Queues: Next becomes Active, Next becomes empty
        // std::mem::take replaces the target with the default value (empty vec) and returns the old value
        self.active_queue = std::mem::take(&mut self.next_queue);

        // 2. Increment Clock
        self.cycle_count += 1;

        // 3. Deterministic Sort (Spec v0.1 Section 2.5)
        // Primary: Priority (Ascending, 0 is highest)
        // Tie-break: Coordinate Z, then Y, then X (Lexicographic)
        self.active_queue.sort_by(|a, b| {
            a.header
                .priority
                .cmp(&b.header.priority)
                .then(a.header.coord_z.cmp(&b.header.coord_z))
                .then(a.header.coord_y.cmp(&b.header.coord_y))
                .then(a.header.coord_x.cmp(&b.header.coord_x))
        });
    }

    // Helper to see if we hit the VOID boundary (Spec v0.1 Section 2.7)
    pub fn is_void(&self) -> bool {
        self.active_queue.is_empty() && self.next_queue.is_empty()
    }
}
