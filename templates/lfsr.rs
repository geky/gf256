//! Template for LFSR structs
//!
//! See examples/lfsr.rs for a more detailed explanation of
//! where these implementations come from

use __crate::internal::rand::RngCore;
use __crate::internal::rand::SeedableRng;
use __crate::internal::cfg_if::cfg_if;
use __crate::traits::FromLossy;
use __crate::traits::TryFrom;
use core::iter::FusedIterator;
use core::mem::size_of;
use core::cmp::min;


/// A linear-feedback shift register.
///
/// ``` rust
/// # use std::iter;
/// use gf256::lfsr::Lfsr16;
///
/// let mut lfsr = Lfsr16::new(1);
/// assert_eq!(lfsr.next(16), 0x0001);
/// assert_eq!(lfsr.next(16), 0x002d);
/// assert_eq!(lfsr.next(16), 0x0451);
/// assert_eq!(lfsr.next(16), 0xbdad);
/// assert_eq!(lfsr.prev(16), 0xbdad);
/// assert_eq!(lfsr.prev(16), 0x0451);
/// assert_eq!(lfsr.prev(16), 0x002d);
/// assert_eq!(lfsr.prev(16), 0x0001);
/// ```
///
/// See the [module-level documentation](../lfsr) for more info.
///
#[derive(Debug, Clone)]
pub struct __lfsr(__nzu);

impl __lfsr {
    /// The irreducible polynomial that defines the LFSR.
    pub const POLYNOMIAL: __p2 = __p2(__polynomial);

    /// Number of non-zero elements in the field, this which is also
    /// the maximum cycle-length of the LFSR.
    pub const NONZEROS: __u = __nonzeros;

    // div/rem tables, if required
    #[cfg(__if(__table || __table_barret))]
    const DIV_TABLE: [u8; 256] = {
        let mut div_table = [0; 256];
        let mut i = 0;
        // TODO make this consistent in both gf and crc?
        while i < div_table.len() {
            div_table[i] = __p2((i as __u2) << (8*size_of::<__u>()))
                .naive_div(__p2(__polynomial << (8*size_of::<__u>()-__width)))
                .0 as u8;
            i += 1;
        }
        div_table
    };
    #[cfg(__if(__table || __table_skip))]
    const REM_TABLE: [__u; 256] = {
        let mut rem_table = [0; 256];
        let mut i = 0;
        while i < rem_table.len() {
            rem_table[i] = __p2((i as __u2) << (8*size_of::<__u>()))
                .naive_rem(__p2(__polynomial << (8*size_of::<__u>()-__width)))
                .0 as __u;
            i += 1;
        }
        rem_table
    };

