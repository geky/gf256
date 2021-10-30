//! Lets compare various CRC implementations

use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use criterion::Throughput;
use rand::SeedableRng;
use rand::RngCore;


#[allow(dead_code)]
#[path = "../examples/lfsr.rs"]
mod lfsr;

/// An xorshift64 implementation to compare against
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


fn bench_lfsr(c: &mut Criterion) {
    let mut group = c.benchmark_group("lfsr");

    // size to bench
    const SIZE: usize = 1024*1024;
    group.throughput(Throughput::Bytes(SIZE as u64));
    let mut buffer = vec![0; SIZE];

    // xorshift prng
    let mut xorshift64 = Xorshift64Rng::from_seed([1,2,3,4,5,6,7,8]);
    group.bench_function("xorshift64", |b| b.iter(
        || xorshift64.fill_bytes(&mut buffer)
    ));

    // gf2p64 prng
    let mut gf2p64 = lfsr::Gf2p64Rng::from_seed([1,2,3,4,5,6,7,8]);
    group.bench_function("gf2p64", |b| b.iter(
        || gf2p64.fill_bytes(&mut buffer)
    ));
}

criterion_group!(benches, bench_lfsr);
criterion_main!(benches);
