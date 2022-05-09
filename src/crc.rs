//! ## CRC functions and macros
//!
//! A [cyclic redundancy check (CRC)][crc-wiki], is a common checksum algorithm
//! that is simple to implement in circuitry, and effective at detecting bit-level
//! errors.
//!
//! Looking at CRCs mathematically, they are nothing more than the remainder
//! after polynomial division by a constant, allowing for efficient implementations
//! that leverage our polynomial types and hardware-accelerated carry-less
//! multiplication.
//!
//! ``` rust
//! use gf256::crc::crc32c;
//! 
//! assert_eq!(crc32c(b"Hello World!", 0), 0xfe6cf1dc);
//! ```
//!
//! Note this module requires feature `crc`.
//!
//! A fully featured implementation of CRCs can be found in
//! [`examples/crc.rs`][crc-example]:
//!
//! ``` bash
//! $ RUSTFLAGS="-Ctarget-cpu=native" cargo run --features thread-rng,lfsr,crc,shamir,raid,rs --example crc
//!
//! testing crc("Hello World!")
//! naive_crc                => 0x1c291ca3
//! less_naive_crc           => 0x1c291ca3
//! word_less_naive_crc      => 0x1c291ca3
//! table_crc                => 0x1c291ca3
//! small_table_crc          => 0x1c291ca3
//! barret_crc               => 0x1c291ca3
//! word_barret_crc          => 0x1c291ca3
//! reversed_barret_crc      => 0x1c291ca3
//! word_reversed_barret_crc => 0x1c291ca3
//! ```
//!
//! ## How do CRCs work?
//! 
//! CRCs mathematically are fascinating as they are nothing more than the binary
//! remainder after polynomial division by some constant.
//!
//! Take some data for example:
//!
//! ``` text
//! input = "hi"
//!       = 0110100001101001
//! ```
//!
//! Choose a size for our CRC, say, 8-bits. Pad our data by that number of
//! zeros, and divide by some polynomial constant with that number of
//! bits + 1, in this case I'm using `0b100000111`:
//!
//! ``` text
//! input = "hi"
//!       = 01101000 01101001
//!
//! pad with 8 zeros:
//!
//!       = 01101000 01101001 00000000
//!
//! divide by 0b100000111:
//!
//!       = 01101000 01101001 00000000
//!       ^  1000001 11
//!       = 00101001 10101001 00000000
//!       ^   100000 111
//!       = 00001001 01001001 00000000
//!       ^     1000 00111
//!       = 00000001 01110001 00000000
//!       ^        1 00000111
//!       = 00000000 01110110 00000000
//!       ^           1000001 11
//!       = 00000000 00110111 11000000
//!       ^            100000 111
//!       = 00000000 00010111 00100000
//!       ^             10000 0111
//!       = 00000000 00000111 01010000
//!       ^               100 000111
//!       = 00000000 00000011 01001100
//!       ^                10 0000111
//!       = 00000000 00000001 01000010
//!       ^                 1 00000111
//!       ---------------------------
//!       = 00000000 00000000 01000101
//!                              |
//! remainder = 01000101 <-------'
//! ```
//!
//! And this is our CRC!
//!
//! Note we are performing polynomial division! So instead of shifts and
//! subtractions, we are doing shifts and xors. See the [p](../p) module's
//! documentation for more info on viewing data as binary polynomials.
//!
//! There's some interesting things to note:
//!
//! 1. The original message bits will always end up as zero if the size of our 
//!    polynomial matches the number of appended zeros + 1
//!
//! 2. The message bits seem to have a rather large impact on every bit in the
//!    resulting CRC, a property of a good checksum.
//!
//! We can of course compute this directly with our polynomial types:
//!
//! ``` rust
//! # use ::gf256::*;
//! #
//! let data = p32(0b0110100001101001) << 8;
//! let polynomial = p32(0b100000111);
//! assert_eq!(data % polynomial, p32(0b01000101));
//! ```
//!
//! One fun feature of CRCs is that, since the CRC is the remainder after division,
//! if we replace our padding zeros with the CRC (note this is the same as appending
//! the CRC to our original message), computing the CRC again will give us a value of
//! zero:
//!
//! ``` rust
//! # use ::gf256::*;
//! #
//! let data_with_crc = p32(0b011010000110100101000101);
//! let polynomial = p32(0b100000111);
//! assert_eq!(data_with_crc % polynomial, p32(0));
//! ```
//!
//! This is because we've effectively calculated:
//!
//! ``` text
//! m' = m - (m % p)
//! ```
//!
//! Where `m` is our original message with padded zeros and `p` is our polynomial.
//!
//! Since the [remainder][remainder] is defined as:
//!
//! ``` text
//! a % b = a - b*floor(a/b)
//! ```
//!
//! We can substitute `m` and `p` in:
//!
//! ``` text
//! m' = (m - (m % p))
//!
//! m' = m - (m - p*floor(m/p))
//!
//! m' = p*floor(m/p)
//! ```
//!
//! Which shows that `m'` is now some integer-polynomial multiple of `p`. And since
//! the remainder of a multiple is always zero, the result of a second remainder,
//! our second CRC, should also be zero.
//!
//! ``` text
//! m' % p = p*floor(m/p) % p
//!
//! m' % p = p*floor(m/p) - (p*floor(p*floor(m/p)/p))
//!
//! m' % p = p*floor(m/p) - (p*floor(floor(m/p)))
//!
//! m' % p = p*floor(m/p) - p*floor(m/p)
//!
//! m' % p = 0
//! ```
//!
//! This trick can sometimes be useful for simplifying the CRC validation.
//!
//! And we can create this exact CRC using gf256:
//!
//! ``` rust
//! # pub use ::gf256::*;
//! use ::gf256::crc;
//!
//! #[crc::crc(polynomial=0b100000111, reflected=false, xor=0)]
//! fn crc8() {}
//!
//! # fn main() {
//! assert_eq!(crc8(&[0b01101000, 0b01101001], 0), 0b01000101);
//! # }
//! ```
//!
//! The `reflected` and `xor` options are extra tweaks to the CRC algorithm that are
//! commonly found in standard CRCs. More info on these in the [crc macro](attr.crc)
//! documentation.
//!
//! ## Optimizations
//!
//! CRCs are simple and fast in circuitry, but not so much in software due to relying
//! on bit-level operations. Fortunately there are several optimizations we can do
//! to speed up CRC calculation in software.
//!
//! A straightforward implementation of polynomial remainder is available in `naive`
//! mode, though it has been tweaked to work well with byte-level slices. Additional modes:
//!
//! - In `table`, CRCs use a precomputed remainder table to compute the remainder a byte
//!   at a time.
//!
//!   The trick for computing the remainder table is to use the xor of the argument with
//!   the current remainder as the index into the table. This allows you to compute the
//!   remainder of any arbitrarily sized inputs using only a single byte index lookup
//!   at a time.
//!   
//!   This is the most common implementation of CRCs you will find, due to its speed and
//!   portability.
//!
//! - In `small_table` mode, the same strategy as `table` mode is used, but with a 16
//!   element  remainder table computer the remainder a nibble at a time.
//!
//! - In `barret` mode, CRCs use [Barret-reduction][barret-reduction] to efficiently
//!   compute the remainder using only multiplication by precomputed constants.
//!
//!   This mode is especially effective when hardware carry-less multiplication
//!   instructions are available.
//!
//! If hardware carry-less multiplication is available, `barret` mode is the fastest
//! option for CRCs, so CRC implementations will use `barret` by default.
//!
//! If hardware carry-less multiplication is not available, `table` mode will be
//! used, unless the feature `small-tables` is enabled, in which case `small_table`
//! mode will be used. If the feature `no-tables` is enabled, `barret` mode will be
//! used as it outperforms a naive implementation even when hardware carry-less
//! multiplication is not available.
//!   
//! Though note the default mode is susceptible to change.
//!
//! ## Choosing a polynomial
//!
//! Choosing a good CRC polynomial is rather complicated. It depends on the length
//! of the data you are protecting and the type of errors you are protecting
//! against.
//!
//! The best resource I've found for choosing good CRC polynomials is the research
//! done in Philip Koopman's [Cyclic Redundancy Code (CRC) Polynomial Selection
//! For Embedded Networks][koopman].
//!
//! Generally you want to choose a CRC that has the largest "[Hamming distance
//! ][hamming-distance]" for your message length, or at least a good Hamming
//! distance over your range of message lengths. Hamming distance is the number
//! of bit-flips required to get to another message with a valid CRC, so a larger
//! Hamming distance means more bit errors before you fail to detect that something
//! is wrong.
//!
//! Philip Koopman also has a list of good CRC polynomials and their effective
//! Hamming distances at various message lengths [here][crc-polynomials].
//!
//! Note you may see several different formats for CRC polynomials! Where the
//! mathematically correct polynomial may be `0x104c11db7`, you may see a truncated
//! `0x04c11db7` or `0x82608edb` representation to fit into 32-bits, or a
//! bit-reflected `0x1db710641`, `0xedb88320`, or `0x1db71064` representation
//! (what a mess!).  Make sure you understand the correct bit-width and endianness
//! of a given polynomial before using it.
//!
//! ## A note on CRC32 vs CRC32C
//!
//! Did I mention choosing a good CRC polynomial is rather complicated? What if
//! I told you that the most popular 32-bit CRC polynomial since 1975 was actually
//! a sub-optimal polynomial for general-purpose 32-bits CRCs?
//!
//! Well that is the situation the world finds itself in today. The most popular
//! 32-bit CRC polynomial `0x104c11db7`, called just CRC32 (though sometimes referred
//! to as CRC32B, perhaps only because it predates CRC32C?), performs poorly the
//! moment there is more than 2 bit errors.
//!
//! The more recent polynomial `0x11edc6f41`, called CRC32C, provides much better
//! error detection for a wider range of bit errors without any change to the
//! underlying algorithm.
//!
//! But CRC32 is still in heavy use today, so gf256 provides both
//! [`crc32`](crate::crc::crc32) and [`crc32c`](crate::crc::crc32c).
//! It's suggested to use [`crc32c`](crate::crc::crc32) for new applications.
//!
//!
//! [crc-wiki]: https://en.wikipedia.org/wiki/Cyclic_redundancy_check
//! [remainder]: https://en.wikipedia.org/wiki/Modulo_operation
//! [barret-reduction]: https://en.wikipedia.org/wiki/Barrett_reduction
//! [hamming-distance]: https://en.wikipedia.org/wiki/Hamming_distance
//! [koopman]: http://users.ece.cmu.edu/~koopman/roses/dsn04/koopman04_crc_poly_embedded.pdf
//! [crc-polynomials]: https://users.ece.cmu.edu/~koopman/crc
//! [crc-example]: https://github.com/geky/gf256/blob/master/examples/crc.rs


