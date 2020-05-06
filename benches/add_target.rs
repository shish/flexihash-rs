use criterion::{criterion_group, criterion_main, Criterion};
use flexihash::*;

fn all(c: &mut Criterion) {
    c.bench_function("one", |b| {
        b.iter(|| {
            let mut fh = Flexihash::new();
            fh.add_target("olive", 10);
        })
    });
    c.bench_function("two", |b| {
        b.iter(|| {
            let mut fh = Flexihash::new();
            fh.add_target("olive", 10);
            fh.add_target("acacia", 10);
        })
    });
    c.bench_function("three", |b| {
        b.iter(|| {
            let mut fh = Flexihash::new();
            fh.add_target("olive", 10);
            fh.add_target("acacia", 10);
            fh.add_target("rose", 10);
        })
    });
    c.bench_function("many", |b| {
        b.iter(|| {
            let mut fh = Flexihash::new();
            for n in 0..10 {
                fh.add_target(format!("olive{}", n), 10);
            }
        })
    });
}

criterion_group!(benches, all);
criterion_main!(benches);
