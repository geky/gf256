//! Pseudo-random numbers using Galois LFSRs
//!
//! A linear-feedback shift register (LFSR) is a simple method of creating
//! a pseudo-random stream of bits using only a small circuit of shifts and
//! xors.
//!
//! LFSRs can be modelled mathematically as multiplication in a Galois-field,
//! allowing efficient bit generation, both forward and backwards, and efficient
//! seeking to any state of the LFSR.
//!
//! More information on how LFSRs work can be found in [`lfsr`'s module-level
//! documentation][lfsr-mod].
//!
//! [lfsr-mod]: https://docs.rs/gf256/latest/gf256/lfsr

use std::cmp::min;
use std::cmp::max;
use std::convert::TryFrom;
use std::iter;
use std::slice;
use flate2::Compression;
use flate2::write::DeflateEncoder;
use std::io::Write;
use ::gf256::*;
use ::gf256::traits::FromLossy;


const POLYNOMIAL: p128 = p128(0x1000000000000001b);
const NONZEROS: u64 = u64::MAX;


/// A naive LFSR, implemented with bit-shifts and xors
#[derive(Debug, Clone)]
pub struct Lfsr64Naive(u64);

impl Lfsr64Naive {
    pub fn new(mut seed: u64) -> Self {
        // make sure seed does not equal zero! otherwise our rng would only
        // ever output zero!
        if seed == 0 {
            seed = 1;
        }

        Self(seed)
    }

    pub fn next(&mut self, bits: u64) -> u64 {
        debug_assert!(bits <= 64);
        let mut x = 0;
        for _ in 0..bits {
            let msb = self.0 >> 63;
            x = (x << 1) | msb;
            self.0 = (self.0 << 1) ^ if msb != 0 {
                u64::from_lossy(POLYNOMIAL)
            } else {
                0
            };
        }
        x
    }

    pub fn prev(&mut self, bits: u64) -> u64 {
        debug_assert!(bits <= 64);
        let mut x = 0;
        for _ in 0..bits {
            let lsb = self.0 & 1;
            x = (x >> 1) | (lsb << (bits-1));
            self.0 = (self.0 >> 1) ^ if lsb != 0 {
                u64::from_lossy(POLYNOMIAL >> 1)
            } else {
                0
            };
        }
        x
    }

    pub fn skip(&mut self, bits: u64) {
        // just iterate naively
        for _ in 0..bits {
            let msb = self.0 >> 63;
            self.0 = (self.0 << 1) ^ if msb != 0 {
                u64::from_lossy(POLYNOMIAL)
            } else {
                0
            };
        }
    }

    pub fn skip_backwards(&mut self, bits: u64) {
        // just iterate naively
        for _ in 0..bits {
            let lsb = self.0 & 1;
            self.0 = (self.0 >> 1) ^ if lsb != 0 {
                u64::from_lossy(POLYNOMIAL >> 1)
            } else {
                0
            };
        }
    }
}


/// An LFSR implemented with polynomial operations
#[derive(Debug, Clone)]
pub struct Lfsr64DivRem(p64);

impl Lfsr64DivRem {
    pub fn new(mut seed: u64) -> Self {
        // make sure seed does not equal zero! otherwise our rng would only
        // ever output zero!
        if seed == 0 {
            seed = 1;
        }

        Self(p64(seed))
    }

    pub fn next(&mut self, bits: u64) -> u64 {
        debug_assert!(bits <= 64);
        let x = p128::from(self.0) << bits;
        let q = x / POLYNOMIAL;
        let r = x % POLYNOMIAL;
        self.0 = p64::try_from(r.0).unwrap();
        u64::try_from(q.0).unwrap()
    }

    pub fn prev(&mut self, bits: u64) -> u64 {
        debug_assert!(bits <= 64);
        let x = p128::from(self.0.reverse_bits()) << bits;
        let q = x / (POLYNOMIAL.reverse_bits() >> 63u64);
        let r = x % (POLYNOMIAL.reverse_bits() >> 63u64);
        self.0 = p64::try_from(r.0).unwrap().reverse_bits();
        u64::try_from(q.0).unwrap().reverse_bits() >> (64-bits)
    }

    pub fn skip(&mut self, bits: u64) {
        // Each step of the lfsr is equivalent to multiplication in a finite
        // field by a primitive element g=2, which means we can use exponents of
        // g=2 to efficiently jump around states of the lfsr.
        //
        // lfsr' = 2^skip
        // 
        let mul = |a: p64, b: p64| -> p64 {
            let x = p128::from(a) * p128::from(b);
            p64::try_from(x % POLYNOMIAL).unwrap()
        };

        // Binary exponentiation
        let mut a = p64(2);
        let mut bits = bits;
        let mut g = p64(1);
        loop {
            if bits & 1 != 0 {
                g = mul(g, a);
            }

            bits >>= 1;
            if bits == 0 {
                break;
            }
            a = mul(a, a);
        };

        // Final multiplication
        self.0 = mul(self.0, g);
    }