/// A macro for generating custom CRC functions.
///
/// ``` rust,ignore
/// # use ::gf256::*;
/// # use ::gf256::crc::crc;
/// #[crc(polynomial=0x11edc6f41)]
/// pub fn my_crc32() {}
///
/// # fn main() {
/// assert_eq!(my_crc32(b"Hello World!", 0), 0xfe6cf1dc);
/// # }
/// ```
///
/// The `crc` macro accepts a number of configuration options:
///
/// - `polynomial` - The irreducible polynomial that defines the CRC.
/// - `u` - The underlying unsigned type, defaults to the minimum sized
///   unsigned type that fits the CRC state space.
/// - `u2` - An unsigned type with twice the width, used as an intermediary type
///   for computations, defaults to the correct type based on `u`.
/// - `p` - The polynomial type used for computation, defaults to the
///   polynomial version of `u`.
/// - `p2` - A polynomial type with twice the width, used as an intermediary type
///   for computations, defaults to the correct type based on `p`.
/// - `reflected` - Indicate if the CRC should have its bits reversed,
///   defaults to true.
/// - `xor` - A bit-mask to xor the input and output CRC with, defaults to
///   all ones.
/// - `naive` - Use a naive bitwise implementation.
/// - `table` - Use precomputed CRC table. This is the default if hardware
///   polynomial multiplication is not available.
/// - `small_table` - Use a small, 16-element CRC table.
/// - `barret` - Use Barret-reduction with polynomial multiplication. This is
///   the default if hardware polynomial multiplication is available.
///
/// ``` rust,ignore
/// # use ::gf256::*;
/// # use ::gf256::crc::crc;
/// #[crc(
///     polynomial=0x11edc6f41,
///     u=u32,
///     u2=u64,
///     p=p32,
///     p2=p64,
///     reflected=true,
///     xor=0xffffffff,
///     // naive,
///     // table,
///     // small_table,
///     // barret,
/// )]
/// pub fn my_crc32() {}
///
/// # fn main() {
/// assert_eq!(my_crc32(b"Hello World!", 0), 0xfe6cf1dc);
/// # }
/// ```
///

