//! Measure runtime of find-p

use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use std::iter;

#[allow(dead_code)]
#[path = "../examples/find-p.rs"]
mod find_p;

fn bench_find_p(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_p");

    // find 9-bit irreducible polynomials
    let mut irreducibles = iter::repeat_with(|| find_p::irreducibles(9)).flatten();
    group.bench_function("find_irreducibles_9", |b| b.iter(
        || irreducibles.next().unwrap(),
    ));

    // find 8-bit generators
    let polynomial = irreducibles.next().unwrap();
    let mut generators = iter::repeat_with(|| find_p::generators(polynomial)).flatten();
    group.bench_function("find_generators_8", |b| b.iter(
        || generators.next().unwrap(),
    ));


    // find 17-bit irreducible polynomials
    let mut irreducibles = iter::repeat_with(|| find_p::irreducibles(17)).flatten();
    group.bench_function("find_irreducibles_17", |b| b.iter(
        || irreducibles.next().unwrap(),
    ));

    // find 16-bit generators
    let polynomial = irreducibles.next().unwrap();
    let mut generators = iter::repeat_with(|| find_p::generators(polynomial)).flatten();
    group.bench_function("find_generators_16", |b| b.iter(
        || generators.next().unwrap(),
    ));

    // find 33-bit irreducible polynomials
    let mut irreducibles = iter::repeat_with(|| find_p::irreducibles(33)).flatten();
    group.bench_function("find_irreducibles_33", |b| b.iter(
        || irreducibles.next().unwrap(),
    ));

    // find 32-bit generators
    let polynomial = irreducibles.next().unwrap();
    let mut generators = iter::repeat_with(|| find_p::generators(polynomial)).flatten();
    group.bench_function("find_generators_32", |b| b.iter(
        || generators.next().unwrap(),
    ));
}

criterion_group!(benches, bench_find_p);
criterion_main!(benches);