    pub fn skip_backwards(&mut self, bits: u64) {
        // Assuming our lfsr is well constructed, we're in a multiplicative
        // cycle with 2^width-1 elements. Which means backwards skips are the
        // same as skipping 2^width-1-(skip % 2^width-1) elements
        // 
        self.skip(NONZEROS - (bits % NONZEROS))
    }
}


/// An LFSR implemented using precomputed division and remainder table
#[derive(Debug, Clone)]
pub struct Lfsr64Table(u64);

impl Lfsr64Table {
    // div/rem tables
    const DIV_TABLE: [u8; 256] = {
        let mut div_table = [0; 256];
        let mut i = 0;
        while i < div_table.len() {
            div_table[i] = p128((i as u128) << 64)
                .naive_div(POLYNOMIAL)
                .0 as u8;
            i += 1;
        }
        div_table
    };

    const REM_TABLE: [u64; 256] = {
        let mut rem_table = [0; 256];
        let mut i = 0;
        while i < rem_table.len() {
            rem_table[i] = p128((i as u128) << 64)
                .naive_rem(POLYNOMIAL)
                .0 as u64;
            i += 1;
        }
        rem_table
    };

    // inverse div/rem tables for iterating backwards
    const INVERSE_DIV_TABLE: [u8; 256] = {
        let mut div_table = [0; 256];
        let mut i = 0;
        while i < div_table.len() {
            div_table[i] = (p128(((i as u8).reverse_bits() as u128) << 64)
                .naive_div(p128(POLYNOMIAL.0.reverse_bits() >> 63u64))
                .0 as u8)
                .reverse_bits();
            i += 1;
        }
        div_table
    };

    const INVERSE_REM_TABLE: [u64; 256] = {
        let mut rem_table = [0; 256];
        let mut i = 0;
        while i < rem_table.len() {
            rem_table[i] = (p128(((i as u8).reverse_bits() as u128) << 64)
                .naive_rem(p128(POLYNOMIAL.0.reverse_bits() >> 63u64))
                .0 as u64)
                .reverse_bits();
            i += 1;
        }
        rem_table
    };

    // implementation starts here
    pub fn new(mut seed: u64) -> Self {
        // make sure seed does not equal zero! otherwise our rng would only
        // ever output zero!
        if seed == 0 {
            seed = 1;
        }

        Self(seed)
    }

    pub fn next(&mut self, bits: u64) -> u64 {
        debug_assert!(bits <= 64);
        let mut x = self.0;
        let mut q = 0;
        for i in (0..bits/8).rev() {
            let n = min(8, bits-8*i);
            q = (q << n) | u64::from(Self::DIV_TABLE[usize::try_from(x >> (64-n)).unwrap()]);
            x = (x << n) ^ Self::REM_TABLE[usize::try_from(x >> (64-n)).unwrap()];
        }
        self.0 = x;
        q
    }

    pub fn prev(&mut self, bits: u64) -> u64 {
        debug_assert!(bits <= 64);
        let mut x = self.0;
        let mut q = 0;
        for i in 0..bits/8 {
            let n = min(8, bits-8*i);
            let m = (1 << n)-1;
            q |= u64::from(Self::INVERSE_DIV_TABLE[usize::try_from(x & m).unwrap()]) << (8*i);
            x = (x >> n) ^ Self::INVERSE_REM_TABLE[usize::try_from(x & m).unwrap()];
        }
        self.0 = x;
        q
    }

    pub fn skip(&mut self, bits: u64) {
        // Each step of the lfsr is equivalent to multiplication in a finite
        // field by a primitive element g=2, which means we can use exponents of
        // g=2 to efficiently jump around states of the lfsr.
        //
        // lfsr' = 2^skip
        // 
        let mul = |a: p64, b: p64| -> p64 {
            let (lo, hi) = a.widening_mul(b);
            let mut x = 0u64;
            for b in hi.to_be_bytes() {
                x = (x << 8) ^ Self::REM_TABLE[usize::from(
                    u8::try_from(x >> (64-8)).unwrap() ^ b)];
            }
            p64(x) + lo
        };

        // Binary exponentiation
        let mut a = p64(2);
        let mut bits = bits;
        let mut g = p64(1);
        loop {
            if bits & 1 != 0 {
                g = mul(g, a);
            }

            bits >>= 1;
            if bits == 0 {
                break;
            }
            a = mul(a, a);
        };

        // Final multiplication
        self.0 = u64::from(mul(p64(self.0), g));
    }

