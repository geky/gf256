//! Let compare various gf(256) division implementations

use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use criterion::BatchSize;
use std::iter;
use ::gf256::*;
use ::gf256::macros::gf;


// generate explicit barret and table implementations
#[gf(polynomial=0x11d, generator=0x02, table)]
type gf256_table;
#[gf(polynomial=0x11d, generator=0x02, barret)]
type gf256_barret;

fn naive_gfdiv(a: u8, b: u8) -> u8 {
    u8::from(gf256(a).naive_div(gf256(b)))
}

fn table_gfdiv(a: u8, b: u8) -> u8 {
    u8::from(gf256_table(a) / gf256_table(b))
}

fn barret_gfdiv(a: u8, b: u8) -> u8 {
    u8::from(gf256_barret(a) / gf256_barret(b))
}


fn bench_gfdiv(c: &mut Criterion) {
    let mut group = c.benchmark_group("gfdiv");

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

    let mut xs = xorshift64(42).map(|x| x as u8);
    let mut ys = xorshift64(42*42).map(|y| y as u8).filter(|y| *y != 0);
    group.bench_function("naive_gfdiv", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| naive_gfdiv(x, y),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| x as u8);
    let mut ys = xorshift64(42*42).map(|y| y as u8).filter(|y| *y != 0);
    group.bench_function("table_gfdiv", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| table_gfdiv(x, y),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| x as u8);
    let mut ys = xorshift64(42*42).map(|y| y as u8).filter(|y| *y != 0);
    group.bench_function("barret_gfdiv", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| barret_gfdiv(x, y),
        BatchSize::SmallInput
    ));
}

criterion_group!(benches, bench_gfdiv);
criterion_main!(benches);
