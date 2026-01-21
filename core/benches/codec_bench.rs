use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_placeholder(c: &mut Criterion) {
    // We will implement real benchmarks here later
    c.bench_function("placeholder", |b| b.iter(|| 1 + 1));
}

criterion_group!(benches, benchmark_placeholder);
criterion_main!(benches);
