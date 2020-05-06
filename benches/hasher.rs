use criterion::{criterion_group, criterion_main, Criterion};
use flexihash::*;

fn all(c: &mut Criterion) {
    c.bench_function("crc32", |b| {
        b.iter(|| hash(&Hasher::Crc32, String::from("test")))
    });
    c.bench_function("md5", |b| {
        b.iter(|| hash(&Hasher::Md5, String::from("test")))
    });
}

criterion_group!(benches, all);
criterion_main!(benches);