pub use gf256_macros::crc;


// CRC functions
//
// Hamming distance (HD) info from here:
// http://users.ece.cmu.edu/~koopman/crc/index.html

// HD=3,4, up to 119+8 bits
#[crc(polynomial=0x107)]
pub fn crc8() {}

// HD=3,4, up to 32751+16 bits
#[crc(polynomial=0x11021)]
pub fn crc16() {}

// HD=3, up to 4294967263+32 bits
// HD=4, up to 91607+32 bits
// HD=5, up to 2974+32 bits
// HD=6, up to 268+32 bits
// HD=7, up to 171+32 bits
// HD=8, up to 91+32 bits
#[crc(polynomial=0x104c11db7)]
pub fn crc32() {}

// HD=3,4, up to 2147483615+32 bits
// HD=5,6, up to 5243+32 bits
// HD=7,8, up to 177+32 bits
#[crc(polynomial=0x11edc6f41)]
pub fn crc32c() {}

// HD=3,4, up to 8589606850+64 bits
// HD=5,6, up to 126701+64 bits
// HD=7,7, up to ~33710+64 bits
#[crc(polynomial=0x142f0e1eba9ea3693)]
pub fn crc64() {}


#[cfg(test)]
mod test {
    use super::*;
    use crate::p::*;

    #[test]
    fn crc() {
        assert_eq!(crc8(b"Hello World!", 0),   0xb3);
        assert_eq!(crc16(b"Hello World!", 0),  0x0bbb);
        assert_eq!(crc32(b"Hello World!", 0),  0x1c291ca3);
        assert_eq!(crc32c(b"Hello World!", 0), 0xfe6cf1dc);
        assert_eq!(crc64(b"Hello World!", 0),  0x75045245c9ea6fe2);
    }

