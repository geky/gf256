//! Lets compare various CRC implementations

use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use criterion::BatchSize;
use criterion::Throughput;
use std::iter;
use std::collections::HashSet;

#[allow(dead_code)]
#[allow(unused_attributes)]
#[path = "../examples/rs.rs"]
mod rs;

fn bench_rs(c: &mut Criterion) {
    let mut group = c.benchmark_group("rs");

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

    // note we are using Reed-Solomon (20, 12) only because it's what is in our
    // example, this isn't necessarily the most efficient geometry, but we
    // really only care about relative benchmarks

    // encode
    let mut xs = xorshift64(42);
    group.bench_function("rs_encode", |b| b.iter_batched_ref(
        || (&mut xs).take(SIZE).map(|x| x as u8).collect::<Vec<u8>>(),
        |data| {
            data.chunks(rs::DATA_SIZE)
                .map(|chunk| {
                    let mut chunk = Vec::from(chunk);
                    chunk.resize(chunk.len() + rs::ECC_SIZE, 0);
                    rs::rs_encode(&mut chunk);
                    chunk
                })
                .collect::<Vec<_>>()
        },
        BatchSize::SmallInput
    ));

    // correct w/ no errors
    let mut xs = xorshift64(42);
    group.bench_function("rs_correct_none", |b| b.iter_batched_ref(
        || {
            let data = (&mut xs).take(SIZE).map(|x| x as u8).collect::<Vec<u8>>();
            data.chunks(rs::DATA_SIZE)
                .map(|chunk| {
                    let mut chunk = Vec::from(chunk);
                    chunk.resize(chunk.len() + rs::ECC_SIZE, 0);
                    rs::rs_encode(&mut chunk);
                    chunk
                })
                .collect::<Vec<_>>()
        },
        |data| {
            for chunk in data.iter_mut() {
                assert!(rs::rs_is_correct(chunk));
            }
        },
        BatchSize::SmallInput
    ));

    // correct w/ <=ECC_SIZE erasures
    let mut xs = xorshift64(42);
    group.bench_function("rs_correct_erasures", |b| b.iter_batched_ref(
        || {
            let data = (&mut xs).take(SIZE).map(|x| x as u8).collect::<Vec<u8>>();
            data.chunks(rs::DATA_SIZE)
                .map(|chunk| {
                    let mut chunk = Vec::from(chunk);
                    chunk.resize(chunk.len() + rs::ECC_SIZE, 0);
                    rs::rs_encode(&mut chunk);
                    let chunk_len = chunk.len();

                    let mut erasures = HashSet::new();
                    for erasure in
                        (&mut xs)
                            .take(rs::ECC_SIZE)
                            .map(|e| (e as usize) % chunk_len)
                    {
                        erasures.insert(erasure);
                        chunk[erasure] = b'x';
                    }

                    (chunk, erasures.into_iter().collect::<Vec<_>>())
                })
                .collect::<Vec<_>>()
        },
        |data| {
            for (chunk, erasures) in data.iter_mut() {
                rs::rs_correct_erasures(chunk, &erasures).unwrap();
            }
        },
        BatchSize::SmallInput
    ));

    // correct w/ <=ECC_SIZE/2 errors
    let mut xs = xorshift64(42);
    group.bench_function("rs_correct_errors", |b| b.iter_batched_ref(
        || {
            let data = (&mut xs).take(SIZE).map(|x| x as u8).collect::<Vec<u8>>();
            data.chunks(rs::DATA_SIZE)
                .map(|chunk| {
                    let mut chunk = Vec::from(chunk);
                    chunk.resize(chunk.len() + rs::ECC_SIZE, 0);
                    rs::rs_encode(&mut chunk);
                    let chunk_len = chunk.len();

                    for error in
                        (&mut xs)
                            .take(rs::ECC_SIZE / 2)
                            .map(|e| (e as usize) % chunk_len)
                    {
                        chunk[error] = b'x';
                    }

                    chunk
                })
                .collect::<Vec<_>>()
        },
        |data| {
            for chunk in data.iter_mut() {
                rs::rs_correct_errors(chunk).unwrap();
            }
        },
        BatchSize::SmallInput
    ));

    // correct w/ 2*errors+erasures <= ECC_SIZE
    let mut xs = xorshift64(42);
    group.bench_function("rs_correct", |b| b.iter_batched_ref(
        || {
            let data = (&mut xs).take(SIZE).map(|x| x as u8).collect::<Vec<u8>>();
            data.chunks(rs::DATA_SIZE)
                .map(|chunk| {
                    let mut chunk = Vec::from(chunk);
                    chunk.resize(chunk.len() + rs::ECC_SIZE, 0);
                    rs::rs_encode(&mut chunk);
                    let chunk_len = chunk.len();

                    let erasure_count = ((&mut xs).next().unwrap() as usize)
                        % rs::ECC_SIZE;

                    let mut erasures = HashSet::new();
                    for erasure in
                        (&mut xs)
                            .take(erasure_count)
                            .map(|e| (e as usize) % chunk_len)
                    {
                        erasures.insert(erasure);
                        chunk[erasure] = b'x';
                    }

                    for error in
                        (&mut xs)
                            .take((rs::ECC_SIZE-erasure_count) / 2)
                            .map(|e| (e as usize) % chunk_len)
                    {
                        chunk[error] = b'x';
                    }

                    (chunk, erasures.into_iter().collect::<Vec<_>>())
                })
                .collect::<Vec<_>>()
        },
        |data| {
            for (chunk, erasures) in data.iter_mut() {
                rs::rs_correct(chunk, &erasures).unwrap();
            }
        },
        BatchSize::SmallInput
    ));
}

criterion_group!(benches, bench_rs);
criterion_main!(benches);
