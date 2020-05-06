use criterion::{criterion_group, criterion_main, Criterion};
use flexihash::*;

fn all(c: &mut Criterion) {
    let mut fh = Flexihash::new();

    fh.add_target("olive", 10);
    c.bench_function("one of one", |b| b.iter(|| fh.lookup_list("foobar", 1)));

    fh.add_target("acacia", 10);
    c.bench_function("one of two", |b| b.iter(|| fh.lookup_list("foobar", 1)));
    c.bench_function("two of two", |b| b.iter(|| fh.lookup_list("foobar", 2)));
    c.bench_function("three of two", |b| b.iter(|| fh.lookup_list("foobar", 3)));
}

criterion_group!(benches, all);
criterion_main!(benches);
