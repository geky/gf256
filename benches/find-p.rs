//! Measure runtime of find-p

use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;

#[allow(dead_code)]
#[path = "../examples/find-p.rs"]
mod find_p;

fn bench_find_p(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_p");

    // find 33-bit irreducible polynomials
    let mut irreducibles = find_p::irreducibles(33);
    group.bench_function("find_irreducibles_33", |b| b.iter(
        || irreducibles.next().unwrap(),
    ));

    // find 32-bit generators
    let polynomial = irreducibles.next().unwrap();
    let mut generators = find_p::generators(polynomial);
    group.bench_function("find_generators_32", |b| b.iter(
        || generators.next().unwrap(),
    ));
}

criterion_group!(benches, bench_find_p);
criterion_main!(benches);
