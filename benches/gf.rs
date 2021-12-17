//! Let compare various gf(256) multiplication implementations

use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use criterion::BatchSize;
use std::iter;
use std::convert::TryFrom;
use ::gf256::gf::gf;


// generate explicit naive barret and table implementations
#[gf(polynomial=0x11d, generator=0x02, naive)]
type gf256_naive;
#[gf(polynomial=0x11d, generator=0x02, table)]
type gf256_table;
#[gf(polynomial=0x11d, generator=0x02, rem_table)]
type gf256_rem_table;
#[gf(polynomial=0x11d, generator=0x02, small_rem_table)]
type gf256_small_rem_table;
#[gf(polynomial=0x11d, generator=0x02, barret)]
type gf256_barret;

#[gf(polynomial=0x13, generator=0x2, naive)]
type gf16_naive;
#[gf(polynomial=0x13, generator=0x2, table)]
type gf16_table;
#[gf(polynomial=0x13, generator=0x2, rem_table)]
type gf16_rem_table;
#[gf(polynomial=0x13, generator=0x2, small_rem_table)]
type gf16_small_rem_table;
#[gf(polynomial=0x13, generator=0x2, barret)]
type gf16_barret;

#[gf(polynomial=0x1002b, generator=0x0003, naive)]
type gf2p16_naive;
#[gf(polynomial=0x1002b, generator=0x0003, rem_table)]
type gf2p16_rem_table;
#[gf(polynomial=0x1002b, generator=0x0003, small_rem_table)]
type gf2p16_small_rem_table;
#[gf(polynomial=0x1002b, generator=0x0003, barret)]
type gf2p16_barret;

#[gf(polynomial=0x10000008d, generator=0x03, naive)]
type gf2p32_naive;
#[gf(polynomial=0x10000008d, generator=0x03, rem_table)]
type gf2p32_rem_table;
#[gf(polynomial=0x10000008d, generator=0x03, small_rem_table)]
type gf2p32_small_rem_table;
#[gf(polynomial=0x10000008d, generator=0x03, barret)]
type gf2p32_barret;

#[gf(polynomial=0x1000000000000001b, generator=0x02, naive)]
type gf2p64_naive;
#[gf(polynomial=0x1000000000000001b, generator=0x02, rem_table)]
type gf2p64_rem_table;
#[gf(polynomial=0x1000000000000001b, generator=0x02, small_rem_table)]
type gf2p64_small_rem_table;
#[gf(polynomial=0x1000000000000001b, generator=0x02, barret)]
type gf2p64_barret;


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

macro_rules! bench_mul {
    ($group:expr, $name:expr, $gf:expr) => {
        let mut xs = xorshift64(42).map(|x| $gf(x as _));
        let mut ys = xorshift64(42*42).map(|y| $gf(y as _));
        $group.bench_function($name, |b| b.iter_batched(
            || (xs.next().unwrap(), ys.next().unwrap()),
            |(x, y)| x * y,
            BatchSize::SmallInput
        ));
    }
}

macro_rules! bench_div {
    ($group:expr, $name:expr, $gf:expr) => {
        let mut xs = xorshift64(42).map(|x| $gf(x as _));
        let mut ys = xorshift64(42*42).map(|y| $gf(y as _)).filter(|x| *x != $gf(0));
        $group.bench_function($name, |b| b.iter_batched(
            || (xs.next().unwrap(), ys.next().unwrap()),
            |(x, y)| x / y,
            BatchSize::SmallInput
        ));
    }
}


