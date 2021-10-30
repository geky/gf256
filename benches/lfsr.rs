//! Lets compare various CRC implementations

use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use criterion::Throughput;
use criterion::measurement::Measurement;
use criterion::measurement::ValueFormatter;
use rand::SeedableRng;
use rand::RngCore;
use flate2::Compression;
use flate2::write::DeflateEncoder;
use std::io::Write;


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
struct Xorshift64Rng(u64);

impl SeedableRng for Xorshift64Rng {
    type Seed = [u8; 8];

    fn from_seed(mut seed: Self::Seed) -> Self {
        // make sure seed does not equal zero! otherwise our rng would only
        // ever output zero!
        if seed.iter().all(|&x| x == 0) {
            seed = [1,2,3,4,5,6,7,8];
        }

        Xorshift64Rng(u64::from_le_bytes(seed))
    }

    fn from_rng<R: RngCore>(mut rng: R) -> Result<Self, rand::Error> {
        let mut seed = [0; 8];
        while seed.iter().all(|&x| x == 0) {
            rng.try_fill_bytes(&mut seed)?;
        }

        Ok(Xorshift64Rng::from_seed(seed))
    }
}

impl RngCore for Xorshift64Rng {
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        rand_core::impls::fill_bytes_via_next(self, dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        Ok(self.fill_bytes(dest))
    }

    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn next_u64(&mut self) -> u64 {
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
    let mut buffer = vec![0; SIZE];

    // xorshift timing
    let mut xorshift64 = Xorshift64Rng::from_seed([1,2,3,4,5,6,7,8]);
    group.bench_function("xorshift64", |b| b.iter(
        || xorshift64.fill_bytes(&mut buffer)
    ));

    // gf2p64 timing
    let mut gf2p64 = lfsr::Gf2p64Rng::from_seed([1,2,3,4,5,6,7,8]);
    group.bench_function("gf2p64", |b| b.iter(
        || gf2p64.fill_bytes(&mut buffer)
    ));
}

fn bench_lfsr_compressability(c: &mut Criterion<Compressability>) {
    let mut group = c.benchmark_group("lfsr");

    // size to bench
    const SIZE: usize = 1024*1024;
    let mut buffer = vec![0; SIZE];

    // xorshift compressability
    let mut xorshift64 = Xorshift64Rng::from_seed([1,2,3,4,5,6,7,8]);
    group.bench_function("xorshift64_compressability", |b| b.iter_custom(
        |iters| {
            let mut sum = 0.0;
            for _ in 0..iters { 
                xorshift64.fill_bytes(&mut buffer);
                let mut comp = DeflateEncoder::new(Vec::new(), Compression::best());
                comp.write_all(&buffer).unwrap();
                let comp = comp.finish().unwrap();
                sum += ((SIZE as f64) - (comp.len() as f64)) / (SIZE as f64);
            }
            sum
        }
    ));

    // gf2p64 compressability
    let mut gf2p64 = lfsr::Gf2p64Rng::from_seed([1,2,3,4,5,6,7,8]);
    group.bench_function("gf2p64_compressability", |b| b.iter_custom(
        |iters| {
            let mut sum = 0.0;
            for _ in 0..iters { 
                gf2p64.fill_bytes(&mut buffer);
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