    pub fn skip_backwards(&mut self, bits: u64) {
        // Assuming our lfsr is well constructed, we're in a multiplicative
        // cycle with 2^width-1 elements. Which means backwards skips are the
        // same as skipping 2^width-1-(skip % 2^width-1) elements
        // 
        self.skip(NONZEROS - (bits % NONZEROS))
    }
}


/// An LFSR implemented using a smaller, 16-element, precomputed division
/// and remainder table
#[derive(Debug, Clone)]
pub struct Lfsr64SmallTable(u64);

impl Lfsr64SmallTable {
    // div/rem tables
    const DIV_TABLE: [u8; 16] = {
        let mut div_table = [0; 16];
        let mut i = 0;
        while i < div_table.len() {
            div_table[i] = p128((i as u128) << 64)
                .naive_div(POLYNOMIAL)
                .0 as u8;
            i += 1;
        }
        div_table
    };

    const REM_TABLE: [u64; 16] = {
        let mut rem_table = [0; 16];
        let mut i = 0;
        while i < rem_table.len() {
            rem_table[i] = p128((i as u128) << 64)
                .naive_rem(POLYNOMIAL)
                .0 as u64;
            i += 1;
        }
        rem_table
    };

    // inverse div/rem tables for iterating backwards
    const INVERSE_DIV_TABLE: [u8; 16] = {
        let mut div_table = [0; 16];
        let mut i = 0;
        while i < div_table.len() {
            div_table[i] = (p128((((i as u8).reverse_bits() >> 4) as u128) << 64)
                .naive_div(p128(POLYNOMIAL.0.reverse_bits() >> 63u64))
                .0 as u8)
                .reverse_bits() >> 4;
            i += 1;
        }
        div_table
    };

    const INVERSE_REM_TABLE: [u64; 16] = {
        let mut rem_table = [0; 16];
        let mut i = 0;
        while i < rem_table.len() {
            rem_table[i] = (p128((((i as u8).reverse_bits() >> 4) as u128) << 64)
                .naive_rem(p128(POLYNOMIAL.0.reverse_bits() >> 63u64))
                .0 as u64)
                .reverse_bits();
            i += 1;
        }
        rem_table
    };

    // implementation starts here
    pub fn new(mut seed: u64) -> Self {
        // make sure seed does not equal zero! otherwise our rng would only
        // ever output zero!
        if seed == 0 {
            seed = 1;
        }

        Self(seed)
    }

    pub fn next(&mut self, bits: u64) -> u64 {
        debug_assert!(bits <= 64);
        let mut x = self.0;
        let mut q = 0;
        for i in (0..bits/4).rev() {
            let n = min(4, bits-4*i);
            q = (q << n) | u64::from(Self::DIV_TABLE[usize::try_from(x >> (64-n)).unwrap()]);
            x = (x << n) ^ Self::REM_TABLE[usize::try_from(x >> (64-n)).unwrap()];
        }
        self.0 = x;
        q
    }

    pub fn prev(&mut self, bits: u64) -> u64 {
        debug_assert!(bits <= 64);
        let mut x = self.0;
        let mut q = 0;
        for i in 0..bits/4 {
            let n = min(4, bits-4*i);
            let m = (1 << n)-1;
            q |= u64::from(Self::INVERSE_DIV_TABLE[usize::try_from(x & m).unwrap()]) << (4*i);
            x = (x >> n) ^ Self::INVERSE_REM_TABLE[usize::try_from(x & m).unwrap()];
        }
        self.0 = x;
        q
    }

    pub fn skip(&mut self, bits: u64) {
        // Each step of the lfsr is equivalent to multiplication in a finite
        // field by a primitive element g=2, which means we can use exponents of
        // g=2 to efficiently jump around states of the lfsr.
        //
        // lfsr' = 2^skip
        // 
        let mul = |a: p64, b: p64| -> p64 {
            let (lo, hi) = a.widening_mul(b);
            let mut x = 0u64;
            for b in hi.to_be_bytes() {
                x = (x << 4) ^ Self::REM_TABLE[usize::from(
                    u8::try_from(x >> (64-4)).unwrap() ^ (b >> 4)) & 0xf];
                x = (x << 4) ^ Self::REM_TABLE[usize::from(
                    u8::try_from(x >> (64-4)).unwrap() ^ (b >> 0)) & 0xf];
            }
            p64(x) + lo
        };

        // Binary exponentiation
        let mut a = p64(2);
        let mut bits = bits;
        let mut g = p64(1);
        loop {
            if bits & 1 != 0 {
                g = mul(g, a);
            }

            bits >>= 1;
            if bits == 0 {
                break;
            }
            a = mul(a, a);
        };

        // Final multiplication
        self.0 = u64::from(mul(p64(self.0), g));
    }

