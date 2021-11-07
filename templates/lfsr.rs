//! Template for LFSR structs
//!
//! See examples/lfsr.rs for a more detailed explanation of
//! where these implementations come from

use __crate::internal::rand::RngCore;
use __crate::internal::rand::SeedableRng;
use __crate::traits::FromLossy;
use core::iter::FusedIterator;
use core::mem::size_of;


/// TODO doc
#[derive(Debug, Clone)]
pub struct __lfsr(__gf);

impl __lfsr {
    #[inline]
    pub const fn new(mut seed: __u) -> Self {
        // make sure seed does not equal zero! otherwise our rng would only
        // ever output zero!
        if seed == 0 {
            seed = 1;
        }

        Self(unsafe { __gf::new_unchecked(seed & __gf::NONZEROS) })
    }

    #[inline]
    pub fn next(&mut self) -> __u {
        // Stepping through the LFSR is equivalent to multiplying be our
        // primitive element
        self.0 *= __gf::GENERATOR;
        __u::from_lossy(self.0)
    }

    #[inline]
    pub fn prev(&mut self) -> __u {
        // Since division is the perfect inverse of multiplication in our
        // field, we can even run our LFSR backwards
        let x = self.0;
        self.0 /= __gf::GENERATOR;
        __u::from_lossy(x)
    }
}


// Iterator implementation

impl Iterator for __lfsr {
    type Item = __u;

    #[inline]
    fn next(&mut self) -> Option<__u> {
        Some(self.next())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // this is an infinite iterator
        (usize::MAX, None)
    }
}

impl FusedIterator for __lfsr {}


// Rng implementation

impl SeedableRng for __lfsr {
    type Seed = [u8; size_of::<__u>()];

    #[inline]
    fn from_seed(seed: Self::Seed) -> Self {
        Self::new(__u::from_le_bytes(seed))
    }

    #[inline]
    fn from_rng<R: RngCore>(mut rng: R) -> Result<Self, rand::Error> {
        let mut seed = [0; size_of::<__u>()];
        while seed.iter().all(|&x| x == 0) {
            rng.try_fill_bytes(&mut seed)?;
        }

        Ok(Self::from_seed(seed))
    }
}

impl RngCore for __lfsr {
    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        // fill words at a time
        let mut chunks = dest.chunks_exact_mut(size_of::<__u>());
        for chunk in &mut chunks {
            chunk.copy_from_slice(&self.next().to_le_bytes());
        }

        let remainder = chunks.into_remainder();
        if remainder.len() > 0 {
            remainder.copy_from_slice(&self.next().to_le_bytes()[..remainder.len()]);
        }
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        Ok(self.fill_bytes(dest))
    }

    #[inline]
    fn next_u32(&mut self) -> u32 {
        // this should get optimized out, it's a bit tricky to make this
        // a compile time check
        if size_of::<__u>() >= size_of::<u32>() {
            self.next() as u32
        } else {
            let mut buf = [0; 4];
            self.fill_bytes(&mut buf);
            u32::from_le_bytes(buf)
        }
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        // this should get optimized out, it's a bit tricky to make this
        // a compile time check
        if size_of::<__u>() >= size_of::<u64>() {
            self.next() as u64
        } else {
            let mut buf = [0; 8];
            self.fill_bytes(&mut buf);
            u64::from_le_bytes(buf)
        }
    }
}

