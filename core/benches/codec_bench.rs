use binling_core::capsules::{Capsule, CapsuleHeader, SquareSpace};
use binling_core::codec::LatticeCodec;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_codec(c: &mut Criterion) {
    // 1. Setup: Create a capsule to test
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
        policy_core: vec![0; 64], // 64 bytes of policy
        payload: vec![0; 128],    // 128 bytes of payload
    };

    // Pre-encode for the decode test
    let encoded_bytes = LatticeCodec::encode(&capsule).unwrap();

    // 2. Benchmark Encoding (Capsule -> Bytes)
    c.bench_function("codec_encode", |b| {
        b.iter(|| LatticeCodec::encode(black_box(&capsule)))
    });

    // 3. Benchmark Decoding (Bytes -> Capsule)
    c.bench_function("codec_decode", |b| {
        b.iter(|| LatticeCodec::decode(black_box(&encoded_bytes)))
    });
}

criterion_group!(benches, benchmark_codec);
criterion_main!(benches);