    pub fn skip_backwards(&mut self, bits: u64) {
        // Assuming our lfsr is well constructed, we're in a multiplicative
        // cycle with 2^width-1 elements. Which means backwards skips are the
        // same as skipping 2^width-1-(skip % 2^width-1) elements
        // 
        self.skip(NONZEROS - (bits % NONZEROS))
    }
}


/// An LFSR implemented using Barret reduction
///
/// We can't use Barret for general division, so this only offers a marginal
/// speedup when generating bits. This does significant improve the efficiency
/// of skipping around the LFSR.
///
#[derive(Debug, Clone)]
pub struct Lfsr64Barret(p64);

impl Lfsr64Barret {
    const BARRET_CONSTANT: p64 = {
        p64(p128(POLYNOMIAL.0 << 64)
            .naive_div(POLYNOMIAL).0 as u64)
    };

    const INVERSE_BARRET_CONSTANT: p64 = {
        p64(p128((POLYNOMIAL.0.reverse_bits() >> 63) << 64)
            .naive_div(p128(POLYNOMIAL.0.reverse_bits() >> 63)).0 as u64)
    };

    // implementation starts here
    pub fn new(mut seed: u64) -> Self {
        // make sure seed does not equal zero! otherwise our rng would only
        // ever output zero!
        if seed == 0 {
            seed = 1;
        }

        Self(p64(seed))
    }

    pub fn next(&mut self, bits: u64) -> u64 {
        debug_assert!(bits <= 64);
        let x = p128::from(self.0) << bits;
        let q = x / POLYNOMIAL;
        let lo = p64::from_lossy(x);
        let hi = p64::try_from(x >> 64).unwrap();
        self.0 = lo + (hi.widening_mul(Self::BARRET_CONSTANT).1 + hi)
            .wrapping_mul(p64::from_lossy(POLYNOMIAL));
        u64::try_from(q.0).unwrap()
    }

    pub fn prev(&mut self, bits: u64) -> u64 {
        debug_assert!(bits <= 64);
        let x = p128::from(self.0.reverse_bits()) << bits;
        let q = x / (POLYNOMIAL.reverse_bits() >> 63u64);
        let lo = p64::from_lossy(x);
        let hi = p64::try_from(x >> 64).unwrap();
        self.0 = (lo + (hi.widening_mul(Self::INVERSE_BARRET_CONSTANT).1 + hi)
            .wrapping_mul(p64::from_lossy(POLYNOMIAL.reverse_bits() >> 63u64)))
            .reverse_bits();
        u64::try_from(q.0).unwrap().reverse_bits() >> (64-bits)
    }

    pub fn skip(&mut self, bits: u64) {
        // Each step of the lfsr is equivalent to multiplication in a finite
        // field by a primitive element g=2, which means we can use exponents of
        // g=2 to efficiently jump around states of the lfsr.
        //
        // lfsr' = 2^skip
        // 
        let mul = |a: p64, b: p64| -> p64 {
            let (lo, hi) = a.widening_mul(b);
            lo + (hi.widening_mul(Self::BARRET_CONSTANT).1 + hi)
                .wrapping_mul(p64::from_lossy(POLYNOMIAL))
        };

        // Binary exponentiation
        let mut a = p64(2);
        let mut bits = bits;
        let mut g = p64(1);
        loop {
            if bits & 1 != 0 {
                g = mul(g, a);
            }

            bits >>= 1;
            if bits == 0 {
                break;
            }
            a = mul(a, a);
        };

        // Final multiplication
        self.0 = mul(self.0, g);
    }

    pub fn skip_backwards(&mut self, bits: u64) {
        // Assuming our lfsr is well constructed, we're in a multiplicative
        // cycle with 2^width-1 elements. Which means backwards skips are the
        // same as skipping 2^width-1-(skip % 2^width-1) elements
        // 
        self.skip(NONZEROS - (bits % NONZEROS))
    }
}


/// An LFSR implemention combining Barret reduction with a division table
///
#[derive(Debug, Clone)]
pub struct Lfsr64TableBarret(p64);

impl Lfsr64TableBarret {
    // div/rem tables
    const DIV_TABLE: [u8; 256] = {
        let mut div_table = [0; 256];
        let mut i = 0;
        while i < div_table.len() {
            div_table[i] = p128((i as u128) << 64)
                .naive_div(POLYNOMIAL)
                .0 as u8;
            i += 1;
        }
        div_table
    };