    // inverse div/rem tables for iterating backwards
    #[cfg(__if(__table || __table_barret))]
    const INVERSE_DIV_TABLE: [u8; 256] = {
        let mut div_table = [0; 256];
        let mut i = 0;
        while i < div_table.len() {
            cfg_if! {
                if #[cfg(__if(__table_barret))] {
                    div_table[i] = (__p2((i as __u2) << (8*size_of::<__u>()))
                        .naive_div(__p2(__inverse_polynomial << (8*size_of::<__u>()-__width)))
                        .0 as u8)
                        .reverse_bits();
                } else {
                    div_table[i] = (__p2(((i as u8).reverse_bits() as __u2) << (8*size_of::<__u>()))
                        .naive_div(__p2(__inverse_polynomial << (8*size_of::<__u>()-__width)))
                        .0 as u8)
                        .reverse_bits();
                }
            }
            i += 1;
        }
        div_table
    };
    #[cfg(__if(__table))]
    const INVERSE_REM_TABLE: [__u; 256] = {
        let mut rem_table = [0; 256];
        let mut i = 0;
        while i < rem_table.len() {
            rem_table[i] = (__p2(((i as u8).reverse_bits() as __u2) << (8*size_of::<__u>()))
                .naive_rem(__p2(__inverse_polynomial << (8*size_of::<__u>()-__width)))
                .0 as __u)
                .reverse_bits();
            i += 1;
        }
        rem_table
    };

    // small div/rem tables, if required
    #[cfg(__if(__small_table || __small_table_barret))]
    const DIV_TABLE: [u8; 16] = {
        let mut div_table = [0; 16];
        let mut i = 0;
        while i < div_table.len() {
            div_table[i] = __p2((i as __u2) << (8*size_of::<__u>()))
                .naive_div(__p2(__polynomial << (8*size_of::<__u>()-__width)))
                .0 as u8;
            i += 1;
        }
        div_table
    };
    #[cfg(__if(__small_table || __small_table_skip))]
    const REM_TABLE: [__u; 16] = {
        let mut rem_table = [0; 16];
        let mut i = 0;
        while i < rem_table.len() {
            rem_table[i] = __p2((i as __u2) << (8*size_of::<__u>()))
                .naive_rem(__p2(__polynomial << (8*size_of::<__u>()-__width)))
                .0 as __u;
            i += 1;
        }
        rem_table
    };

    // small inverse div/rem tables for iterating backwards
    #[cfg(__if(__small_table || __small_table_barret))]
    const INVERSE_DIV_TABLE: [u8; 16] = {
        let mut div_table = [0; 16];
        let mut i = 0;
        while i < div_table.len() {
            cfg_if! {
                if #[cfg(__if(__small_table_barret))] {
                    div_table[i] = (__p2((i as __u2) << (8*size_of::<__u>()))
                        .naive_div(__p2(__inverse_polynomial << (8*size_of::<__u>()-__width)))
                        .0 as u8)
                        .reverse_bits() >> 4;
                } else {
                    div_table[i] = (__p2((((i as u8).reverse_bits() >> 4) as __u2) << (8*size_of::<__u>()))
                        .naive_div(__p2(__inverse_polynomial << (8*size_of::<__u>()-__width)))
                        .0 as u8)
                        .reverse_bits() >> 4;
                }
            }
            i += 1;
        }
        div_table
    };
    #[cfg(__if(__small_table))]
    const INVERSE_REM_TABLE: [__u; 16] = {
        let mut rem_table = [0; 16];
        let mut i = 0;
        while i < rem_table.len() {
            rem_table[i] = (__p2((((i as u8).reverse_bits() >> 4) as __u2) << (8*size_of::<__u>()))
                .naive_rem(__p2(__inverse_polynomial << (8*size_of::<__u>()-__width)))
                .0 as __u)
                .reverse_bits();
            i += 1;
        }
        rem_table
    };

    // Barret constants, if required
    #[cfg(__if(__barret || __table_barret || __small_table_barret || __barret_skip))]
    const BARRET_CONSTANT: __p = {
        __p(
            __p2((__polynomial & __nonzeros) << (8*size_of::<__u>() + 8*size_of::<__u>()-__width))
                .naive_div(__p2(__polynomial << (8*size_of::<__u>()-__width)))
                .0 as __u
        )
    };
    #[cfg(__if(__barret || __table_barret || __small_table_barret))]
    const INVERSE_BARRET_CONSTANT: __p = {
        __p(
            __p2((__inverse_polynomial & __nonzeros) << (8*size_of::<__u>() + 8*size_of::<__u>()-__width))
                .naive_div(__p2(__inverse_polynomial << (8*size_of::<__u>()-__width)))
                .0 as __u
        )
    };

    /// Create an LFSR with the given seed.
    ///
    /// The seed can't be `0`, so if `0` is provided, the seed `1` is used
    /// instead.
    ///
    #[inline]
    pub const fn new(mut seed: __u) -> Self {
        seed = seed & __nonzeros;

        // make sure seed does not equal zero! otherwise our rng would only
        // ever output zero!
        if seed == 0 {
            seed = 1;
        }

        cfg_if! {
            if #[cfg(__if(__reflected))] {
                seed = seed.reverse_bits() >> (8*size_of::<__u>()-__width);
            }
        }

        Self(unsafe { __nzu::new_unchecked(seed) })
    }

    /// Generate the next n-bits of pseudo-random data.
    ///
    /// ``` rust
    /// # use ::gf256::lfsr::*;
    /// let mut lfsr = Lfsr16::new(1);
    /// assert_eq!(lfsr.next(16), 0x0001);
    /// assert_eq!(lfsr.next(16), 0x002d);
    /// assert_eq!(lfsr.next(16), 0x0451);
    /// assert_eq!(lfsr.next(16), 0xbdad);
    /// ```
    /// 
    #[inline]
    pub fn next(&mut self, bits: __u) -> __u {
        debug_assert!(bits <= __width);
        let bits = bits as usize;
        cfg_if! {
            if #[cfg(__if(__naive))] {
                // naive lfsr using bitshifts and xors
                let mut x = __u::from(self.0);
                let mut q = 0;
                for _ in 0..bits {
                    let msb = x >> (__width-1);
                    q = (q << 1) | msb;
                    x = (x << 1) ^ if msb != 0 {
                        __polynomial as __u
                    } else {
                        0
                    };
                }
                // update state
                self.0 = __nzu::try_from(x).unwrap();
            } else if #[cfg(__if(__table))] {
                // lfsr with a per-byte division and remainder table
                let mut x = __u::from(self.0) << (8*size_of::<__u>()-__width);
                let mut q = 0;
                for i in (0..(bits+7)/8).rev() {
                    let n = min(8, bits-8*i);
                    if n == 8*size_of::<__u>() {
                        q = __u::from(Self::DIV_TABLE[usize::try_from(
                            x >> (8*size_of::<__u>()-n)).unwrap()]);
                        x = Self::REM_TABLE[usize::try_from(
                            x >> (8*size_of::<__u>()-n)).unwrap()];
                    } else {
                        q = (q << n) | __u::from(Self::DIV_TABLE[usize::try_from(
                            x >> (8*size_of::<__u>()-n)).unwrap()]);
                        x = (x << n) ^ Self::REM_TABLE[usize::try_from(
                            x >> (8*size_of::<__u>()-n)).unwrap()];
                    }
                }
                // adjust for alignment
                x = x >> (8*size_of::<__u>()-__width);
                // update state
                self.0 = __nzu::try_from(x).unwrap();
            } else if #[cfg(__if(__small_table))] {
                // lfsr with a per-nibble division and remainder table
                let mut x = __u::from(self.0) << (8*size_of::<__u>()-__width);
                let mut q = 0;
                for i in (0..(bits+3)/4).rev() {
                    let n = min(4, bits-4*i);
                    q = (q << n) | __u::from(Self::DIV_TABLE[usize::try_from(
                        x >> (8*size_of::<__u>()-n)).unwrap()]);
                    x = (x << n) ^ Self::REM_TABLE[usize::try_from(
                        x >> (8*size_of::<__u>()-n)).unwrap()];
                }
                // adjust for alignment
                x = x >> (8*size_of::<__u>()-__width);
                // update state
                self.0 = __nzu::try_from(x).unwrap();
            } else if #[cfg(__if(__barret))] {
                // lfsr using naive division with Barret-reduction
                let x = __p2::from(__u::from(self.0)) << (bits + (8*size_of::<__u>()-__width));
                let q = x / __p2(__polynomial << (8*size_of::<__u>()-__width));
                let lo = __p::from_lossy(x);
                let hi = __p::try_from(x >> (8*size_of::<__u>())).unwrap();
                let mut x = lo + (hi.widening_mul(Self::BARRET_CONSTANT).1 + hi)
                    .wrapping_mul(__p((__polynomial & __nonzeros) << (8*size_of::<__u>()-__width)));
                // adjust for alignment
                x = x >> (8*size_of::<__u>()-__width);
                // update state
                self.0 = __nzu::try_from(__u::from(x)).unwrap();
                let q = __u::try_from(q.0).unwrap();
            } else if #[cfg(__if(__table_barret))] {
                // lfsr using a per-byte division table with Barret-reduction
                let mut x = __p::from(__u::from(self.0)) << (8*size_of::<__u>()-__width);
                let mut q = 0;
                for i in (0..(bits+7)/8).rev() {
                    let n = min(8, bits-8*i);
                    if n == 8*size_of::<__u>() {
                        q = __u::from(Self::DIV_TABLE[usize::try_from(
                            x >> (8*size_of::<__u>()-n)).unwrap()]);
                        let hi = x >> (8*size_of::<__u>()-n);
                        x = (hi.widening_mul(Self::BARRET_CONSTANT).1 + hi)
                            .wrapping_mul(__p((__polynomial & __nonzeros) << (8*size_of::<__u>()-__width)));
                    } else {
                        q = (q << n) | __u::from(Self::DIV_TABLE[usize::try_from(
                            x >> (8*size_of::<__u>()-n)).unwrap()]);
                        let lo = x << n;
                        let hi = x >> (8*size_of::<__u>()-n);
                        x = lo + (hi.widening_mul(Self::BARRET_CONSTANT).1 + hi)
                            .wrapping_mul(__p((__polynomial & __nonzeros) << (8*size_of::<__u>()-__width)));
                    }
                }
                // adjust for alignment
                x = x >> (8*size_of::<__u>()-__width);
                // update state
                self.0 = __nzu::try_from(__u::from(x)).unwrap();
            } else if #[cfg(__if(__small_table_barret))] {
                // lfsr using a per-nibble division table with Barret-reduction
                let mut x = __p::from(__u::from(self.0)) << (8*size_of::<__u>()-__width);
                let mut q = 0;
                for i in (0..(bits+3)/4).rev() {
                    let n = min(4, bits-4*i);
                    q = (q << n) | __u::from(Self::DIV_TABLE[usize::try_from(
                        x >> (8*size_of::<__u>()-n)).unwrap()]);
                    let lo = x << n;
                    let hi = x >> (8*size_of::<__u>()-n);
                    x = lo + (hi.widening_mul(Self::BARRET_CONSTANT).1 + hi)
                        .wrapping_mul(__p((__polynomial & __nonzeros) << (8*size_of::<__u>()-__width)));
                }
                // adjust for alignment
                x = x >> (8*size_of::<__u>()-__width);
                // update state
                self.0 = __nzu::try_from(__u::from(x)).unwrap();
            }
        }

        cfg_if! {
            if #[cfg(__if(__reflected))] {
                q.reverse_bits() >> (8*size_of::<__u>() - bits)
            } else {
                q
            }
        }
    }

    /// Generate the previous n-bits of pseudo-random data.
    ///
    /// ``` rust
    /// # use ::gf256::lfsr::*;
    /// let mut lfsr = Lfsr16::new(1);
    /// lfsr.skip(64);
    /// assert_eq!(lfsr.prev(16), 0xbdad);
    /// assert_eq!(lfsr.prev(16), 0x0451);
    /// assert_eq!(lfsr.prev(16), 0x002d);
    /// assert_eq!(lfsr.prev(16), 0x0001);
    /// ```
    /// 
    #[inline]
    pub fn prev(&mut self, bits: __u) -> __u {
        debug_assert!(bits <= __width);
        let bits = bits as usize;
        cfg_if! {
            if #[cfg(__if(__naive))] {
                // naive lfsr using bitshifts and xors
                let mut x = __u::from(self.0);
                let mut q = 0;
                for _ in 0..bits {
                    let lsb = x & 1;
                    q = (q >> 1) | (lsb << (bits-1));
                    x = (x >> 1) ^ if lsb != 0 {
                        ((__polynomial as __u2) >> 1) as __u
                    } else {
                        0
                    };
                }
                // update state
                self.0 = __nzu::try_from(x).unwrap();
            } else if #[cfg(__if(__table))] {
                // lfsr with a per-byte division and remainder table
                let mut x = __u::from(self.0);
                let mut q = 0;
                for i in (0..(bits+7)/8).rev() {
                    let n = min(8, bits-8*i);
                    if n == 8*size_of::<__u>() {
                        q = __u::from(Self::INVERSE_DIV_TABLE[usize::try_from(
                            x).unwrap()]);
                        x = Self::INVERSE_REM_TABLE[usize::try_from(
                            x).unwrap()];
                    } else {
                        q = (q >> n) | (__u::from(Self::INVERSE_DIV_TABLE[usize::try_from(
                            (x << (8-n)) & 0xff).unwrap()]) << (8*size_of::<__u>()-8));
                        x = (x >> n) ^ Self::INVERSE_REM_TABLE[usize::try_from(
                            (x << (8-n)) & 0xff).unwrap()];
                    }
                }
                // update state
                self.0 = __nzu::try_from(x).unwrap();
                let q = q >> (8*size_of::<__u>() - bits);
            } else if #[cfg(__if(__small_table))] {
                // lfsr with a per-nibble division and remainder table
                let mut x = __u::from(self.0);
                let mut q = 0;
                for i in (0..(bits+3)/4).rev() {
                    let n = min(4, bits-4*i);
                    q = (q >> n) | (__u::from(Self::INVERSE_DIV_TABLE[usize::try_from(
                        (x << (4-n)) & 0xf).unwrap()]) << (8*size_of::<__u>()-4));
                    x = (x >> n) ^ Self::INVERSE_REM_TABLE[usize::try_from(
                        (x << (4-n)) & 0xf).unwrap()];
                }
                // update state
                self.0 = __nzu::try_from(x).unwrap();
                let q = q >> (8*size_of::<__u>() - bits);
            } else if #[cfg(__if(__barret))] {
                // lfsr using naive division with Barret-reduction
                let x = __p2::from(__u::from(self.0).reverse_bits()) << bits;
                let q = x / __p2(__inverse_polynomial << (8*size_of::<__u>()-__width));
                let lo = __p::from_lossy(x);
                let hi = __p::try_from(x >> (8*size_of::<__u>())).unwrap();
                let mut x = lo + (hi.widening_mul(Self::INVERSE_BARRET_CONSTANT).1 + hi)
                        .wrapping_mul(__p((__inverse_polynomial & __nonzeros) << (8*size_of::<__u>()-__width)));
                // adjust for alignment
                x = x >> (8*size_of::<__u>()-__width);
                x = x.reverse_bits() >> (8*size_of::<__u>()-__width);
                // update state
                self.0 = __nzu::try_from(__u::from(x)).unwrap();
                let q = __u::try_from(q.0).unwrap().reverse_bits() >> (8*size_of::<__u>() - bits);
            } else if #[cfg(__if(__table_barret))] {
                // lfsr using a per-byte division table with Barret-reduction
                let mut x = __p::from(__u::from(self.0).reverse_bits());
                let mut q = 0;
                for i in (0..(bits+7)/8).rev() {
                    let n = min(8, bits-8*i);
                    if n == 8*size_of::<__u>() {
                        q = __u::from(Self::INVERSE_DIV_TABLE[usize::try_from(
                            x >> (8*size_of::<__u>()-n)).unwrap()]);
                        let hi = x >> (8*size_of::<__u>()-n);
                        x = (hi.widening_mul(Self::INVERSE_BARRET_CONSTANT).1 + hi)
                            .wrapping_mul(__p((__inverse_polynomial & __nonzeros) << (8*size_of::<__u>()-__width)));
                    } else {
                        q = (q >> n) | (__u::from(Self::INVERSE_DIV_TABLE[usize::try_from(
                            x >> (8*size_of::<__u>()-n)).unwrap()])) << (8*size_of::<__u>()-8);
                        let lo = x << n;
                        let hi = x >> (8*size_of::<__u>()-n);
                        x = lo + (hi.widening_mul(Self::INVERSE_BARRET_CONSTANT).1 + hi)
                            .wrapping_mul(__p((__inverse_polynomial & __nonzeros) << (8*size_of::<__u>()-__width)));
                    }
                }
                // adjust for alignment
                x = x >> (8*size_of::<__u>()-__width);
                x = x.reverse_bits() >> (8*size_of::<__u>()-__width);
                // update state
                self.0 = __nzu::try_from(__u::from(x)).unwrap();
                let q = q >> (8*size_of::<__u>() - bits);
            } else if #[cfg(__if(__small_table_barret))] {
                // lfsr using a per-nibble division table with Barret-reduction
                let mut x = __p::from(__u::from(self.0).reverse_bits());
                let mut q = 0;
                for i in (0..(bits+3)/4).rev() {
                    let n = min(4, bits-4*i);
                    q = (q >> n) | (__u::from(Self::INVERSE_DIV_TABLE[usize::try_from(
                        x >> (8*size_of::<__u>()-n)).unwrap()])) << (8*size_of::<__u>()-4);
                    let lo = x << n;
                    let hi = x >> (8*size_of::<__u>()-n);
                    x = lo + (hi.widening_mul(Self::INVERSE_BARRET_CONSTANT).1 + hi)
                        .wrapping_mul(__p((__inverse_polynomial & __nonzeros) << (8*size_of::<__u>()-__width)));
                }
                // adjust for alignment
                x = x >> (8*size_of::<__u>()-__width);
                x = x.reverse_bits() >> (8*size_of::<__u>()-__width);
                // update state
                self.0 = __nzu::try_from(__u::from(x)).unwrap();
                let q = q >> (8*size_of::<__u>() - bits);
            }
        }

        cfg_if! {
            if #[cfg(__if(__reflected))] {
                q.reverse_bits() >> (8*size_of::<__u>() - bits)
            } else {
                q
            }
        }
    }

    /// Skip n-bits of pseudo-random data.
    ///
    /// This takes advantage of the Galois-field representation of the LFSR to
    /// compute the new state in only `O(log log n)` multiplications.
    ///
    /// ``` rust
    /// # use ::gf256::lfsr::*;
    /// let mut lfsr = Lfsr16::new(1);
    /// lfsr.skip(48);
    /// assert_eq!(lfsr.next(16), 0xbdad);
    ///
    /// let mut lfsr = Lfsr16::new(1);
    /// assert_eq!(lfsr.next(16), 0x0001);
    /// assert_eq!(lfsr.next(16), 0x002d);
    /// assert_eq!(lfsr.next(16), 0x0451);
    /// assert_eq!(lfsr.next(16), 0xbdad);
    /// ```
    /// 
    #[inline]
    pub fn skip(&mut self, bits: __u) {
        // Each step of the lfsr is equivalent to multiplication in a finite
        // field by a primitive element g=2, which means we can use exponents of
        // g=2 to efficiently jump around states of the lfsr.
        //
        // lfsr' = 2^skip
        //
        let mul = |a: __p, b: __p| -> __p {
            cfg_if! {
                if #[cfg(__if(__naive_skip))] {
                    // naive Galois-field multiplication
                    let x = __p2::from(a) * __p2::from(b);
                    __p::try_from(x % __p2(__polynomial)).unwrap()
                } else if #[cfg(__if(__table_skip))] {
                    // Galois-field multiplication with remainder table
                    let (lo, hi) = (a << (8*size_of::<__u>()-__width))
                        .widening_mul(b);
                    let mut x = 0;
                    for b in hi.to_be_bytes() {
                        cfg_if! {
                            if #[cfg(__if(__width <= 8))] {
                                x = Self::REM_TABLE[usize::from(
                                    u8::try_from(x).unwrap() ^ b)];
                            } else {
                                x = (x << 8) ^ Self::REM_TABLE[usize::from(
                                    u8::try_from(x >> (8*size_of::<__u>()-8)).unwrap() ^ b)];
                            }
                        }
                    }
                    (__p(x) + lo) >> (8*size_of::<__u>()-__width)
                } else if #[cfg(__if(__small_table_skip))] {
                    // Galois-field multiplication with small remainder table
                    let (lo, hi) = (a << (8*size_of::<__u>()-__width))
                        .widening_mul(b);
                    let mut x = 0;
                    for b in hi.to_be_bytes() {
                        x = (x << 4) ^ Self::REM_TABLE[usize::from(
                            u8::try_from(x >> (8*size_of::<__u>()-4)).unwrap() ^ (b >> 4)) & 0xf];
                        x = (x << 4) ^ Self::REM_TABLE[usize::from(
                            u8::try_from(x >> (8*size_of::<__u>()-4)).unwrap() ^ (b >> 0)) & 0xf];
                    }
                    (__p(x) + lo) >> (8*size_of::<__u>()-__width)
                } else if #[cfg(__if(__barret_skip))] {
                    // Galois-field multiplication with Barret-reduction
                    let (lo, hi) = (a << (8*size_of::<__u>()-__width))
                        .widening_mul(b);
                    let x = lo + (hi.widening_mul(Self::BARRET_CONSTANT).1 + hi)
                        .wrapping_mul(__p((__polynomial & __nonzeros) << (8*size_of::<__u>()-__width)));
                    x >> (8*size_of::<__u>()-__width)
                }
            }
        };

        // Binary exponentiation
        let mut a = __p(2);
        let mut bits = bits;
        let mut g = __p(1);
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
        self.0 = __nzu::try_from(__u::from(mul(__p::from(__u::from(self.0)), g))).unwrap();
    }

    /// Skip n-bits of pseudo-random data backwards.
    ///
    /// This takes advantage of the Galois-field representation of the LFSR to
    /// compute the new state in only `O(log log n)` multiplications.
    ///
    /// ``` rust
    /// # use ::gf256::lfsr::*;
    /// let mut lfsr = Lfsr16::new(1);
    /// assert_eq!(lfsr.next(16), 0x0001);
    /// assert_eq!(lfsr.next(16), 0x002d);
    /// assert_eq!(lfsr.next(16), 0x0451);
    /// assert_eq!(lfsr.next(16), 0xbdad);
    /// lfsr.skip_backwards(32);
    /// assert_eq!(lfsr.next(16), 0x0451);
    /// assert_eq!(lfsr.next(16), 0xbdad);
    /// ```
    /// 
    #[inline]
    pub fn skip_backwards(&mut self, bits: __u) {
        // Assuming our lfsr is well constructed, we're in a multiplicative
        // cycle with 2^width-1 elements. Which means backwards skips are the
        // same as skipping 2^width-1-(skip % 2^width-1) elements
        //
        self.skip(__nonzeros - (bits % __nonzeros))
    }
}