    // explicit modes
    #[crc(polynomial=0x107, naive)] fn crc8_naive() {}
    #[crc(polynomial=0x11021, naive)] fn crc16_naive() {}
    #[crc(polynomial=0x104c11db7, naive)] fn crc32_naive() {}
    #[crc(polynomial=0x11edc6f41, naive)] fn crc32c_naive() {}
    #[crc(polynomial=0x142f0e1eba9ea3693, naive)] fn crc64_naive() {}

    #[crc(polynomial=0x107, table)] fn crc8_table() {}
    #[crc(polynomial=0x11021, table)] fn crc16_table() {}
    #[crc(polynomial=0x104c11db7, table)] fn crc32_table() {}
    #[crc(polynomial=0x11edc6f41, table)] fn crc32c_table() {}
    #[crc(polynomial=0x142f0e1eba9ea3693, table)] fn crc64_table() {}

    #[crc(polynomial=0x107, small_table)] fn crc8_small_table() {}
    #[crc(polynomial=0x11021, small_table)] fn crc16_small_table() {}
    #[crc(polynomial=0x104c11db7, small_table)] fn crc32_small_table() {}
    #[crc(polynomial=0x11edc6f41, small_table)] fn crc32c_small_table() {}
    #[crc(polynomial=0x142f0e1eba9ea3693, small_table)] fn crc64_small_table() {}