    const INVERSE_DIV_TABLE: [u8; 256] = {
        let mut div_table = [0; 256];
        let mut i = 0;
        while i < div_table.len() {
            div_table[i] = (p128((i as u128) << 64)
                .naive_div(p128(POLYNOMIAL.0.reverse_bits() >> 63u64))
                .0 as u8)
                .reverse_bits();
            i += 1;
        }
        div_table
    };

    // Barret constants
    const BARRET_CONSTANT: p64 = {
        p64(p128(POLYNOMIAL.0 << 64)
            .naive_div(POLYNOMIAL).0 as u64)
    };

    const INVERSE_BARRET_CONSTANT: p64 = {
        p64(p128((POLYNOMIAL.0.reverse_bits() >> 63) << 64)
            .naive_div(p128(POLYNOMIAL.0.reverse_bits() >> 63)).0 as u64)
    };

    // implementation starts here
    pub fn new(mut seed: u64) -> Self {
        // make sure seed does not equal zero! otherwise our rng would only
        // ever output zero!
        if seed == 0 {
            seed = 1;
        }

        Self(p64(seed))
    }

    pub fn next(&mut self, bits: u64) -> u64 {
        debug_assert!(bits <= 64);
        let mut x = self.0;
        let mut q = 0;
        for i in (0..bits/8).rev() {
            let n = min(8, bits-8*i);
            q = (q << n) | u64::from(Self::DIV_TABLE[usize::try_from(x >> (64-n)).unwrap()]);
            x = (x << n) + ((x >> (64-n)).widening_mul(Self::BARRET_CONSTANT).1 + (x >> (64-n)))
                .wrapping_mul(p64::from_lossy(POLYNOMIAL));
        }
        self.0 = x;
        q
    }

    pub fn prev(&mut self, bits: u64) -> u64 {
        debug_assert!(bits <= 64);
        let mut x = self.0.reverse_bits();
        let mut q = 0;
        for i in (0..bits/8).rev() {
            let n = min(8, bits-8*i);
            q |= u64::from(Self::INVERSE_DIV_TABLE[usize::try_from(x >> (64-n)).unwrap()]) << (bits-8-8*i);
            x = (x << n) + ((x >> (64-n)).widening_mul(Self::INVERSE_BARRET_CONSTANT).1 + (x >> (64-n)))
                .wrapping_mul(p64::from_lossy(POLYNOMIAL.reverse_bits() >> 63u64));
        }
        self.0 = x.reverse_bits();
        q
    }

    pub fn skip(&mut self, bits: u64) {
        // Each step of the lfsr is equivalent to multiplication in a finite
        // field by a primitive element g=2, which means we can use exponents of
        // g=2 to efficiently jump around states of the lfsr.
        //
        // lfsr' = 2^skip
        // 
        let mul = |a: p64, b: p64| -> p64 {
            let (lo, hi) = a.widening_mul(b);
            lo + (hi.widening_mul(Self::BARRET_CONSTANT).1 + hi)
                .wrapping_mul(p64::from_lossy(POLYNOMIAL))
        };

        // Binary exponentiation
        let mut a = p64(2);
        let mut bits = bits;
        let mut g = p64(1);
        loop {
            if bits & 1 != 0 {
                g = mul(g, a);
            }

            bits >>= 1;
            if bits == 0 {
                break;
            }
            a = mul(a, a);
        };

        // Final multiplication
        self.0 = mul(self.0, g);
    }

    pub fn skip_backwards(&mut self, bits: u64) {
        // Assuming our lfsr is well constructed, we're in a multiplicative
        // cycle with 2^width-1 elements. Which means backwards skips are the
        // same as skipping 2^width-1-(skip % 2^width-1) elements
        // 
        self.skip(NONZEROS - (bits % NONZEROS))
    }
}

/// An LFSR implemention combining Barret reduction with a small division table
///
#[derive(Debug, Clone)]
pub struct Lfsr64SmallTableBarret(p64);

impl Lfsr64SmallTableBarret {
    // div/rem tables
    const DIV_TABLE: [u8; 16] = {
        let mut div_table = [0; 16];
        let mut i = 0;
        while i < div_table.len() {
            div_table[i] = p128((i as u128) << 64)
                .naive_div(POLYNOMIAL)
                .0 as u8;
            i += 1;
        }
        div_table
    };

    const INVERSE_DIV_TABLE: [u8; 16] = {
        let mut div_table = [0; 16];
        let mut i = 0;
        while i < div_table.len() {
            div_table[i] = (p128((i as u128) << 64)
                .naive_div(p128(POLYNOMIAL.0.reverse_bits() >> 63u64))
                .0 as u8)
                .reverse_bits() >> 4;
            i += 1;
        }
        div_table
    };

    // Barret constants
    const BARRET_CONSTANT: p64 = {
        p64(p128(POLYNOMIAL.0 << 64)
            .naive_div(POLYNOMIAL).0 as u64)
    };

