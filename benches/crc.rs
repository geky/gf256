//! Lets compare various CRC implementations

use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use criterion::BatchSize;
use criterion::Throughput;
use std::iter;

#[allow(dead_code)]
#[path = "../examples/crc.rs"]
mod crc;

fn bench_crc(c: &mut Criterion) {
    let mut group = c.benchmark_group("crc");

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
    const SIZE: usize = 1024*1024;
    group.throughput(Throughput::Bytes(SIZE as u64));

    // naive crc
    let mut xs = xorshift64(42).map(|x| x as u8);
    group.bench_function("naive_crc", |b| b.iter_batched_ref(
        || (&mut xs).take(SIZE).collect::<Vec<u8>>(),
        |data| crc::naive_crc(data),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| x as u8);
    group.bench_function("less_naive_crc", |b| b.iter_batched_ref(
        || (&mut xs).take(SIZE).collect::<Vec<u8>>(),
        |data| crc::less_naive_crc(data),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| x as u8);
    group.bench_function("word_less_naive_crc", |b| b.iter_batched_ref(
        || (&mut xs).take(SIZE).collect::<Vec<u8>>(),
        |data| crc::word_less_naive_crc(data),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| x as u8);
    group.bench_function("table_crc", |b| b.iter_batched_ref(
        || (&mut xs).take(SIZE).collect::<Vec<u8>>(),
        |data| crc::table_crc(data),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| x as u8);
    group.bench_function("small_table_crc", |b| b.iter_batched_ref(
        || (&mut xs).take(SIZE).collect::<Vec<u8>>(),
        |data| crc::small_table_crc(data),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| x as u8);
    group.bench_function("barret_crc", |b| b.iter_batched_ref(
        || (&mut xs).take(SIZE).collect::<Vec<u8>>(),
        |data| crc::barret_crc(data),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| x as u8);
    group.bench_function("word_barret_crc", |b| b.iter_batched_ref(
        || (&mut xs).take(SIZE).collect::<Vec<u8>>(),
        |data| crc::word_barret_crc(data),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| x as u8);
    group.bench_function("reversed_barret_crc", |b| b.iter_batched_ref(
        || (&mut xs).take(SIZE).collect::<Vec<u8>>(),
        |data| crc::reversed_barret_crc(data),
        BatchSize::SmallInput
    ));

    let mut xs = xorshift64(42).map(|x| x as u8);
    group.bench_function("word_reversed_barret_crc", |b| b.iter_batched_ref(
        || (&mut xs).take(SIZE).collect::<Vec<u8>>(),
        |data| crc::word_reversed_barret_crc(data),
        BatchSize::SmallInput
    ));
}

criterion_group!(benches, bench_crc);
criterion_main!(benches);
