//! Let compare various gf(256) multiplication implementations

use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use criterion::BatchSize;
use std::iter;
use ::gf256::macros::gf;


// generate explicit naive barret and table implementations
#[gf(polynomial=0x11d, generator=0x02, naive)]
type gf256_naive;
#[gf(polynomial=0x11d, generator=0x02, table)]
type gf256_table;
#[gf(polynomial=0x11d, generator=0x02, barret)]
type gf256_barret;

#[gf(polynomial=0x13, generator=0x2, naive)]
type gf16_naive;
#[gf(polynomial=0x13, generator=0x2, table)]
type gf16_table;
#[gf(polynomial=0x13, generator=0x2, barret)]
type gf16_barret;

#[gf(polynomial=0x1002b, generator=0x0003, naive)]
type gf2p16_naive;
#[gf(polynomial=0x1002b, generator=0x0003, barret)]
type gf2p16_barret;

#[gf(polynomial=0x10000008d, generator=0x03, naive)]
type gf2p32_naive;
#[gf(polynomial=0x10000008d, generator=0x03, barret)]
type gf2p32_barret;

#[gf(polynomial=0x1000000000000001b, generator=0x02, naive)]
type gf2p64_naive;
#[gf(polynomial=0x1000000000000001b, generator=0x02, barret)]
type gf2p64_barret;


fn bench_gfmul(c: &mut Criterion) {
    let mut group = c.benchmark_group("gfmul");

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

    // gf256 mul/div
    let mut xs = xorshift64(42).map(|x| x as u8);
    let mut ys = xorshift64(42*42).map(|y| y as u8);
    group.bench_function("gf256_naive_mul", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| gf256_naive(x) * gf256_naive(y),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| x as u8);
    let mut ys = xorshift64(42*42).map(|y| y as u8);
    group.bench_function("gf256_table_mul", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| gf256_table(x) * gf256_table(y),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| x as u8);
    let mut ys = xorshift64(42*42).map(|y| y as u8);
    group.bench_function("gf256_barret_mul", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| gf256_barret(x) * gf256_barret(y),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| x as u8);
    let mut ys = xorshift64(42*42).map(|y| y as u8).filter(|y| *y != 0);
    group.bench_function("gf256_naive_div", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| gf256_naive(x) / gf256_naive(y),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| x as u8);
    let mut ys = xorshift64(42*42).map(|y| y as u8).filter(|y| *y != 0);
    group.bench_function("gf256_table_div", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| gf256_table(x) / gf256_table(y),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| x as u8);
    let mut ys = xorshift64(42*42).map(|y| y as u8).filter(|y| *y != 0);
    group.bench_function("gf256_barret_div", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| gf256_barret(x) / gf256_barret(y),
        BatchSize::SmallInput
    ));

    // gf16 mul/div
    let mut xs = xorshift64(42).map(|x| (x&0xf) as u8);
    let mut ys = xorshift64(42*42).map(|y| (y&0xf) as u8);
    group.bench_function("gf16_naive_mul", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| gf16_naive(x) * gf16_naive(y),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| (x&0xf) as u8);
    let mut ys = xorshift64(42*42).map(|y| (y&0xf) as u8);
    group.bench_function("gf16_table_mul", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| gf16_table(x) * gf16_table(y),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| (x&0xf) as u8);
    let mut ys = xorshift64(42*42).map(|y| (y&0xf) as u8);
    group.bench_function("gf16_barret_mul", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| gf16_barret(x) * gf16_barret(y),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| (x&0xf) as u8);
    let mut ys = xorshift64(42*42).map(|y| (y&0xf) as u8).filter(|y| *y != 0);
    group.bench_function("gf16_naive_div", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| gf16_naive(x) / gf16_naive(y),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| (x&0xf) as u8);
    let mut ys = xorshift64(42*42).map(|y| (y&0xf) as u8).filter(|y| *y != 0);
    group.bench_function("gf16_table_div", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| gf16_table(x) / gf16_table(y),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| (x&0xf) as u8);
    let mut ys = xorshift64(42*42).map(|y| (y&0xf) as u8).filter(|y| *y != 0);
    group.bench_function("gf16_barret_div", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| gf16_barret(x) / gf16_barret(y),
        BatchSize::SmallInput
    ));

    // gf2p16 mul/div
    let mut xs = xorshift64(42).map(|x| x as u16);
    let mut ys = xorshift64(42*42).map(|y| y as u16);
    group.bench_function("gf2p16_naive_mul", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| gf2p16_naive(x) * gf2p16_naive(y),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| x as u16);
    let mut ys = xorshift64(42*42).map(|y| y as u16);
    group.bench_function("gf2p16_barret_mul", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| gf2p16_barret(x) * gf2p16_barret(y),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| x as u16);
    let mut ys = xorshift64(42*42).map(|y| y as u16).filter(|y| *y != 0);
    group.bench_function("gf2p16_naive_div", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| gf2p16_naive(x) / gf2p16_naive(y),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| x as u16);
    let mut ys = xorshift64(42*42).map(|y| y as u16).filter(|y| *y != 0);
    group.bench_function("gf2p16_barret_div", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| gf2p16_barret(x) / gf2p16_barret(y),
        BatchSize::SmallInput
    ));

    // gf2p32 mul/div
    let mut xs = xorshift64(42).map(|x| x as u32);
    let mut ys = xorshift64(42*42).map(|y| y as u32);
    group.bench_function("gf2p32_naive_mul", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| gf2p32_naive(x) * gf2p32_naive(y),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| x as u32);
    let mut ys = xorshift64(42*42).map(|y| y as u32);
    group.bench_function("gf2p32_barret_mul", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| gf2p32_barret(x) * gf2p32_barret(y),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| x as u32);
    let mut ys = xorshift64(42*42).map(|y| y as u32).filter(|y| *y != 0);
    group.bench_function("gf2p32_naive_div", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| gf2p32_naive(x) / gf2p32_naive(y),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| x as u32);
    let mut ys = xorshift64(42*42).map(|y| y as u32).filter(|y| *y != 0);
    group.bench_function("gf2p32_barret_div", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| gf2p32_barret(x) / gf2p32_barret(y),
        BatchSize::SmallInput
    ));

    // gf2p64 mul/div
    let mut xs = xorshift64(42).map(|x| x as u64);
    let mut ys = xorshift64(42*42).map(|y| y as u64);
    group.bench_function("gf2p64_naive_mul", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| gf2p64_naive(x) * gf2p64_naive(y),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| x as u64);
    let mut ys = xorshift64(42*42).map(|y| y as u64);
    group.bench_function("gf2p64_barret_mul", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| gf2p64_barret(x) * gf2p64_barret(y),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| x as u64);
    let mut ys = xorshift64(42*42).map(|y| y as u64).filter(|y| *y != 0);
    group.bench_function("gf2p64_naive_div", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| gf2p64_naive(x) / gf2p64_naive(y),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| x as u64);
    let mut ys = xorshift64(42*42).map(|y| y as u64).filter(|y| *y != 0);
    group.bench_function("gf2p64_barret_div", |b| b.iter_batched(
        || (xs.next().unwrap(), ys.next().unwrap()),
        |(x, y)| gf2p64_barret(x) / gf2p64_barret(y),
        BatchSize::SmallInput
    ));
}

criterion_group!(benches, bench_gfmul);
criterion_main!(benches);