fn bench_gfmul(c: &mut Criterion) {
    let mut group = c.benchmark_group("gfmul");

    // gf256 mul/div
    bench_mul!(group, "gf256_naive_mul",            gf256_naive);
    bench_mul!(group, "gf256_table_mul",            gf256_table);
    bench_mul!(group, "gf256_rem_table_mul",        gf256_rem_table);
    bench_mul!(group, "gf256_small_rem_table_mul",  gf256_small_rem_table);
    bench_mul!(group, "gf256_barret_mul",           gf256_barret);

    bench_div!(group, "gf256_naive_div",            gf256_naive);
    bench_div!(group, "gf256_table_div",            gf256_table);
    bench_div!(group, "gf256_rem_table_div",        gf256_rem_table);
    bench_div!(group, "gf256_small_rem_table_div",  gf256_small_rem_table);
    bench_div!(group, "gf256_barret_div",           gf256_barret);

    // gf16 mul/div
    bench_mul!(group, "gf16_naive_mul",             |x: u8| gf16_naive::try_from(x&0xf).unwrap());
    bench_mul!(group, "gf16_table_mul",             |x: u8| gf16_table::try_from(x&0xf).unwrap());
    bench_mul!(group, "gf16_rem_table_mul",         |x: u8| gf16_rem_table::try_from(x&0xf).unwrap());
    bench_mul!(group, "gf16_small_rem_table_mul",   |x: u8| gf16_small_rem_table::try_from(x&0xf).unwrap());
    bench_mul!(group, "gf16_barret_mul",            |x: u8| gf16_barret::try_from(x&0xf).unwrap());

    bench_div!(group, "gf16_naive_div",             |x: u8| gf16_naive::try_from(x&0xf).unwrap());
    bench_div!(group, "gf16_table_div",             |x: u8| gf16_table::try_from(x&0xf).unwrap());
    bench_div!(group, "gf16_rem_table_div",         |x: u8| gf16_rem_table::try_from(x&0xf).unwrap());
    bench_div!(group, "gf16_small_rem_table_div",   |x: u8| gf16_small_rem_table::try_from(x&0xf).unwrap());
    bench_div!(group, "gf16_barret_div",            |x: u8| gf16_barret::try_from(x&0xf).unwrap());

    // gf2p16 mul/div
    bench_mul!(group, "gf2p16_naive_mul",           gf2p16_naive);
    bench_mul!(group, "gf2p16_rem_table_mul",       gf2p16_rem_table);
    bench_mul!(group, "gf2p16_small_rem_table_mul", gf2p16_small_rem_table);
    bench_mul!(group, "gf2p16_barret_mul",          gf2p16_barret);

    bench_div!(group, "gf2p16_naive_div",           gf2p16_naive);
    bench_div!(group, "gf2p16_rem_table_div",       gf2p16_rem_table);
    bench_div!(group, "gf2p16_small_rem_table_div", gf2p16_small_rem_table);
    bench_div!(group, "gf2p16_barret_div",          gf2p16_barret);

    // gf2p32 mul/div
    bench_mul!(group, "gf2p32_naive_mul",           gf2p32_naive);
    bench_mul!(group, "gf2p32_rem_table_mul",       gf2p32_rem_table);
    bench_mul!(group, "gf2p32_small_rem_table_mul", gf2p32_small_rem_table);
    bench_mul!(group, "gf2p32_barret_mul",          gf2p32_barret);

    bench_div!(group, "gf2p32_naive_div",           gf2p32_naive);
    bench_div!(group, "gf2p32_rem_table_div",       gf2p32_rem_table);
    bench_div!(group, "gf2p32_small_rem_table_div", gf2p32_small_rem_table);
    bench_div!(group, "gf2p32_barret_div",          gf2p32_barret);

    // gf2p64 mul/div
    bench_mul!(group, "gf2p64_naive_mul",           gf2p64_naive);
    bench_mul!(group, "gf2p64_rem_table_mul",       gf2p64_rem_table);
    bench_mul!(group, "gf2p64_small_rem_table_mul", gf2p64_small_rem_table);
    bench_mul!(group, "gf2p64_barret_mul",          gf2p64_barret);

    bench_div!(group, "gf2p64_naive_div",           gf2p64_naive);
    bench_div!(group, "gf2p64_rem_table_div",       gf2p64_rem_table);
    bench_div!(group, "gf2p64_small_rem_table_div", gf2p64_small_rem_table);
    bench_div!(group, "gf2p64_barret_div",          gf2p64_barret);
}

criterion_group!(benches, bench_gfmul);
criterion_main!(benches);