// Rng implementation

impl SeedableRng for __lfsr {
    type Seed = [u8; size_of::<__u>()];

    #[inline]
    fn from_seed(seed: Self::Seed) -> Self {
        Self::new(__u::from_le_bytes(seed))
    }

    #[inline]
    fn from_rng<R: RngCore>(mut rng: R) -> Result<Self, rand::Error> {
        // find the first non-zero seed
        let mut seed = [0; size_of::<__u>()];
        loop {
            rng.try_fill_bytes(&mut seed)?;
            if __u::from_le_bytes(seed) & __nonzeros != 0 {
                break;
            }
        }

        Ok(Self::from_seed(seed))
    }
}

impl RngCore for __lfsr {
    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        // special handling for <8 bit lfsrs since these can't even
        // fill up a single byte in one go
        cfg_if! {
            if #[cfg(__if(__width < 8))] {
                for byte in dest {
                    let mut bits = 0;
                    for i in (0..8).step_by(__width) {
                        let n = min(__width, 8-i);
                        cfg_if! {
                            if #[cfg(__if(__reflected))] {
                                bits = (bits >> n) | (self.next(n) << (8-n));
                            } else {
                                bits = (bits << n) | self.next(n);
                            }
                        }
                    }
                    *byte = bits;
                }
            } else {
                // fill words at a time
                let mut chunks = dest.chunks_exact_mut(__width/8);
                for chunk in &mut chunks {
                    chunk.copy_from_slice({
                        cfg_if! {
                            if #[cfg(__if(__reflected))] {
                                &self.next(8*(__width/8)).to_le_bytes()[..__width/8]
                            } else {
                                &self.next(8*(__width/8)).to_be_bytes()[size_of::<__u>()-(__width/8)..]
                            }
                        }
                    });
                }

                let remainder = chunks.into_remainder();
                if remainder.len() > 0 {
                    remainder.copy_from_slice({
                        cfg_if! {
                            if #[cfg(__if(__reflected))] {
                                &self.next(8*remainder.len() as __u).to_le_bytes()[..remainder.len()]
                            } else {
                                &self.next(8*remainder.len() as __u).to_be_bytes()[size_of::<__u>()-remainder.len()..]
                            }
                        }
                    });
                }
            }
        }
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        Ok(self.fill_bytes(dest))
    }

    #[inline]
    fn next_u32(&mut self) -> u32 {
        cfg_if! {
            if #[cfg(__if(__width >= 32))] {
                cfg_if! {
                    if #[cfg(__if(__reflected))] {
                        self.next(32) as u32
                    } else {
                        (self.next(32) as u32).swap_bytes()
                    }
                }
            } else {
                let mut buf = [0; 4];
                self.fill_bytes(&mut buf);
                u32::from_le_bytes(buf)
            }
        }
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        cfg_if! {
            if #[cfg(__if(__width >= 64))] {
                cfg_if! {
                    if #[cfg(__if(__reflected))] {
                        self.next(64) as u64
                    } else {
                        (self.next(64) as u64).swap_bytes()
                    }
                }
            } else {
                let mut buf = [0; 8];
                self.fill_bytes(&mut buf);
                u64::from_le_bytes(buf)
            }
        }
    }
}

