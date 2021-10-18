//! Lets compare various CRC implementations

use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use criterion::BatchSize;
use criterion::Throughput;
use std::iter;
use std::io;
use std::convert::TryFrom;
use std::cell::RefCell;
use std::io::Write;

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
    group.bench_function("raid4_format", |b| b.iter_batched_ref(
        || {
            iter::repeat_with(|| {
                io::Cursor::new((&mut xs).take(SIZE).collect::<Vec<u8>>())
            })
            .take(COUNT+1)
            .collect::<Vec<_>>()
        },
        |disks| raid::Raid4::format(disks),
        BatchSize::SmallInput
    ));

    // update
    let mut xs = xorshift64(42).map(|x| x as u8);
    let mut disks = iter::repeat_with(|| {
            io::Cursor::new((&mut xs).take(SIZE).collect::<Vec<u8>>())
        })
        .take(COUNT+1)
        .collect::<Vec<_>>();
    raid::Raid4::format(&mut disks).unwrap();
    let disks = RefCell::new(disks);
    let mut raid_ = raid::Raid4::mount(&disks).unwrap();
    group.throughput(Throughput::Bytes(SIZE as u64));
    group.bench_function("raid4_update", |b| b.iter_batched_ref(
        || (
            usize::from((&mut xs).next().unwrap() % u8::try_from(COUNT).unwrap()),
            (&mut xs).take(SIZE).collect::<Vec<u8>>(),
        ),
        |(i, data)| {
            raid_[*i].write_all(data).unwrap();
            raid_[*i].flush().unwrap();
        },
        BatchSize::SmallInput
    ));

    // repair
    let mut xs = xorshift64(42).map(|x| x as u8);
    group.throughput(Throughput::Bytes((COUNT*SIZE) as u64));
    group.bench_function("raid4_repair", |b| b.iter_batched_ref(
        || {
            (
                usize::from((&mut xs).next().unwrap() % u8::try_from(COUNT+1).unwrap()),
                iter::repeat_with(|| {
                    io::Cursor::new((&mut xs).take(SIZE).collect::<Vec<u8>>())
                })
                .take(COUNT+1)
                .collect::<Vec<_>>()
            )
        },
        |(i, disks)| raid::Raid4::repair(disks, &[*i]),
        BatchSize::SmallInput
    ));

    // format
    let mut xs = xorshift64(42).map(|x| x as u8);
    group.throughput(Throughput::Bytes((COUNT*SIZE) as u64));
    group.bench_function("raid6_format", |b| b.iter_batched_ref(
        || {
            iter::repeat_with(|| {
                io::Cursor::new((&mut xs).take(SIZE).collect::<Vec<u8>>())
            })
            .take(COUNT+2)
            .collect::<Vec<_>>()
        },
        |disks| raid::Raid6::format(disks),
        BatchSize::SmallInput
    ));

    // update
    let mut xs = xorshift64(42).map(|x| x as u8);
    let mut disks = iter::repeat_with(|| {
            io::Cursor::new((&mut xs).take(SIZE).collect::<Vec<u8>>())
        })
        .take(COUNT+2)
        .collect::<Vec<_>>();
    raid::Raid6::format(&mut disks).unwrap();
    let disks = RefCell::new(disks);
    let mut raid_ = raid::Raid6::mount(&disks).unwrap();
    group.throughput(Throughput::Bytes(SIZE as u64));
    group.bench_function("raid6_update", |b| b.iter_batched_ref(
        || (
            usize::from((&mut xs).next().unwrap() % u8::try_from(COUNT).unwrap()),
            (&mut xs).take(SIZE).collect::<Vec<u8>>(),
        ),
        |(i, data)| {
            raid_[*i].write_all(data).unwrap();
            raid_[*i].flush().unwrap();
        },
        BatchSize::SmallInput
    ));

    // repair 1
    let mut xs = xorshift64(42).map(|x| x as u8);
    group.throughput(Throughput::Bytes((COUNT*SIZE) as u64));
    group.bench_function("raid6_repair_1", |b| b.iter_batched_ref(
        || {
            (
                usize::from((&mut xs).next().unwrap() % u8::try_from(COUNT+2).unwrap()),
                iter::repeat_with(|| {
                    io::Cursor::new((&mut xs).take(SIZE).collect::<Vec<u8>>())
                })
                .take(COUNT+2)
                .collect::<Vec<_>>()
            )
        },
        |(i, disks)| raid::Raid6::repair(disks, &[*i]),
        BatchSize::SmallInput
    ));

    // repair 2
    let mut xs = xorshift64(42).map(|x| x as u8);
    group.throughput(Throughput::Bytes((COUNT*SIZE) as u64));
    group.bench_function("raid6_repair_2", |b| b.iter_batched_ref(
        || {
            let i = usize::from((&mut xs).next().unwrap() % u8::try_from(COUNT+2).unwrap());
            (
                i,
                (i+1) % (COUNT+2),
                iter::repeat_with(|| {
                    io::Cursor::new((&mut xs).take(SIZE).collect::<Vec<u8>>())
                })
                .take(COUNT+2)
                .collect::<Vec<_>>()
            )
        },
        |(i, j, disks)| raid::Raid6::repair(disks, &[*i, *j]),
        BatchSize::SmallInput
    ));
}

criterion_group!(benches, bench_raid);
criterion_main!(benches);