    #[crc(polynomial=0x107, barret)] fn crc8_barret() {}
    #[crc(polynomial=0x11021, barret)] fn crc16_barret() {}
    #[crc(polynomial=0x104c11db7, barret)] fn crc32_barret() {}
    #[crc(polynomial=0x11edc6f41, barret)] fn crc32c_barret() {}
    #[crc(polynomial=0x142f0e1eba9ea3693, barret)] fn crc64_barret() {}

    #[test]
    fn crc_naive() {
        assert_eq!(crc8_naive(b"Hello World!", 0),   0xb3);
        assert_eq!(crc16_naive(b"Hello World!", 0),  0x0bbb);
        assert_eq!(crc32_naive(b"Hello World!", 0),  0x1c291ca3);
        assert_eq!(crc32c_naive(b"Hello World!", 0), 0xfe6cf1dc);
        assert_eq!(crc64_naive(b"Hello World!", 0),  0x75045245c9ea6fe2);
    }

    #[test]
    fn crc_table() {
        assert_eq!(crc8_table(b"Hello World!", 0),   0xb3);
        assert_eq!(crc16_table(b"Hello World!", 0),  0x0bbb);
        assert_eq!(crc32_table(b"Hello World!", 0),  0x1c291ca3);
        assert_eq!(crc32c_table(b"Hello World!", 0), 0xfe6cf1dc);
        assert_eq!(crc64_table(b"Hello World!", 0),  0x75045245c9ea6fe2);
    }

    #[test]
    fn crc_small_table() {
        assert_eq!(crc8_small_table(b"Hello World!", 0),   0xb3);
        assert_eq!(crc16_small_table(b"Hello World!", 0),  0x0bbb);
        assert_eq!(crc32_small_table(b"Hello World!", 0),  0x1c291ca3);
        assert_eq!(crc32c_small_table(b"Hello World!", 0), 0xfe6cf1dc);
        assert_eq!(crc64_small_table(b"Hello World!", 0),  0x75045245c9ea6fe2);
    }

    #[test]
    fn crc_barret() {
        assert_eq!(crc8_barret(b"Hello World!", 0),   0xb3);
        assert_eq!(crc16_barret(b"Hello World!", 0),  0x0bbb);
        assert_eq!(crc32_barret(b"Hello World!", 0),  0x1c291ca3);
        assert_eq!(crc32c_barret(b"Hello World!", 0), 0xfe6cf1dc);
        assert_eq!(crc64_barret(b"Hello World!", 0),  0x75045245c9ea6fe2);
    }

    #[test]
    fn crc_unaligned() {
        assert_eq!(crc8_naive(b"Hello World!!", 0),   0x2f);
        assert_eq!(crc16_naive(b"Hello World!!", 0),  0xcba0);
        assert_eq!(crc32_naive(b"Hello World!!", 0),  0xd1a8249d);
        assert_eq!(crc32c_naive(b"Hello World!!", 0), 0x1ec51c06);
        assert_eq!(crc64_naive(b"Hello World!!", 0),  0xf5a8a397b60da2e1);

        assert_eq!(crc8_table(b"Hello World!!", 0),   0x2f);
        assert_eq!(crc16_table(b"Hello World!!", 0),  0xcba0);
        assert_eq!(crc32_table(b"Hello World!!", 0),  0xd1a8249d);
        assert_eq!(crc32c_table(b"Hello World!!", 0), 0x1ec51c06);
        assert_eq!(crc64_table(b"Hello World!!", 0),  0xf5a8a397b60da2e1);

        assert_eq!(crc8_small_table(b"Hello World!!", 0),   0x2f);
        assert_eq!(crc16_small_table(b"Hello World!!", 0),  0xcba0);
        assert_eq!(crc32_small_table(b"Hello World!!", 0),  0xd1a8249d);
        assert_eq!(crc32c_small_table(b"Hello World!!", 0), 0x1ec51c06);
        assert_eq!(crc64_small_table(b"Hello World!!", 0),  0xf5a8a397b60da2e1);

        assert_eq!(crc8_barret(b"Hello World!!", 0),   0x2f);
        assert_eq!(crc16_barret(b"Hello World!!", 0),  0xcba0);
        assert_eq!(crc32_barret(b"Hello World!!", 0),  0xd1a8249d);
        assert_eq!(crc32c_barret(b"Hello World!!", 0), 0x1ec51c06);
        assert_eq!(crc64_barret(b"Hello World!!", 0),  0xf5a8a397b60da2e1);
    }

