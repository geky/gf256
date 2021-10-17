//! Lets compare various CRC implementations

use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use criterion::BatchSize;
use criterion::Throughput;
use std::iter;

#[allow(dead_code)]
#[path = "../examples/shamir.rs"]
mod shamir;

fn bench_shamir(c: &mut Criterion) {
    let mut group = c.benchmark_group("shamir");

    // xorshift64 for deterministic random numbers
    fn xorshift64(seed: u64) -> impl Iterator<Item=u64> {
        let mut x = seed;
        iter::repeat_with(move || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        })
    }

    // size to bench
    const SIZE: usize = 1024;
    const N: usize = 5;
    group.throughput(Throughput::Bytes((N*SIZE) as u64));

    // benchmark the time it takes to generate shares
    let mut xs = xorshift64(42).map(|x| x as u8);
    group.bench_function("shamir_generate", |b| b.iter_batched_ref(
        || (&mut xs).take(SIZE).collect::<Vec<u8>>(),
        |input| shamir::shamir_generate(input, N, N),
        BatchSize::SmallInput
    ));

    // benchmark the time it takes to reconstruct shares
    let mut xs = xorshift64(42).map(|x| x as u8);
    group.bench_function("shamir_reconstruct", |b| b.iter_batched_ref(
        || {
            let input = (&mut xs).take(SIZE).collect::<Vec<u8>>();
            shamir::shamir_generate(&input, N, N)
        },
        |shares| shamir::shamir_reconstruct(shares),
        BatchSize::SmallInput
    ));
}

criterion_group!(benches, bench_shamir);
criterion_main!(benches);
