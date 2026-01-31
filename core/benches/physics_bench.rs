use criterion::{criterion_group, criterion_main, Criterion};

#[cfg(feature = "cli-mode")]
use binling_core::capsules::{Capsule, CapsuleHeader, SquareSpace};
#[cfg(feature = "cli-mode")]
use binling_core::vm::LatticeVM;

#[cfg(feature = "cli-mode")]
fn benchmark_physics_loop(c: &mut Criterion) {
    c.bench_function("vm_genesis_cycle_10", |b| {
        b.iter(|| {
            // 1. Setup VM with a dummy ID (Fixed line below)
            let mut vm = LatticeVM::new("BENCHMARK_UNIVERSE".to_string());

            // 2. Create Kernel (Simplified payload)
            let payload = vec![7u8; 10]; // 10 SPAWN instructions
            let kernel = Capsule {
                header: CapsuleHeader {
                    magic: *b"BLE1",
                    version_major: 0,
                    version_minor: 1,
                    flags: 0,
                    capsule_id: 777,
                    ss_n: SquareSpace::SS64,
                    priority: 10,
                    coord_x: 0,
                    coord_y: 0,
                    coord_z: 0,
                    header_len: 122,
                    policy_len: 0,
                    payload_len: 10,
                    pad_len: 0,
                    dict_hash: [0; 32],
                    policy_core_hash: [0; 32],
                    capsule_hash: [0; 32],
                },
                policy_core: vec![],
                payload,
            };

            // 3. Inject
            vm.activate(kernel);

            // 4. Run 10 Cycles
            for _ in 0..10 {
                vm.next_cycle();
            }
        })
    });
}

#[cfg(not(feature = "cli-mode"))]
fn benchmark_physics_loop(_c: &mut Criterion) {}

criterion_group!(benches, benchmark_physics_loop);
criterion_main!(benches);
