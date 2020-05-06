use criterion::{criterion_group, criterion_main, Criterion};
use flexihash::*;

fn all(c: &mut Criterion) {
    c.bench_function("new", |b| b.iter(|| Flexihash::new()));
}

criterion_group!(benches, all);
criterion_main!(benches);
