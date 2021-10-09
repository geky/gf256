//! Let compare naive carry-less multiplication vs hardware accelerated
//! carry-less multiplication

use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use criterion::BatchSize;
use std::iter;
use gf256::*;

fn naive_xmul(a: p64, b: p64) -> p64 {
    a.wrapping_naive_mul(b)
}

fn hardware_xmul(a: p64, b: p64) -> p64 {
    a.wrapping_mul(b)
}

fn bench_mul(c: &mut Criterion) {
    let mut group = c.benchmark_group("xmul");

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

    // naive xmul
    let mut xs = xorshift64(42).map(p64);
    let mut ys = xorshift64(42*42).map(p64);
    group.bench_function("naive_xmul", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| naive_xmul(x, y),
        BatchSize::SmallInput
    ));

    // hardware accelerated xmul (leveraging pclmulqdq, pmul, etc)
    let mut xs = xorshift64(42).map(p64);
    let mut ys = xorshift64(42*42).map(p64);
    group.bench_function("hardware_xmul", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| hardware_xmul(x, y),
        BatchSize::SmallInput
    ));
}

criterion_group!(benches, bench_mul);
criterion_main!(benches);
