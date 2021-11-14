//! Lets compare various CRC implementations

use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use criterion::Throughput;
use criterion::measurement::Measurement;
use criterion::measurement::ValueFormatter;
use flate2::Compression;
use flate2::write::DeflateEncoder;
use std::io::Write;
use std::mem::size_of;


#[allow(dead_code)]
#[path = "../examples/lfsr.rs"]
mod lfsr;


/// An xorshift64 implementation to compare against
///
/// Note this prng prioritizes efficiency over randomness quality, which
/// is perfect since the same can be said for the gf256 LFSRs
///
/// https://en.wikipedia.org/wiki/Xorshift
///
#[derive(Debug, Clone)]
struct Xorshift64(u64);

impl Xorshift64 {
    fn new(mut seed: u64) -> Self {
        if seed == 0 {
            seed = 1;
        }

        Self(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

/// Custom measurement trait for Criterion
struct Compressability;
impl Measurement for Compressability {
    type Intermediate = f64;
    type Value = f64;

    fn start(&self) -> Self::Intermediate {
        unimplemented!()
    }

    fn end(&self, _: Self::Intermediate) -> Self::Value {
        unimplemented!()
    }

    fn add(&self, v1: &Self::Value, v2: &Self::Value) -> Self::Value {
        v1 + v2
    }

    fn zero(&self) -> Self::Value {
        0.0
    }

    fn to_f64(&self, v: &Self::Value) -> f64 {
        *v
    }

    fn formatter(&self) -> &dyn ValueFormatter {
        &CompressabilityFormatter
    }
}

struct CompressabilityFormatter;
impl ValueFormatter for CompressabilityFormatter {
    fn scale_values(
        &self,
        _typical_value: f64,
        _values: &mut [f64]
    ) -> &'static str {
        "%"
    }

    fn scale_throughputs(
        &self,
        _typical_value: f64,
        _throughput: &Throughput,
        _values: &mut [f64]
    ) -> &'static str {
        "%"
    }

    fn scale_for_machines(
        &self,
        _values: &mut [f64]
    ) -> &'static str {
        "%"
    }
}


fn bench_lfsr(c: &mut Criterion) {
    let mut group = c.benchmark_group("lfsr");

    // size to bench
    const SIZE: usize = 1024*1024;
    group.throughput(Throughput::Bytes(SIZE as u64));
    let mut buffer = vec![0u64; SIZE/size_of::<u64>()];

    // xorshift timing
    let mut xorshift64 = Xorshift64::new(0x123456789abcdef0);
    group.bench_function("xorshift64", |b| b.iter(
        || buffer.fill_with(|| xorshift64.next())
    ));

    // lfsr64 timings
    let mut lfs64_naive = lfsr::Lfsr64Naive::new(0x123456789abcdef0);
    group.bench_function("lfsr64_naive", |b| b.iter(
        || buffer.fill_with(|| lfs64_naive.next(64))
    ));

    let mut lfs64_divrem = lfsr::Lfsr64DivRem::new(0x123456789abcdef0);
    group.bench_function("lfsr64_divrem", |b| b.iter(
        || buffer.fill_with(|| lfs64_divrem.next(64))
    ));

    let mut lfs64_table = lfsr::Lfsr64Table::new(0x123456789abcdef0);
    group.bench_function("lfsr64_table", |b| b.iter(
        || buffer.fill_with(|| lfs64_table.next(64))
    ));

    let mut lfs64_small_table = lfsr::Lfsr64SmallTable::new(0x123456789abcdef0);
    group.bench_function("lfsr64_small_table", |b| b.iter(
        || buffer.fill_with(|| lfs64_small_table.next(64))
    ));

    let mut lfs64_barret = lfsr::Lfsr64Barret::new(0x123456789abcdef0);
    group.bench_function("lfsr64_barret", |b| b.iter(
        || buffer.fill_with(|| lfs64_barret.next(64))
    ));

    let mut lfs64_table_barret = lfsr::Lfsr64TableBarret::new(0x123456789abcdef0);
    group.bench_function("lfsr64_table_barret", |b| b.iter(
        || buffer.fill_with(|| lfs64_table_barret.next(64))
    ));

    let mut lfs64_small_table_barret = lfsr::Lfsr64SmallTableBarret::new(0x123456789abcdef0);
    group.bench_function("lfsr64_small_table_barret", |b| b.iter(
        || buffer.fill_with(|| lfs64_small_table_barret.next(64))
    ));
}

fn bench_lfsr_compressability(c: &mut Criterion<Compressability>) {
    let mut group = c.benchmark_group("lfsr");

    // size to bench
    const SIZE: usize = 1024*1024;
    let mut buffer = vec![0; SIZE];

    // xorshift compressability
    let mut xorshift64 = Xorshift64::new(0x123456789abcdef0);
    group.bench_function("xorshift64_compressability", |b| b.iter_custom(
        |iters| {
            let mut sum = 0.0;
            for _ in 0..iters { 
                buffer.fill_with(|| xorshift64.next() as u8);
                let mut comp = DeflateEncoder::new(Vec::new(), Compression::best());
                comp.write_all(&buffer).unwrap();
                let comp = comp.finish().unwrap();
                sum += ((SIZE as f64) - (comp.len() as f64)) / (SIZE as f64);
            }
            sum
        }
    ));

    // lfsr64 compressability
    //
    // this should be consistent across all implementations, unless
    // one of them is broken
    let mut lfsr64 = lfsr::Lfsr64Table::new(0x123456789abcdef0);
    group.bench_function("lfsr64_compressability", |b| b.iter_custom(
        |iters| {
            let mut sum = 0.0;
            for _ in 0..iters {
                buffer.fill_with(|| lfsr64.next(8) as u8);
                let mut comp = DeflateEncoder::new(Vec::new(), Compression::best());
                comp.write_all(&buffer).unwrap();
                let comp = comp.finish().unwrap();
                sum += ((SIZE as f64) - (comp.len() as f64)) / (SIZE as f64);
            }
            sum
        }
    ));
}

criterion_group!(benches, bench_lfsr);
criterion_group! {
    name = benches_compressability;
    config = Criterion::default().with_measurement(Compressability);
    targets = bench_lfsr_compressability
}
criterion_main!(benches, benches_compressability);
