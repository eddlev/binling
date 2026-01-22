// Only compile the imports if "cli-mode" is enabled
#[cfg(feature = "cli-mode")]
use binling_core::capsules::{Capsule, CapsuleHeader, SquareSpace};
#[cfg(feature = "cli-mode")]
use binling_core::codec::LatticeCodec;
#[cfg(feature = "cli-mode")]
use criterion::{black_box, criterion_group, criterion_main, Criterion};

// --- REAL BENCHMARK (Only active in CLI Mode) ---

#[cfg(feature = "cli-mode")]
fn benchmark_codec(c: &mut Criterion) {
    // 1. Setup
    let capsule = Capsule {
        header: CapsuleHeader {
            magic: *b"BLE1",
            version_major: 0,
            version_minor: 1,
            flags: 0,
            ss_n: SquareSpace::SS64,
            priority: 10,
            header_len: 122,
            policy_len: 0,
            payload_len: 0,
            pad_len: 0,
            coord_x: 100,
            coord_y: 200,
            coord_z: 300,
            capsule_id: 999,
            dict_hash: [1; 32],
            policy_core_hash: [2; 32],
            capsule_hash: [3; 32],
        },
        policy_core: vec![0; 64],
        payload: vec![0; 128],
    };

    // Pre-encode for the decode test
    let encoded_bytes = LatticeCodec::encode(&capsule).unwrap();

    // 2. Benchmark Encoding
    c.bench_function("codec_encode", |b| {
        b.iter(|| LatticeCodec::encode(black_box(&capsule)))
    });

    // 3. Benchmark Decoding
    c.bench_function("codec_decode", |b| {
        b.iter(|| LatticeCodec::decode(black_box(&encoded_bytes)))
    });
}

// Register the group ONLY if feature is on
#[cfg(feature = "cli-mode")]
criterion_group!(benches, benchmark_codec);

#[cfg(feature = "cli-mode")]
criterion_main!(benches);

// --- DUMMY MAIN (Active in WASM/Default Mode) ---
// This prevents "Main function not found" or Import errors when feature is off.

#[cfg(not(feature = "cli-mode"))]
fn main() {
    println!("NOTICE: Codec benchmark skipped. Requires 'cli-mode' feature.");
}