    #[test]
    fn crc_partial() {
        assert_eq!(crc8_naive(b"World!", crc8_naive(b"Hello ", 0)),     0xb3);
        assert_eq!(crc16_naive(b"World!", crc16_naive(b"Hello ", 0)),   0x0bbb);
        assert_eq!(crc32_naive(b"World!", crc32_naive(b"Hello ", 0)),   0x1c291ca3);
        assert_eq!(crc32c_naive(b"World!", crc32c_naive(b"Hello ", 0)), 0xfe6cf1dc);
        assert_eq!(crc64_naive(b"World!", crc64_naive(b"Hello ", 0)),   0x75045245c9ea6fe2);

        assert_eq!(crc8_table(b"World!", crc8_table(b"Hello ", 0)),     0xb3);
        assert_eq!(crc16_table(b"World!", crc16_table(b"Hello ", 0)),   0x0bbb);
        assert_eq!(crc32_table(b"World!", crc32_table(b"Hello ", 0)),   0x1c291ca3);
        assert_eq!(crc32c_table(b"World!", crc32c_table(b"Hello ", 0)), 0xfe6cf1dc);
        assert_eq!(crc64_table(b"World!", crc64_table(b"Hello ", 0)),   0x75045245c9ea6fe2);

        assert_eq!(crc8_small_table(b"World!", crc8_small_table(b"Hello ", 0)),     0xb3);
        assert_eq!(crc16_small_table(b"World!", crc16_small_table(b"Hello ", 0)),   0x0bbb);
        assert_eq!(crc32_small_table(b"World!", crc32_small_table(b"Hello ", 0)),   0x1c291ca3);
        assert_eq!(crc32c_small_table(b"World!", crc32c_small_table(b"Hello ", 0)), 0xfe6cf1dc);
        assert_eq!(crc64_small_table(b"World!", crc64_small_table(b"Hello ", 0)),   0x75045245c9ea6fe2);

        assert_eq!(crc8_barret(b"World!", crc8_barret(b"Hello ", 0)),     0xb3);
        assert_eq!(crc16_barret(b"World!", crc16_barret(b"Hello ", 0)),   0x0bbb);
        assert_eq!(crc32_barret(b"World!", crc32_barret(b"Hello ", 0)),   0x1c291ca3);
        assert_eq!(crc32c_barret(b"World!", crc32c_barret(b"Hello ", 0)), 0xfe6cf1dc);
        assert_eq!(crc64_barret(b"World!", crc64_barret(b"Hello ", 0)),   0x75045245c9ea6fe2);
    }

    // odd-sized crcs
    #[crc(polynomial=0x13, naive)] fn crc4_naive() {}
    #[crc(polynomial=0x13, table)] fn crc4_table() {}
    #[crc(polynomial=0x13, small_table)] fn crc4_small_table() {}
    #[crc(polynomial=0x13, barret)] fn crc4_barret() {}

    #[crc(polynomial=0x11e7, naive)] fn crc12_naive() {}
    #[crc(polynomial=0x11e7, table)] fn crc12_table() {}
    #[crc(polynomial=0x11e7, small_table)] fn crc12_small_table() {}
    #[crc(polynomial=0x11e7, barret)] fn crc12_barret() {}

    #[crc(polynomial=0x8002a9, naive)] fn crc23_naive() {}
    #[crc(polynomial=0x8002a9, table)] fn crc23_table() {}
    #[crc(polynomial=0x8002a9, small_table)] fn crc23_small_table() {}
    #[crc(polynomial=0x8002a9, barret)] fn crc23_barret() {}

