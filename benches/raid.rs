//! Lets compare various CRC implementations

use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use criterion::BatchSize;
use criterion::Throughput;
use std::iter;
use std::convert::TryFrom;

#[allow(dead_code)]
#[path = "../examples/raid.rs"]
mod raid;

fn bench_raid(c: &mut Criterion) {
    let mut group = c.benchmark_group("raid");

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
    const COUNT: usize = 5;

    // format
    let mut xs = xorshift64(42).map(|x| x as u8);
    group.throughput(Throughput::Bytes((COUNT*SIZE) as u64));
    group.bench_function("raid5_format", |b| b.iter_batched_ref(
        || {(
            iter::repeat_with(|| {
                (&mut xs).take(SIZE).collect::<Vec<u8>>()
            })
                .take(COUNT)
                .collect::<Vec<_>>(),
            (&mut xs).take(SIZE).collect::<Vec<u8>>()
        )},
        |(disks, p)| raid::raid5_format(disks, p),
        BatchSize::SmallInput
    ));

    // update
    let mut xs = xorshift64(42).map(|x| x as u8);
    let mut disks = iter::repeat_with(|| {
            (&mut xs).take(SIZE).collect::<Vec<u8>>()
        })
        .take(COUNT)
        .collect::<Vec<_>>();
    let mut p = (&mut xs).take(SIZE).collect::<Vec<u8>>();
    raid::raid5_format(&disks, &mut p);
    group.throughput(Throughput::Bytes(SIZE as u64));
    group.bench_function("raid5_update", |b| b.iter_batched_ref(
        || {(
            usize::from((&mut xs).next().unwrap() % u8::try_from(COUNT).unwrap()),
            (&mut xs).take(SIZE).collect::<Vec<u8>>(),
        )},
        |(i, data)| {
            raid::raid5_update(*i, &disks[*i], data, &mut p);
            disks[*i].copy_from_slice(data);
        },
        BatchSize::SmallInput
    ));

    // repair
    let mut xs = xorshift64(42).map(|x| x as u8);
    group.throughput(Throughput::Bytes((1*SIZE) as u64));
    group.bench_function("raid5_repair", |b| b.iter_batched_ref(
        || {(
            usize::from((&mut xs).next().unwrap() % u8::try_from(COUNT+1).unwrap()),
            iter::repeat_with(|| {
                (&mut xs).take(SIZE).collect::<Vec<u8>>()
            })
                .take(COUNT)
                .collect::<Vec<_>>(),
            (&mut xs).take(SIZE).collect::<Vec<u8>>()
        )},
        |(i, disks, p)| raid::raid5_repair(disks, p, &[*i]),
        BatchSize::SmallInput
    ));

    // format
    let mut xs = xorshift64(42).map(|x| x as u8);
    group.throughput(Throughput::Bytes((COUNT*SIZE) as u64));
    group.bench_function("raid6_format", |b| b.iter_batched_ref(
        || {(
            iter::repeat_with(|| {
                    (&mut xs).take(SIZE).collect::<Vec<u8>>()
                })
                .take(COUNT)
                .collect::<Vec<_>>(),
            (&mut xs).take(SIZE).collect::<Vec<u8>>(),
            (&mut xs).take(SIZE).collect::<Vec<u8>>()
        )},
        |(disks, p, q)| raid::raid6_format(disks, p, q),
        BatchSize::SmallInput
    ));

    // update
    let mut xs = xorshift64(42).map(|x| x as u8);
    let mut disks = iter::repeat_with(|| {
            (&mut xs).take(SIZE).collect::<Vec<u8>>()
        })
        .take(COUNT+2)
        .collect::<Vec<_>>();
    let mut p = (&mut xs).take(SIZE).collect::<Vec<u8>>();
    let mut q = (&mut xs).take(SIZE).collect::<Vec<u8>>();
    raid::raid6_format(&mut disks, &mut p, &mut q);
    group.throughput(Throughput::Bytes(SIZE as u64));
    group.bench_function("raid6_update", |b| b.iter_batched_ref(
        || {(
            usize::from((&mut xs).next().unwrap() % u8::try_from(COUNT).unwrap()),
            (&mut xs).take(SIZE).collect::<Vec<u8>>(),
        )},
        |(i, data)| {
            raid::raid6_update(*i, &disks[*i], data, &mut p, &mut q);
            disks[*i].copy_from_slice(data);
        },
        BatchSize::SmallInput
    ));

    // repair 1
    let mut xs = xorshift64(42).map(|x| x as u8);
    group.throughput(Throughput::Bytes((1*SIZE) as u64));
    group.bench_function("raid6_repair_1", |b| b.iter_batched_ref(
        || {(
            usize::from((&mut xs).next().unwrap() % u8::try_from(COUNT+2).unwrap()),
            iter::repeat_with(|| {
                    (&mut xs).take(SIZE).collect::<Vec<u8>>()
                })
                .take(COUNT+2)
                .collect::<Vec<_>>(),
            (&mut xs).take(SIZE).collect::<Vec<u8>>(),
            (&mut xs).take(SIZE).collect::<Vec<u8>>()
        )},
        |(i, disks, p, q)| raid::raid6_repair(disks, p, q, &[*i]),
        BatchSize::SmallInput
    ));

    // repair 2
    let mut xs = xorshift64(42).map(|x| x as u8);
    group.throughput(Throughput::Bytes((2*SIZE) as u64));
    group.bench_function("raid6_repair_2", |b| b.iter_batched_ref(
        || {
            let i = usize::from((&mut xs).next().unwrap() % u8::try_from(COUNT+2).unwrap());
            (
                i,
                (i+1) % (COUNT+2),
                iter::repeat_with(|| {
                        (&mut xs).take(SIZE).collect::<Vec<u8>>()
                    })
                    .take(COUNT+2)
                    .collect::<Vec<_>>(),
                (&mut xs).take(SIZE).collect::<Vec<u8>>(),
                (&mut xs).take(SIZE).collect::<Vec<u8>>()
            )
        },
        |(i, j, disks, p, q)| raid::raid6_repair(disks, p, q, &[*i, *j]),
        BatchSize::SmallInput
    ));

    // format
    let mut xs = xorshift64(42).map(|x| x as u8);
    group.throughput(Throughput::Bytes((COUNT*SIZE) as u64));
    group.bench_function("raid7_format", |b| b.iter_batched_ref(
        || {(
            iter::repeat_with(|| {
                    (&mut xs).take(SIZE).collect::<Vec<u8>>()
                })
                .take(COUNT)
                .collect::<Vec<_>>(),
            (&mut xs).take(SIZE).collect::<Vec<u8>>(),
            (&mut xs).take(SIZE).collect::<Vec<u8>>(),
            (&mut xs).take(SIZE).collect::<Vec<u8>>()
        )},
        |(disks, p, q, r)| raid::raid7_format(disks, p, q, r),
        BatchSize::SmallInput
    ));

    // update
    let mut xs = xorshift64(42).map(|x| x as u8);
    let mut disks = iter::repeat_with(|| {
            (&mut xs).take(SIZE).collect::<Vec<u8>>()
        })
        .take(COUNT+2)
        .collect::<Vec<_>>();
    let mut p = (&mut xs).take(SIZE).collect::<Vec<u8>>();
    let mut q = (&mut xs).take(SIZE).collect::<Vec<u8>>();
    let mut r = (&mut xs).take(SIZE).collect::<Vec<u8>>();
    raid::raid6_format(&mut disks, &mut p, &mut q);
    group.throughput(Throughput::Bytes(SIZE as u64));
    group.bench_function("raid7_update", |b| b.iter_batched_ref(
        || {(
            usize::from((&mut xs).next().unwrap() % u8::try_from(COUNT).unwrap()),
            (&mut xs).take(SIZE).collect::<Vec<u8>>(),
        )},
        |(i, data)| {
            raid::raid7_update(*i, &disks[*i], data, &mut p, &mut q, &mut r);
            disks[*i].copy_from_slice(data);
        },
        BatchSize::SmallInput
    ));

    // repair 1
    let mut xs = xorshift64(42).map(|x| x as u8);
    group.throughput(Throughput::Bytes((1*SIZE) as u64));
    group.bench_function("raid7_repair_1", |b| b.iter_batched_ref(
        || {(
            usize::from((&mut xs).next().unwrap() % u8::try_from(COUNT+3).unwrap()),
            iter::repeat_with(|| {
                    (&mut xs).take(SIZE).collect::<Vec<u8>>()
                })
                .take(COUNT+2)
                .collect::<Vec<_>>(),
            (&mut xs).take(SIZE).collect::<Vec<u8>>(),
            (&mut xs).take(SIZE).collect::<Vec<u8>>(),
            (&mut xs).take(SIZE).collect::<Vec<u8>>()
        )},
        |(i, disks, p, q, r)| raid::raid7_repair(disks, p, q, r, &[*i]),
        BatchSize::SmallInput
    ));

    // repair 2
    let mut xs = xorshift64(42).map(|x| x as u8);
    group.throughput(Throughput::Bytes((2*SIZE) as u64));
    group.bench_function("raid7_repair_2", |b| b.iter_batched_ref(
        || {
            let i = usize::from((&mut xs).next().unwrap() % u8::try_from(COUNT+3).unwrap());
            (
                i,
                (i+1) % (COUNT+2),
                iter::repeat_with(|| {
                        (&mut xs).take(SIZE).collect::<Vec<u8>>()
                    })
                    .take(COUNT+2)
                    .collect::<Vec<_>>(),
                (&mut xs).take(SIZE).collect::<Vec<u8>>(),
                (&mut xs).take(SIZE).collect::<Vec<u8>>(),
                (&mut xs).take(SIZE).collect::<Vec<u8>>()
            )
        },
        |(i, j, disks, p, q, r)| raid::raid7_repair(disks, p, q, r, &[*i, *j]),
        BatchSize::SmallInput
    ));

    // repair 3
    let mut xs = xorshift64(42).map(|x| x as u8);
    group.throughput(Throughput::Bytes((2*SIZE) as u64));
    group.bench_function("raid7_repair_3", |b| b.iter_batched_ref(
        || {
            let i = usize::from((&mut xs).next().unwrap() % u8::try_from(COUNT+3).unwrap());
            (
                i,
                (i+1) % (COUNT+3),
                (i+2) % (COUNT+3),
                iter::repeat_with(|| {
                        (&mut xs).take(SIZE).collect::<Vec<u8>>()
                    })
                    .take(COUNT+2)
                    .collect::<Vec<_>>(),
                (&mut xs).take(SIZE).collect::<Vec<u8>>(),
                (&mut xs).take(SIZE).collect::<Vec<u8>>(),
                (&mut xs).take(SIZE).collect::<Vec<u8>>()
            )
        },
        |(i, j, k, disks, p, q, r)| raid::raid7_repair(disks, p, q, r, &[*i, *j, *k]),
        BatchSize::SmallInput
    ));
}

criterion_group!(benches, bench_raid);
criterion_main!(benches);