    const INVERSE_BARRET_CONSTANT: p64 = {
        p64(p128((POLYNOMIAL.0.reverse_bits() >> 63) << 64)
            .naive_div(p128(POLYNOMIAL.0.reverse_bits() >> 63)).0 as u64)
    };

    // implementation starts here
    pub fn new(mut seed: u64) -> Self {
        // make sure seed does not equal zero! otherwise our rng would only
        // ever output zero!
        if seed == 0 {
            seed = 1;
        }

        Self(p64(seed))
    }

    pub fn next(&mut self, bits: u64) -> u64 {
        debug_assert!(bits <= 64);
        let mut x = self.0;
        let mut q = 0;
        for i in (0..bits/4).rev() {
            let n = min(4, bits-4*i);
            q = (q << n) | u64::from(Self::DIV_TABLE[usize::try_from(x >> (64-n)).unwrap()]);
            x = (x << n) + ((x >> (64-n)).widening_mul(Self::BARRET_CONSTANT).1 + (x >> (64-n)))
                .wrapping_mul(p64::from_lossy(POLYNOMIAL));
        }
        self.0 = x;
        q
    }

    pub fn prev(&mut self, bits: u64) -> u64 {
        debug_assert!(bits <= 64);
        let mut x = self.0.reverse_bits();
        let mut q = 0;
        for i in (0..bits/4).rev() {
            let n = min(4, bits-4*i);
            q |= u64::from(Self::INVERSE_DIV_TABLE[usize::try_from(x >> (64-n)).unwrap()]) << (bits-4-4*i);
            x = (x << n) + ((x >> (64-n)).widening_mul(Self::INVERSE_BARRET_CONSTANT).1 + (x >> (64-n)))
                .wrapping_mul(p64::from_lossy(POLYNOMIAL.reverse_bits() >> 63u64));
        }
        self.0 = x.reverse_bits();
        q
    }

    pub fn skip(&mut self, bits: u64) {
        // Each step of the lfsr is equivalent to multiplication in a finite
        // field by a primitive element g=2, which means we can use exponents of
        // g=2 to efficiently jump around states of the lfsr.
        //
        // lfsr' = 2^skip
        // 
        let mul = |a: p64, b: p64| -> p64 {
            let (lo, hi) = a.widening_mul(b);
            lo + (hi.widening_mul(Self::BARRET_CONSTANT).1 + hi)
                .wrapping_mul(p64::from_lossy(POLYNOMIAL))
        };

        // Binary exponentiation
        let mut a = p64(2);
        let mut bits = bits;
        let mut g = p64(1);
        loop {
            if bits & 1 != 0 {
                g = mul(g, a);
            }

            bits >>= 1;
            if bits == 0 {
                break;
            }
            a = mul(a, a);
        };

        // Final multiplication
        self.0 = mul(self.0, g);
    }

    pub fn skip_backwards(&mut self, bits: u64) {
        // Assuming our lfsr is well constructed, we're in a multiplicative
        // cycle with 2^width-1 elements. Which means backwards skips are the
        // same as skipping 2^width-1-(skip % 2^width-1) elements
        // 
        self.skip(NONZEROS - (bits % NONZEROS))
    }
}