    #[test]
    fn crc_odd_sizes() {
        assert_eq!(crc4_naive(b"Hello World!", 0),       0x7);
        assert_eq!(crc4_table(b"Hello World!", 0),       0x7);
        assert_eq!(crc4_small_table(b"Hello World!", 0), 0x7);
        assert_eq!(crc4_barret(b"Hello World!", 0),      0x7);

        assert_eq!(crc12_naive(b"Hello World!", 0),       0x1d4);
        assert_eq!(crc12_table(b"Hello World!", 0),       0x1d4);
        assert_eq!(crc12_small_table(b"Hello World!", 0), 0x1d4);
        assert_eq!(crc12_barret(b"Hello World!", 0),      0x1d4);

        assert_eq!(crc23_naive(b"Hello World!", 0),       0x32da1c);
        assert_eq!(crc23_table(b"Hello World!", 0),       0x32da1c);
        assert_eq!(crc23_small_table(b"Hello World!", 0), 0x32da1c);
        assert_eq!(crc23_barret(b"Hello World!", 0),      0x32da1c);

        assert_eq!(crc4_naive(b"Hello World!!", 0),       0x1);
        assert_eq!(crc4_table(b"Hello World!!", 0),       0x1);
        assert_eq!(crc4_small_table(b"Hello World!!", 0), 0x1);
        assert_eq!(crc4_barret(b"Hello World!!", 0),      0x1);

        assert_eq!(crc12_naive(b"Hello World!!", 0),       0xb8d);
        assert_eq!(crc12_table(b"Hello World!!", 0),       0xb8d);
        assert_eq!(crc12_small_table(b"Hello World!!", 0), 0xb8d);
        assert_eq!(crc12_barret(b"Hello World!!", 0),      0xb8d);

        assert_eq!(crc23_naive(b"Hello World!!", 0),       0x11685a);
        assert_eq!(crc23_table(b"Hello World!!", 0),       0x11685a);
        assert_eq!(crc23_small_table(b"Hello World!!", 0), 0x11685a);
        assert_eq!(crc23_barret(b"Hello World!!", 0),      0x11685a);
    }

    // bit reflected 
    #[crc(polynomial=0x104c11db7, naive, reflected=false)] fn crc32_naive_unreflected() {}
    #[crc(polynomial=0x104c11db7, table, reflected=false)] fn crc32_table_unreflected() {}
    #[crc(polynomial=0x104c11db7, small_table, reflected=false)] fn crc32_small_table_unreflected() {}
    #[crc(polynomial=0x104c11db7, barret, reflected=false)] fn crc32_barret_unreflected() {}

    #[test]
    fn crc_unreflected() {
        assert_eq!(crc32_naive_unreflected(b"Hello World!", 0),       0x6b1a7cae);
        assert_eq!(crc32_table_unreflected(b"Hello World!", 0),       0x6b1a7cae);
        assert_eq!(crc32_small_table_unreflected(b"Hello World!", 0), 0x6b1a7cae);
        assert_eq!(crc32_barret_unreflected(b"Hello World!", 0),      0x6b1a7cae);
    }

    // bit inverted 
    #[crc(polynomial=0x104c11db7, naive, xor=0)] fn crc32_naive_uninverted() {}
    #[crc(polynomial=0x104c11db7, table, xor=0)] fn crc32_table_uninverted() {}
    #[crc(polynomial=0x104c11db7, small_table, xor=0)] fn crc32_small_table_uninverted() {}
    #[crc(polynomial=0x104c11db7, barret, xor=0)] fn crc32_barret_uninverted() {}

    #[test]
    fn crc_uninverted() {
        assert_eq!(crc32_naive_uninverted(b"Hello World!", 0),       0x67fcdacc);
        assert_eq!(crc32_table_uninverted(b"Hello World!", 0),       0x67fcdacc);
        assert_eq!(crc32_small_table_uninverted(b"Hello World!", 0), 0x67fcdacc);
        assert_eq!(crc32_barret_uninverted(b"Hello World!", 0),      0x67fcdacc);
    }

    // all CRC params
    #[crc(
        polynomial=0x104c11db7,
        u=u32,
        u2=u64,
        p=p32,
        p2=p64,
        reflected=true,
        xor=0xffffffff,
    )]
    fn crc32_all_params() {}

    #[test]
    fn crc_all_params() {
        assert_eq!(crc32_all_params(b"Hello World!", 0), 0x1c291ca3);
    }
}