fn main() {
    fn hex(xs: &[u8]) -> String {
        xs.iter()
            .map(|x| format!("{:02x}", x))
            .collect()
    }

    fn grid<'a>(width: usize, bs: &'a [u8]) -> impl Iterator<Item=String> + 'a {
        (0 .. (bs.len()+width-1)/width)
            .step_by(2)
            .rev()
            .map(move |y| {
                let mut line = String::new();
                for x in 0..width {
                    let mut b = 0;
                    for i in 0..2 {
                        if bs.get((y+i)*width + x).filter(|b| **b != 0).is_some() {
                            b |= 1 << (1-i);
                        }
                    }
                    line.push(match b {
                        0x0 => ' ',
                        0x1 => '\'',
                        0x2 => '.',
                        0x3 => ':',
                        _ => unreachable!(),
                    });
                }
                line
            })
    }


    println!();
    println!("testing lfsr64");

    // Test normal iteration

    let mut lfsr64_naive = Lfsr64Naive::new(1);
    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_naive.next(8)).unwrap();
    }
    println!("{:<35} => {}", "lfsr64_naive", hex(&buffer));

    let mut lfsr64_divrem = Lfsr64DivRem::new(1);
    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_divrem.next(8)).unwrap();
    }
    println!("{:<35} => {}", "lfsr64_divrem", hex(&buffer));

    let mut lfsr64_table = Lfsr64Table::new(1);
    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_table.next(8)).unwrap();
    }
    println!("{:<35} => {}", "lfsr64_table", hex(&buffer));

    let mut lfsr64_small_table = Lfsr64SmallTable::new(1);
    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_small_table.next(8)).unwrap();
    }
    println!("{:<35} => {}", "lfsr64_small_table", hex(&buffer));

    let mut lfsr64_barret = Lfsr64Barret::new(1);
    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_barret.next(8)).unwrap();
    }
    println!("{:<35} => {}", "lfsr64_barret", hex(&buffer));

    let mut lfsr64_table_barret = Lfsr64TableBarret::new(1);
    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_table_barret.next(8)).unwrap();
    }
    println!("{:<35} => {}", "lfsr64_table_barret", hex(&buffer));

    let mut lfsr64_small_table_barret = Lfsr64SmallTableBarret::new(1);
    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_small_table_barret.next(8)).unwrap();
    }
    println!("{:<35} => {}", "lfsr64_small_table_barret", hex(&buffer));


    // Test reverse iteration

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_naive.prev(8)).unwrap();
    }
    println!("{:<35} => {}", "rev(lfsr64_naive)", hex(&buffer));

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_divrem.prev(8)).unwrap();
    }
    println!("{:<35} => {}", "rev(lfsr64_divrem)", hex(&buffer));

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_table.prev(8)).unwrap();
    }
    println!("{:<35} => {}", "rev(lfsr64_table)", hex(&buffer));

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_small_table.prev(8)).unwrap();
    }
    println!("{:<35} => {}", "rev(lfsr64_small_table)", hex(&buffer));

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_barret.prev(8)).unwrap();
    }
    println!("{:<35} => {}", "rev(lfsr64_barret)", hex(&buffer));

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_table_barret.prev(8)).unwrap();
    }
    println!("{:<35} => {}", "rev(lfsr64_table_barret)", hex(&buffer));

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_small_table_barret.prev(8)).unwrap();
    }
    println!("{:<35} => {}", "rev(lfsr64_small_table_barret)", hex(&buffer));


    // What about 9000 steps in the future?

    lfsr64_naive.skip(9000);

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_naive.next(8)).unwrap();
    }
    println!("{:<35} => {}", "lfsr64_naive+9000", hex(&buffer));

    lfsr64_divrem.skip(9000);

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_divrem.next(8)).unwrap();
    }
    println!("{:<35} => {}", "lfsr64_divrem+9000", hex(&buffer));

    lfsr64_table.skip(9000);

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_table.next(8)).unwrap();
    }
    println!("{:<35} => {}", "lfsr64_table+9000", hex(&buffer));

    lfsr64_small_table.skip(9000);

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_small_table.next(8)).unwrap();
    }
    println!("{:<35} => {}", "lfsr64_small_table+9000", hex(&buffer));

    lfsr64_barret.skip(9000);

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_barret.next(8)).unwrap();
    }
    println!("{:<35} => {}", "lfsr64_barret+9000", hex(&buffer));

    lfsr64_table_barret.skip(9000);

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_table_barret.next(8)).unwrap();
    }
    println!("{:<35} => {}", "lfsr64_table_barret+9000", hex(&buffer));

    lfsr64_small_table_barret.skip(9000);

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_small_table_barret.next(8)).unwrap();
    }
    println!("{:<35} => {}", "lfsr64_small_table_barret+9000", hex(&buffer));


    // And reverse

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_naive.prev(8)).unwrap();
    }
    println!("{:<35} => {}", "rev(lfsr64_naive+9000)", hex(&buffer));

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_divrem.prev(8)).unwrap();
    }
    println!("{:<35} => {}", "rev(lfsr64_divrem+9000)", hex(&buffer));

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_table.prev(8)).unwrap();
    }
    println!("{:<35} => {}", "rev(lfsr64_table+9000)", hex(&buffer));

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_small_table.prev(8)).unwrap();
    }
    println!("{:<35} => {}", "rev(lfsr64_small_table+9000)", hex(&buffer));

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_barret.prev(8)).unwrap();
    }
    println!("{:<35} => {}", "rev(lfsr64_barret+9000)", hex(&buffer));

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_table_barret.prev(8)).unwrap();
    }
    println!("{:<35} => {}", "rev(lfsr64_table_barret+9000)", hex(&buffer));

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_small_table_barret.prev(8)).unwrap();
    }
    println!("{:<35} => {}", "rev(lfsr64_small_table_barret+9000)", hex(&buffer));


    // And then skip back to the beginning

    lfsr64_naive.skip_backwards(9000);

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_naive.next(8)).unwrap();
    }
    println!("{:<35} => {}", "lfsr64_naive+9000-9000", hex(&buffer));

    lfsr64_divrem.skip_backwards(9000);

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_divrem.next(8)).unwrap();
    }
    println!("{:<35} => {}", "lfsr64_divrem+9000-9000", hex(&buffer));

    lfsr64_table.skip_backwards(9000);

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_table.next(8)).unwrap();
    }
    println!("{:<35} => {}", "lfsr64_table+9000-9000", hex(&buffer));

    lfsr64_small_table.skip_backwards(9000);

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_small_table.next(8)).unwrap();
    }
    println!("{:<35} => {}", "lfsr64_small_table+9000-9000", hex(&buffer));

    lfsr64_barret.skip_backwards(9000);

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_barret.next(8)).unwrap();
    }
    println!("{:<35} => {}", "lfsr64_barret+9000-9000", hex(&buffer));

    lfsr64_table_barret.skip_backwards(9000);

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_table_barret.next(8)).unwrap();
    }
    println!("{:<35} => {}", "lfsr64_table_barret+9000-9000", hex(&buffer));

    lfsr64_small_table_barret.skip_backwards(9000);

    let mut buffer = [0u8; 32];
    for i in 0..buffer.len() {
        buffer[i] = u8::try_from(lfsr64_small_table_barret.next(8)).unwrap();
    }
    println!("{:<35} => {}", "lfsr64_small_table_barret+9000-9000", hex(&buffer));

    println!();
    

    // For our final trick, lets render our randomness on a 2d-grid, this
    // representation makes is much easier for us humans to see patterns in
    // the data.
    //
    // Uniform distributions are boring, lets show a rough triangle
    // distribution distribution, X = Y+Z where Y and Z are uniform (our prng)
    //
    const SAMPLES: usize = 2048;
    const WIDTH: usize = 64;
    const HEIGHT: usize = 32;

    println!("{} lfsr64 samples:", SAMPLES);
    let mut lfsr = Lfsr64Table::new(1);

    let samples = iter::repeat_with(|| lfsr.next(64)).take(SAMPLES).collect::<Vec<_>>();
    let mut buffer = [0u8; WIDTH*HEIGHT];
    for i in (0..SAMPLES).step_by(4) {
        let x = (samples[i+0] as usize % (WIDTH/2))  + (samples[i+1] as usize % (WIDTH/2));
        let y = (samples[i+2] as usize % (HEIGHT/2)) + (samples[i+3] as usize % (HEIGHT/2));
        buffer[x+y*WIDTH] = buffer[x+y*WIDTH].saturating_add(1);
    }

    let mut x_dist = [0u8; 4*WIDTH];
    let mut x_max = 0;
    for x in 0..WIDTH {
        x_max = max(x_max, (0..HEIGHT).map(|y| u32::from(buffer[x+y*WIDTH])).sum());
    }
    for x in 0..WIDTH {
        let v: u32 = (0..HEIGHT).map(|y| u32::from(buffer[x+y*WIDTH])).sum();
        let v = (4*v+x_max-1) / x_max;
        for i in 0..usize::try_from(v).unwrap() {
            x_dist[x+i*WIDTH] = 1;
        }
    }

    let mut y_dist = [0u8; 4*HEIGHT];
    let mut y_max = 0;
    for y in 0..HEIGHT {
        y_max = max(y_max, (0..WIDTH).map(|x| u32::from(buffer[x+y*WIDTH])).sum());
    }
    for y in 0..HEIGHT {
        let v: u32 = (0..WIDTH).map(|x| u32::from(buffer[x+y*WIDTH])).sum();
        let v = (4*v+y_max-1) / y_max;
        for i in 0..usize::try_from(v).unwrap() {
            y_dist[(3-i)+y*4] = 1;
        }
    }

    for (line, y_dist_line) in grid(WIDTH, &buffer).zip(grid(4, &y_dist)) {
        println!("    {}  {}", line, y_dist_line);
    }
    println!();

    for x_dist_line in grid(WIDTH, &x_dist) {
        println!("    {}", x_dist_line);
    }
    println!();

    // other stats
    let ones: u32 = samples.iter().map(|x| x.count_ones()).sum();
    let zeros: u32 = samples.iter().map(|x| x.count_zeros()).sum();
    let mut comp = DeflateEncoder::new(Vec::new(), Compression::best());
    let bytes = unsafe { slice::from_raw_parts(samples.as_ptr() as *const u8, 8*samples.len()) };
    comp.write_all(&bytes).unwrap();
    let comp = comp.finish().unwrap();
    println!("{}/{} ones ({:.2}%), {:.2}% compressability",
        ones,
        ones + zeros,
        100.0 * (ones as f64 / (ones as f64 + zeros as f64)),
        100.0 * ((bytes.len() as f64 - comp.len() as f64) / bytes.len() as f64),
    );
    println!();
}
