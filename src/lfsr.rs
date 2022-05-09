//! ## LFSR structs and macros
//!
//! A [linear-feedback shift register (LFSR)][lfsr-wiki] is a simple method of
//! creating a pseudo-random stream of bits using only a small circuit of shifts
//! and xors.
//!
//! LFSRs can be modelled mathematically as multiplication in a Galois-field,
//! allowing efficient bit generation, both forward and backwards, and efficient
//! seeking to any state of the LFSR.
//!
//! ``` rust
//! # use std::iter;
//! use gf256::lfsr::Lfsr16;
//!
//! let mut lfsr = Lfsr16::new(1);
//! assert_eq!(lfsr.next(16), 0x0001);
//! assert_eq!(lfsr.next(16), 0x002d);
//! assert_eq!(lfsr.next(16), 0x0451);
//! assert_eq!(lfsr.next(16), 0xbdad);
//! assert_eq!(lfsr.prev(16), 0xbdad);
//! assert_eq!(lfsr.prev(16), 0x0451);
//! assert_eq!(lfsr.prev(16), 0x002d);
//! assert_eq!(lfsr.prev(16), 0x0001);
//! ```
//!
//! Note this module requires feature `lfsr`.
//!
//! A fully featured implementation of LFSRs can be found in
//! [`examples/lfsr.rs`][lfsr-example]:
//!
//! ``` bash
//! $ RUSTFLAGS="-Ctarget-cpu=native" cargo run --features thread-rng,lfsr,crc,shamir,raid,rs --example lfsr
//!
//! testing lfsr64
//! lfsr64_naive                        => 0000000000000001000000000000001b00000000000001450000000000001db7
//! lfsr64_divrem                       => 0000000000000001000000000000001b00000000000001450000000000001db7
//! lfsr64_table                        => 0000000000000001000000000000001b00000000000001450000000000001db7
//! lfsr64_small_table                  => 0000000000000001000000000000001b00000000000001450000000000001db7
//! lfsr64_barret                       => 0000000000000001000000000000001b00000000000001450000000000001db7
//! lfsr64_table_barret                 => 0000000000000001000000000000001b00000000000001450000000000001db7
//! lfsr64_small_table_barret           => 0000000000000001000000000000001b00000000000001450000000000001db7
//! rev(lfsr64_naive)                   => b71d00000000000045010000000000001b000000000000000100000000000000
//! rev(lfsr64_divrem)                  => b71d00000000000045010000000000001b000000000000000100000000000000
//! rev(lfsr64_table)                   => b71d00000000000045010000000000001b000000000000000100000000000000
//! rev(lfsr64_small_table)             => b71d00000000000045010000000000001b000000000000000100000000000000
//! rev(lfsr64_barret)                  => b71d00000000000045010000000000001b000000000000000100000000000000
//! rev(lfsr64_table_barret)            => b71d00000000000045010000000000001b000000000000000100000000000000
//! rev(lfsr64_small_table_barret)      => b71d00000000000045010000000000001b000000000000000100000000000000
//! lfsr64_naive+9000                   => 4aec56b077706daa269325545515010d361e101b06c71bad8b33b14551004b42
//! lfsr64_divrem+9000                  => 4aec56b077706daa269325545515010d361e101b06c71bad8b33b14551004b42
//! lfsr64_table+9000                   => 4aec56b077706daa269325545515010d361e101b06c71bad8b33b14551004b42
//! lfsr64_small_table+9000             => 4aec56b077706daa269325545515010d361e101b06c71bad8b33b14551004b42
//! lfsr64_barret+9000                  => 4aec56b077706daa269325545515010d361e101b06c71bad8b33b14551004b42
//! lfsr64_table_barret+9000            => 4aec56b077706daa269325545515010d361e101b06c71bad8b33b14551004b42
//! lfsr64_small_table_barret+9000      => 4aec56b077706daa269325545515010d361e101b06c71bad8b33b14551004b42
//! rev(lfsr64_naive+9000)              => 424b005145b1338bad1bc7061b101e360d01155554259326aa6d7077b056ec4a
//! rev(lfsr64_divrem+9000)             => 424b005145b1338bad1bc7061b101e360d01155554259326aa6d7077b056ec4a
//! rev(lfsr64_table+9000)              => 424b005145b1338bad1bc7061b101e360d01155554259326aa6d7077b056ec4a
//! rev(lfsr64_small_table+9000)        => 424b005145b1338bad1bc7061b101e360d01155554259326aa6d7077b056ec4a
//! rev(lfsr64_barret+9000)             => 424b005145b1338bad1bc7061b101e360d01155554259326aa6d7077b056ec4a
//! rev(lfsr64_table_barret+9000)       => 424b005145b1338bad1bc7061b101e360d01155554259326aa6d7077b056ec4a
//! rev(lfsr64_small_table_barret+9000) => 424b005145b1338bad1bc7061b101e360d01155554259326aa6d7077b056ec4a
//! lfsr64_naive+9000-9000              => 0000000000000001000000000000001b00000000000001450000000000001db7
//! lfsr64_divrem+9000-9000             => 0000000000000001000000000000001b00000000000001450000000000001db7
//! lfsr64_table+9000-9000              => 0000000000000001000000000000001b00000000000001450000000000001db7
//! lfsr64_small_table+9000-9000        => 0000000000000001000000000000001b00000000000001450000000000001db7
//! lfsr64_barret+9000-9000             => 0000000000000001000000000000001b00000000000001450000000000001db7
//! lfsr64_table_barret+9000-9000       => 0000000000000001000000000000001b00000000000001450000000000001db7
//! lfsr64_small_table_barret+9000-9000 => 0000000000000001000000000000001b00000000000001450000000000001db7
//!
//! 2048 lfsr64 samples:
//!                                           .      .                       .
//!                  .     '  .    .  .'       .                             :
//!              .  '               '   '' '    .      '  .'  '             ':
//!           . .        '  . . .:  ::  . .' .. . '                .        .:
//!        '   ' '    .:  ' ' . .: . .  ::  .:. .    . ' . ' .      .      .::
//!             .'  .: ' ..   :.   ::.'.:. ' ' . '.'.  ' '       '         '::
//!           '    .  ' .''..:..'.: .:.  '  '.    :'':.        '           .::
//!          :   ''.. ''. '. .:':''''.:':..''...  :.  :  '.   :. '        ::::
//!             .  . .:  .' ' :':.:'. :.''::' :::'.:... :    .'           ::::
//!               ': ':''  '': :.: .:' .::::.:': ..' :.''   .' . :        ::::
//!         .  .  .    ':.' :'   ''.:. ::.'''' '. : '.  ..     .   '       :::
//!        '    . ' '.. .'  '..'.' .:'' . '. ''. ':  ' .. '     '   '       ::
//!            '  ' .    '      ...''.: .:. .' :    ..' . '   :             ::
//!                 '.         .     '  '' .      '    .  ': .'.             :
//!                    '    .. .  '.'   .             '                      :
//!     '         '    '.'.'  .    ''   :         .  .              '       ':
//!
//!                                 :   :.     .
//!     .  ...........:::...::::::.:::::::::.:::..:..::........... ..
//!
//! 64513/131072 ones (49.22%), 0.27% compressability
//! ```
//!
//! ## How do LFSRs work?
//!
//! Consider the following LFSR, where each bit represents a single-bit register,
//! and each `x` represents an xor gate, sometimes called the "taps" of an LFSR.
//!
//! We can step through some states by hand to see what the output would be:
//!
//! ``` text
//! output <-.- 1 <-- 1 <-- 0 <-- 0 <x- 1 <x- 1 <x- 0 <-- 0 <-.
//!          '-----------------------'-----'-----'------------'
//! output      internal state
//!      1      1     0     0     0     0     1     0     1
//!      1      0     0     0     1     0     1     1     1
//!      0      0     0     1     0     1     1     1     0
//!      0      0     1     0     1     1     1     0     0
//!      0      1     0     1     1     1     0     0     0
//!      1      0     1     1     0     1     1     0     1
//!      0      1     1     0     1     1     0     1     0
//!      1      1     0     1     0     1     0     0     1
//!      ...
//! ```
//!
//! Consider what this might look like in code. We can model the bit of feedback
//! as a branch on whether or not to xor our internal state with an integer
//! containing a 1 for each LFSR tap:
//!
//! ``` rust
//! let mut state: u32 = 0b11001100;
//! let mut step = || {
//!     state = state << 1;
//!     let output = (state & 0x100) >> 8;
//!     if output != 0 {
//!         state ^= 0x1d
//!     }
//!     state = 0xff & state;
//!     output
//! };
//! assert_eq!(step(), 1);
//! assert_eq!(step(), 1);
//! assert_eq!(step(), 0);
//! assert_eq!(step(), 0);
//! assert_eq!(step(), 0);
//! assert_eq!(step(), 1);
//! assert_eq!(step(), 0);
//! assert_eq!(step(), 1);
//! ```
//!
//! Normally, we think of the high bit just "falling off" when we shift the LFSR
//! state. But we could instead model it as an xor with itself, making the high
//! bit a part of our taps and removing the bit mask:
//!
//! ``` rust
//! let mut state: u32 = 0b11001100;
//! let mut step = || {
//!     state = state << 1;
//!     let output = (state & 0x100) >> 8;
//!     if output != 0 {
//!         state ^= 0x11d
//!     }
//!     output
//! };
//! assert_eq!(step(), 1);
//! assert_eq!(step(), 1);
//! assert_eq!(step(), 0);
//! assert_eq!(step(), 0);
//! assert_eq!(step(), 0);
//! assert_eq!(step(), 1);
//! assert_eq!(step(), 0);
//! assert_eq!(step(), 1);
//! ```
//!
//! This is actually polynomial division of a 9-bit number! Remember that
//! polynomial division is repeated shifts and xors until the dividend is
//! smaller than our divisor. In this case, we only need a single step.
//!
//! And if you remember your computer-science tricks, shifting by one is
//! equivalent to multiplying by 2 (this is still true with polynomial
//! multiplication because no carry could have occured).
//!
//! So instead of using bit-shifts and xors, we could use multiplication and
//! division with our polynomial types:
//!
//! ``` rust
//! # use ::gf256::*;
//! #
//! let mut state: p32 = p32(0b11001100);
//! let mut step = || {
//!     state = state * p32(2);
//!     let output = state / p32(0x11d);
//!     state = state % p32(0x11d);
//!     u32::from(output)
//! };
//! assert_eq!(step(), 1);
//! assert_eq!(step(), 1);
//! assert_eq!(step(), 0);
//! assert_eq!(step(), 0);
//! assert_eq!(step(), 0);
//! assert_eq!(step(), 1);
//! assert_eq!(step(), 0);
//! assert_eq!(step(), 1);
//! ```
//!
//! And a polynomial multiplication followed by a remainder with a constant...
//! Isn't that equivalent to Galois-field multiplication?
//!
//! Why yes it is!
//!
//! ``` rust
//! # pub use ::gf256::*;
//! use ::gf256::gf::gf;
//!
//! #[gf(polynomial=0x11d, generator=2)]
//! type gf256_lfsr;
//!
//! # fn main() {
//! let mut state: gf256_lfsr = gf256_lfsr(0b11001100);
//! let mut step = || {
//!     let output = (state & 0x80) >> 7;
//!     state = state * gf256_lfsr(2);
//!     u32::from(output)
//! };
//! assert_eq!(step(), 1);
//! assert_eq!(step(), 1);
//! assert_eq!(step(), 0);
//! assert_eq!(step(), 0);
//! assert_eq!(step(), 0);
//! assert_eq!(step(), 1);
//! assert_eq!(step(), 0);
//! assert_eq!(step(), 1);
//! # }
//! ```
//!
//! Up until this point we were just using a seemingly arbitrary sequence of taps
//! to define our LFSR. But since LFSRs are equivalent to Galois-fields, we can
//! use our knowledge of Galois-fields to come up with a good sequence of taps, aka
//! the polynomial that defines our field.
//!
//! From exploring [Galois-fields](../gf), we know that a good polynomial must be
//! "irreducible", that is it must not have any other factors or else we end up
//! with multiple smaller non-intersecting fields.
//!
//! We also know that every finite-field contains "primitive elements", which are
//! numbers in the field whose multiplicative cycle contains every element of the
//! field.
//!
//! So what we want, is an irreducible polynomial, where 2 is a primitive element
//! of the finite-field defined by said polynomial. If these conditions are met,
//! than repeated multiplications by 2 (shifting our LFSR) will actually iterate
//! through every non-zero element of our field before looping, giving us the
//! maximum cycle-length for any `n`-bit LFSR (or any pseudo-random number generator
//! with `n`-bits of state).
//!
//! These polynomials are sometimes called "primitive polynomials", but there's
//! nothing really magical about them. In order to find primitive polynomials, we
//! just brute force search all polynomials until we find one that works. There is
//! more information about this in [`gf`'s module-level documentation
//! ](../gf#finding-irreducible-polynomials-and-generators).
//!
//! Knowing that LFSRs are equivalent to polynomial multiplication and division,
//! we can also step through multiple bits at a time by multiplying by a
//! power-of-two:
//!
//! ``` rust
//! # use ::gf256::*;
//! #
//! let mut state: p32 = p32(0b11001100);
//! let mut step = |n: u32| {
//!     state = state * p32(2).pow(n);
//!     let output = state / p32(0x11d);
//!     state = state % p32(0x11d);
//!     u32::from(output)
//! };
//! assert_eq!(step(8), 0b11000101);
//! ```
//!
//! And we can create this exact LFSR using gf256:
//!
//! ``` rust
//! # pub use ::gf256::*;
//! use ::gf256::lfsr::lfsr;
//!
//! #[lfsr(polynomial=0x11d)]
//! struct Lfsr {}
//!
//! # fn main() {
//! let mut lfsr = Lfsr::new(0b11001100);
//! assert_eq!(lfsr.next(8), 0b11000101);
//! # }
//! ```
//!
//! ## Stepping backwards
//!
//! Fascinating to me was learning that you can actually run an LFSR _backwards_.
//! All you need to do is invert the taps and shift in the other direction:
//!
//! ``` text
//!         .-> 1 --> 0 --> 1 --> 0 -x> 1 -x> 0 -x> 0 --> 1 -.-> output
//!         '------------------------'-----'-----'-----------'
//!             internal state
//!             1     1     0     1     1     0     1     0       1
//!             0     1     1     0     1     1     0     1       0
//!             1     0     1     1     1     0     0     0       1
//!             0     1     0     1     1     1     0     0       0
//!             0     0     1     0     1     1     1     0       0
//!             0     0     0     1     0     1     1     1       0
//!             1     0     0     0     0     1     0     1       1
//!             1     1     0     0     1     1     0     0       1
//!             ...
//! ```
//!
//! This works perfectly fine with powers-of-two, allowing multiple bits to be
//! stepped through at a time:
//!
//! ``` rust
//! # use ::gf256::*;
//! #
//! let mut state: p32 = p32(0b10101001);
//! let mut step = |n: u32| {
//!     state = (state.reverse_bits() >> 32u32-8) * p32(2).pow(n);
//!     let output = (state / (p32(0x11d).reverse_bits() >> 32u32-9))
//!         .reverse_bits() >> 32u32-8;
//!     state = (state % (p32(0x11d).reverse_bits() >> 32u32-9))
//!         .reverse_bits() >> 32u32-8;
//!     u32::from(output)
//! };
//! assert_eq!(step(8), 0b11000101);
//! ```
//!
//! And is supported by the LFSR structs:
//!
//! ``` rust
//! # pub use ::gf256::*;
//! use ::gf256::lfsr::lfsr;
//!
//! #[lfsr(polynomial=0x11d)]
//! struct Lfsr {}
//!
//! # fn main() {
//! let mut lfsr = Lfsr::new(0b10101001);
//! assert_eq!(lfsr.prev(8), 0b11000101);
//! # }
//! ```
//!
//! However, as far as I can tell, this doesn't generalize to non-power-of-two
//! multiplicands, which is a real shame since it would provide a much more
//! efficient division implementation for general Galois-fields.
//!
//! ## Seeking
//!
//! Perhaps the most useful feature of modeling LFSRs as Galois-fields, is that we
//! can step through multiple bits at a time. In fact, if we don't care about the
//! output, we can skip to any position in the LFSR state by exponentiation.
//!
//! If we use an efficient exponentation algorithm, such as [exponentiation by
//! squaring][exp-by-squaring], we can seek to any position in the LFSR state with
//! only `O(log log n)` multiplications:
//!
//! ``` rust
//! # use ::gf256::*;
//! #
//! let mut state: p32 = p32(0b11001100);
//! let mut skip = |mut n: u32| {
//!     // Binary exponentiation
//!     let mut a = p32(2);
//!     let mut g = p32(1);
//!     loop {
//!         if n & 1 != 0 {
//!             g = (g * a) % p32(0x11d);
//!         }
//!
//!         n >>= 1;
//!         if n == 0 {
//!             break;
//!         }
//!         a = (a * a) % p32(0x11d);
//!     };
//!
//!     // Final multiplication
//!     state = (state * g) % p32(0x11d);
//! };
//! skip(100);
//!
//! let mut step = |n: u32| {
//!     state = state * p32(2).pow(n);
//!     let output = state / p32(0x11d);
//!     state = state % p32(0x11d);
//!     u32::from(output)
//! };
//! assert_eq!(step(8), 0b10011111);
//! ```
//!
//! And with our LFSR structs:
//!
//! ``` rust
//! # pub use ::gf256::*;
//! use ::gf256::lfsr::lfsr;
//!
//! #[lfsr(polynomial=0x11d)]
//! struct Lfsr {}
//!
//! # fn main() {
//! let mut lfsr = Lfsr::new(0b11001100);
//! lfsr.skip(100);
//! assert_eq!(lfsr.next(8), 0b10011111);
//! # }
//! ```
//!
//! ## Optimizations
//!
//! Since LFSRs are equivalent to Galois-fields, they share a lot of the same
//! optimizations. With the exception that LFSRs need to worry about capturing
//! the quotient portion of division, since this forms the actual output bits.
//!
//! This naive implementation of LFSRs is available with the `naive` mode. Note this
//! uses the shift-and-xor implementation, since this is much faster than full
//! polynomial division/remainder without other optimizations. Additional modes:
//!
//! - In `table` mode, LFSRs use a precomputed division and remainder table to
//!   compute both the quotient and remainder a byte at a time.
//!
//!   This uses the same technique as the precomputed remainder tables in CRC
//!   calculation, since a CRC is just a polynomial remainder. The [crc](../crc)
//!   module-level documentation has more info on this.
//!
//! - In `small_table` mode, the same strategy as `table` mode is used, but
//!   with 16 element tables computing the quotient and remainder a nibble at a time.
//!
//! - In `barret` mode, LFSRs use [Barret-reduction][barret-reduction] to efficiently
//!   compute the remainder using only multiplication by precomputed constants.
//!
//!   However we still need to compute the quotient using naive polynomial division,
//!   making this mode less effective than in other types/utilities.
//!
//! - In `table_barret` mode, LFSRs use [Barret-reduction][barret-reduction] to find
//!   the remainder, and a precomputed division table to compute the quotient.
//!
//!   This seems to have the worst of both worlds, and does not perform that well.
//!
//! - In `small_table_barret` mode, the same strategy as `table_barret` mode is used,
//!   but with a 16 element table for the quotient, making this mode even less
//!   performant.
//!
//! By default `barret` mode is used for LFSR structs, as it outperforms all other
//! options, even when hardware carry-less multiplication is not available.
//!
//! Though note the default mode is susceptible to change.
//!
//! See also [BENCHMARKS.md][benchmarks]
//!
//! ## The `Rng` trait
//!
//! In addition to the above APIs, the LFSR structs in this module satisfy the
//! [`RngCore`](rand::RngCore) and [`SeedableRng`](rand::SeedableRng) traits found
//! in the [`rand`] crate. This allows custom LFSRs to act as drop-in replacements
//! for other pseudo-random number generators with the additional ability to seek and
//! rewind, allowing perfect replayability.
//!
//! Note! If you're just looking for a pseudo-random number generator, the
//! randomness generated by these LFSRs is equivalent to the same-sized, naive
//! [Xorshift generators][xorshift], with the same limitations and cycle-length.
//! However, Xorshift generators are much more efficient, using only a handful of
//! shifts and xors.
//!
//!
//! [lfsr-wiki]: https://en.wikipedia.org/wiki/Linear-feedback_shift_register
//! [exp-by-squaring]: https://en.wikipedia.org/wiki/Exponentiation_by_squaring
//! [barret-reduction]: https://en.wikipedia.org/wiki/Barrett_reduction
//! [xorshift]: https://en.wikipedia.org/wiki/Xorshift
//! [lfsr-example]: https://github.com/geky/gf256/blob/master/examples/lfsr.rs
//! [benchmarks]: https://github.com/geky/gf256/blob/master/BENCHMARKS.md


/// A macro for generating custom LFSR structs.
///
/// ``` rust
/// # use ::gf256::*;
/// # use ::gf256::lfsr::lfsr;
/// #[lfsr(polynomial=0x1002d)]
/// pub struct MyLfsr16 {}
///
/// # fn main() {
/// let mut lfsr = MyLfsr16::new(1);
/// assert_eq!(lfsr.next(16), 0x0001);
/// assert_eq!(lfsr.next(16), 0x002d);
/// assert_eq!(lfsr.next(16), 0x0451);
/// assert_eq!(lfsr.next(16), 0xbdad);
/// assert_eq!(lfsr.prev(16), 0xbdad);
/// assert_eq!(lfsr.prev(16), 0x0451);
/// assert_eq!(lfsr.prev(16), 0x002d);
/// assert_eq!(lfsr.prev(16), 0x0001);
/// # }
/// ```
///
/// The `lfsr` macro accepts a number of configuration options:
///
/// - `polynomial` - The irreducible polynomial that defines the LFSR.
/// - `u` - The underlying unsigned type, defaults to the minimum sized
///   unsigned type that fits the LFSR state space.
/// - `u2` - An unsigned type with twice the width, used as an intermediary type
///   for computations, defaults to the correct type based on `u`.
/// - `nzu` - The non-zero unsigned type, used to store the LFSR state with
///   niches, defaults to the non-zero unsigned version of `u`.
/// - `nzu2` - A non-zero unsigned type with twice the width, used as an
///   intermediary type for computations, defaults to the correct type based
///   on `nzu`.
/// - `p` - The polynomial type used for computation, defaults to the
///   polynomial version of `u`.
/// - `p2` - A polynomial type with twice the width, used as an intermediary type
///   for computations, defaults to the correct type based on `p`.
/// - `reflected` - Indicate if the LFSR should have its bits reversed,
///   defaults to false.
/// - `naive` - Use a naive bitwise implementation.
/// - `table` - Use precomputed quotient and remainder tables. This is the default.
/// - `small_table` - Use small, 16-element division and remainder tables.
/// - `barret` - Use Barret-reduction with polynomial multiplication.
/// - `table_barret` - Use Barret-reduction for the remainder, and a
///   precomputed division table for the quotient.
/// - `small_table_barret` - Use Barret-reduction for the remainder, and a small,
///   16-element division table for the quotient.
/// - `naive_skip` - Use a naive bitwise implementation to calculate skips.
/// - `table_skip` - Use a precomputed remainder table to calculate skips.
/// - `small_table_skip` - Use a small, 16-element remainder table to calculate skips.
/// - `barret_skip` - Use Barret-reduction with polynomial multiplication to
///   calculate skips. This is the default.
///
/// ``` rust
/// # use ::gf256::*;
/// # use ::gf256::lfsr::lfsr;
/// # use std::num::*;
/// #[lfsr(
///     polynomial=0x1002d,
///     u=u16,
///     u2=u32,
///     nzu=NonZeroU16,
///     nzu2=NonZeroU32,
///     p=p16,
///     p2=p32,
///     reflected=false,
///     // naive,
///     // table,
///     // small_table,
///     // barret,
///     // table_barret,
///     // small_table_barret,
///     // naive_skip,
///     // table_skip,
///     // small_table_skip,
///     // barret_skip,
/// )]
/// pub struct MyLfsr16 {}
///
/// # fn main() {
/// let mut lfsr = MyLfsr16::new(1);
/// assert_eq!(lfsr.next(16), 0x0001);
/// assert_eq!(lfsr.next(16), 0x002d);
/// assert_eq!(lfsr.next(16), 0x0451);
/// assert_eq!(lfsr.next(16), 0xbdad);
/// assert_eq!(lfsr.prev(16), 0xbdad);
/// assert_eq!(lfsr.prev(16), 0x0451);
/// assert_eq!(lfsr.prev(16), 0x002d);
/// assert_eq!(lfsr.prev(16), 0x0001);
/// # }
/// ```
///
pub use gf256_macros::lfsr;


// Default LFSR structs
//
#[lfsr(polynomial=0x11d)]
pub struct Lfsr8 {}
#[lfsr(polynomial=0x1002d)]
pub struct Lfsr16 {}
#[lfsr(polynomial=0x1000000af)]
pub struct Lfsr32 {}
#[lfsr(polynomial=0x1000000000000001b)]
pub struct Lfsr64 {}


#[cfg(test)]
mod test {
    use super::*;
    use crate::p::p64;
    use crate::p::p128;
    use core::num::NonZeroU64;
    use core::num::NonZeroU128;
    use core::iter::FromIterator;
    use rand::Rng;

    extern crate alloc;
    use alloc::vec::Vec;
    use alloc::vec;
    use alloc::collections::BTreeSet;

    extern crate std;
    use std::iter;

    #[test]
    fn lfsr() {
        let mut lfsr8 = Lfsr8::new(1);
        let buf = iter::repeat_with(|| lfsr8.next(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x01,0x1c,0x4b,0x81,0x92,0x6e,0x41,0x5b]);
        let buf = iter::repeat_with(|| lfsr8.prev(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x5b,0x41,0x6e,0x92,0x81,0x4b,0x1c,0x01]);

        let mut lfsr16 = Lfsr16::new(1);
        let buf = iter::repeat_with(|| lfsr16.next(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0001,0x002d,0x0451,0xbdad,0x13d3,0xb877,0x94e7,0xfcb8]);
        let buf = iter::repeat_with(|| lfsr16.prev(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xfcb8,0x94e7,0xb877,0x13d3,0xbdad,0x0451,0x002d,0x0001]);

        let mut lfsr32 = Lfsr32::new(1);
        let buf = iter::repeat_with(|| lfsr32.next(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x00000001,0x000000af,0x00004455,0x00295f23,0x1010111b,0xfafa511e,0x11579360,0x7d21bf13]);
        let buf = iter::repeat_with(|| lfsr32.prev(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7d21bf13,0x11579360,0xfafa511e,0x1010111b,0x00295f23,0x00004455,0x000000af,0x00000001]);

        let mut lfsr64 = Lfsr64::new(1);
        let buf = iter::repeat_with(|| lfsr64.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0000000000000001,0x000000000000001b,0x0000000000000145,0x0000000000001db7,0x0000000000011011,0x00000000001ab1ab,0x0000000001514515,0x000000001c6db6c7]);
        let buf = iter::repeat_with(|| lfsr64.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    #[test]
    fn lfsr_skip() {
        let mut lfsr8 = Lfsr8::new(1);
        lfsr8.skip(8*8);
        let buf = iter::repeat_with(|| lfsr8.prev(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x5b,0x41,0x6e,0x92,0x81,0x4b,0x1c,0x01]);

        let mut lfsr16 = Lfsr16::new(1);
        lfsr16.skip(16*8);
        let buf = iter::repeat_with(|| lfsr16.prev(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xfcb8,0x94e7,0xb877,0x13d3,0xbdad,0x0451,0x002d,0x0001]);

        let mut lfsr32 = Lfsr32::new(1);
        lfsr32.skip(32*8);
        let buf = iter::repeat_with(|| lfsr32.prev(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7d21bf13,0x11579360,0xfafa511e,0x1010111b,0x00295f23,0x00004455,0x000000af,0x00000001]);

        let mut lfsr64 = Lfsr64::new(1);
        lfsr64.skip(64*8);
        let buf = iter::repeat_with(|| lfsr64.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    #[test]
    fn lfsr_skip_backwards() {
        let mut lfsr8 = Lfsr8::new(1);
        lfsr8.skip(8*16);
        lfsr8.skip_backwards(8*8);
        let buf = iter::repeat_with(|| lfsr8.prev(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x5b,0x41,0x6e,0x92,0x81,0x4b,0x1c,0x01]);

        let mut lfsr16 = Lfsr16::new(1);
        lfsr16.skip(16*16);
        lfsr16.skip_backwards(16*8);
        let buf = iter::repeat_with(|| lfsr16.prev(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xfcb8,0x94e7,0xb877,0x13d3,0xbdad,0x0451,0x002d,0x0001]);

        let mut lfsr32 = Lfsr32::new(1);
        lfsr32.skip(32*16);
        lfsr32.skip_backwards(32*8);
        let buf = iter::repeat_with(|| lfsr32.prev(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7d21bf13,0x11579360,0xfafa511e,0x1010111b,0x00295f23,0x00004455,0x000000af,0x00000001]);

        let mut lfsr64 = Lfsr64::new(1);
        lfsr64.skip(64*16);
        lfsr64.skip_backwards(64*8);
        let buf = iter::repeat_with(|| lfsr64.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    // explicit modes
    #[lfsr(polynomial=0x11d, naive, naive_skip)]               pub struct Lfsr8Naive {}
    #[lfsr(polynomial=0x11d, table, table_skip)]               pub struct Lfsr8Table {}
    #[lfsr(polynomial=0x11d, small_table, small_table_skip)]   pub struct Lfsr8SmallTable {}
    #[lfsr(polynomial=0x11d, barret, barret_skip)]             pub struct Lfsr8Barret {}
    #[lfsr(polynomial=0x11d, table_barret, barret_skip)]       pub struct Lfsr8TableBarret {}
    #[lfsr(polynomial=0x11d, small_table_barret, barret_skip)] pub struct Lfsr8SmallTableBarret {}

    #[lfsr(polynomial=0x1002d, naive, naive_skip)]               pub struct Lfsr16Naive {}
    #[lfsr(polynomial=0x1002d, table, table_skip)]               pub struct Lfsr16Table {}
    #[lfsr(polynomial=0x1002d, small_table, small_table_skip)]   pub struct Lfsr16SmallTable {}
    #[lfsr(polynomial=0x1002d, barret, barret_skip)]             pub struct Lfsr16Barret {}
    #[lfsr(polynomial=0x1002d, table_barret, barret_skip)]       pub struct Lfsr16TableBarret {}
    #[lfsr(polynomial=0x1002d, small_table_barret, barret_skip)] pub struct Lfsr16SmallTableBarret {}

    #[lfsr(polynomial=0x1000000af, naive, naive_skip)]               pub struct Lfsr32Naive {}
    #[lfsr(polynomial=0x1000000af, table, table_skip)]               pub struct Lfsr32Table {}
    #[lfsr(polynomial=0x1000000af, small_table, small_table_skip)]   pub struct Lfsr32SmallTable {}
    #[lfsr(polynomial=0x1000000af, barret, barret_skip)]             pub struct Lfsr32Barret {}
    #[lfsr(polynomial=0x1000000af, table_barret, barret_skip)]       pub struct Lfsr32TableBarret {}
    #[lfsr(polynomial=0x1000000af, small_table_barret, barret_skip)] pub struct Lfsr32SmallTableBarret {}

    #[lfsr(polynomial=0x1000000000000001b, naive, naive_skip)]               pub struct Lfsr64Naive {}
    #[lfsr(polynomial=0x1000000000000001b, table, table_skip)]               pub struct Lfsr64Table {}
    #[lfsr(polynomial=0x1000000000000001b, small_table, small_table_skip)]   pub struct Lfsr64SmallTable {}
    #[lfsr(polynomial=0x1000000000000001b, barret, barret_skip)]             pub struct Lfsr64Barret {}
    #[lfsr(polynomial=0x1000000000000001b, table_barret, barret_skip)]       pub struct Lfsr64TableBarret {}
    #[lfsr(polynomial=0x1000000000000001b, small_table_barret, barret_skip)] pub struct Lfsr64SmallTableBarret {}

    // test explicit div/rem modes
    #[test]
    fn lfsr_naive() {
        let mut lfsr8_naive = Lfsr8Naive::new(1);
        let buf = iter::repeat_with(|| lfsr8_naive.next(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x01,0x1c,0x4b,0x81,0x92,0x6e,0x41,0x5b]);
        let buf = iter::repeat_with(|| lfsr8_naive.prev(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x5b,0x41,0x6e,0x92,0x81,0x4b,0x1c,0x01]);

        let mut lfsr16_naive = Lfsr16Naive::new(1);
        let buf = iter::repeat_with(|| lfsr16_naive.next(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0001,0x002d,0x0451,0xbdad,0x13d3,0xb877,0x94e7,0xfcb8]);
        let buf = iter::repeat_with(|| lfsr16_naive.prev(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xfcb8,0x94e7,0xb877,0x13d3,0xbdad,0x0451,0x002d,0x0001]);

        let mut lfsr32_naive = Lfsr32Naive::new(1);
        let buf = iter::repeat_with(|| lfsr32_naive.next(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x00000001,0x000000af,0x00004455,0x00295f23,0x1010111b,0xfafa511e,0x11579360,0x7d21bf13]);
        let buf = iter::repeat_with(|| lfsr32_naive.prev(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7d21bf13,0x11579360,0xfafa511e,0x1010111b,0x00295f23,0x00004455,0x000000af,0x00000001]);

        let mut lfsr64_naive = Lfsr64Naive::new(1);
        let buf = iter::repeat_with(|| lfsr64_naive.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0000000000000001,0x000000000000001b,0x0000000000000145,0x0000000000001db7,0x0000000000011011,0x00000000001ab1ab,0x0000000001514515,0x000000001c6db6c7]);
        let buf = iter::repeat_with(|| lfsr64_naive.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    #[test]
    fn lfsr_table() {
        let mut lfsr8_table = Lfsr8Table::new(1);
        let buf = iter::repeat_with(|| lfsr8_table.next(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x01,0x1c,0x4b,0x81,0x92,0x6e,0x41,0x5b]);
        let buf = iter::repeat_with(|| lfsr8_table.prev(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x5b,0x41,0x6e,0x92,0x81,0x4b,0x1c,0x01]);

        let mut lfsr16_table = Lfsr16Table::new(1);
        let buf = iter::repeat_with(|| lfsr16_table.next(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0001,0x002d,0x0451,0xbdad,0x13d3,0xb877,0x94e7,0xfcb8]);
        let buf = iter::repeat_with(|| lfsr16_table.prev(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xfcb8,0x94e7,0xb877,0x13d3,0xbdad,0x0451,0x002d,0x0001]);

        let mut lfsr32_table = Lfsr32Table::new(1);
        let buf = iter::repeat_with(|| lfsr32_table.next(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x00000001,0x000000af,0x00004455,0x00295f23,0x1010111b,0xfafa511e,0x11579360,0x7d21bf13]);
        let buf = iter::repeat_with(|| lfsr32_table.prev(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7d21bf13,0x11579360,0xfafa511e,0x1010111b,0x00295f23,0x00004455,0x000000af,0x00000001]);

        let mut lfsr64_table = Lfsr64Table::new(1);
        let buf = iter::repeat_with(|| lfsr64_table.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0000000000000001,0x000000000000001b,0x0000000000000145,0x0000000000001db7,0x0000000000011011,0x00000000001ab1ab,0x0000000001514515,0x000000001c6db6c7]);
        let buf = iter::repeat_with(|| lfsr64_table.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    #[test]
    fn lfsr_small_table() {
        let mut lfsr8_small_table = Lfsr8SmallTable::new(1);
        let buf = iter::repeat_with(|| lfsr8_small_table.next(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x01,0x1c,0x4b,0x81,0x92,0x6e,0x41,0x5b]);
        let buf = iter::repeat_with(|| lfsr8_small_table.prev(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x5b,0x41,0x6e,0x92,0x81,0x4b,0x1c,0x01]);

        let mut lfsr16_small_table = Lfsr16SmallTable::new(1);
        let buf = iter::repeat_with(|| lfsr16_small_table.next(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0001,0x002d,0x0451,0xbdad,0x13d3,0xb877,0x94e7,0xfcb8]);
        let buf = iter::repeat_with(|| lfsr16_small_table.prev(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xfcb8,0x94e7,0xb877,0x13d3,0xbdad,0x0451,0x002d,0x0001]);

        let mut lfsr32_small_table = Lfsr32SmallTable::new(1);
        let buf = iter::repeat_with(|| lfsr32_small_table.next(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x00000001,0x000000af,0x00004455,0x00295f23,0x1010111b,0xfafa511e,0x11579360,0x7d21bf13]);
        let buf = iter::repeat_with(|| lfsr32_small_table.prev(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7d21bf13,0x11579360,0xfafa511e,0x1010111b,0x00295f23,0x00004455,0x000000af,0x00000001]);

        let mut lfsr64_small_table = Lfsr64SmallTable::new(1);
        let buf = iter::repeat_with(|| lfsr64_small_table.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0000000000000001,0x000000000000001b,0x0000000000000145,0x0000000000001db7,0x0000000000011011,0x00000000001ab1ab,0x0000000001514515,0x000000001c6db6c7]);
        let buf = iter::repeat_with(|| lfsr64_small_table.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    #[test]
    fn lfsr_barret() {
        let mut lfsr8_barret = Lfsr8Barret::new(1);
        let buf = iter::repeat_with(|| lfsr8_barret.next(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x01,0x1c,0x4b,0x81,0x92,0x6e,0x41,0x5b]);
        let buf = iter::repeat_with(|| lfsr8_barret.prev(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x5b,0x41,0x6e,0x92,0x81,0x4b,0x1c,0x01]);

        let mut lfsr16_barret = Lfsr16Barret::new(1);
        let buf = iter::repeat_with(|| lfsr16_barret.next(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0001,0x002d,0x0451,0xbdad,0x13d3,0xb877,0x94e7,0xfcb8]);
        let buf = iter::repeat_with(|| lfsr16_barret.prev(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xfcb8,0x94e7,0xb877,0x13d3,0xbdad,0x0451,0x002d,0x0001]);

        let mut lfsr32_barret = Lfsr32Barret::new(1);
        let buf = iter::repeat_with(|| lfsr32_barret.next(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x00000001,0x000000af,0x00004455,0x00295f23,0x1010111b,0xfafa511e,0x11579360,0x7d21bf13]);
        let buf = iter::repeat_with(|| lfsr32_barret.prev(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7d21bf13,0x11579360,0xfafa511e,0x1010111b,0x00295f23,0x00004455,0x000000af,0x00000001]);

        let mut lfsr64_barret = Lfsr64Barret::new(1);
        let buf = iter::repeat_with(|| lfsr64_barret.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0000000000000001,0x000000000000001b,0x0000000000000145,0x0000000000001db7,0x0000000000011011,0x00000000001ab1ab,0x0000000001514515,0x000000001c6db6c7]);
        let buf = iter::repeat_with(|| lfsr64_barret.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    #[test]
    fn lfsr_table_barret() {
        let mut lfsr8_table_barret = Lfsr8TableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr8_table_barret.next(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x01,0x1c,0x4b,0x81,0x92,0x6e,0x41,0x5b]);
        let buf = iter::repeat_with(|| lfsr8_table_barret.prev(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x5b,0x41,0x6e,0x92,0x81,0x4b,0x1c,0x01]);

        let mut lfsr16_table_barret = Lfsr16TableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr16_table_barret.next(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0001,0x002d,0x0451,0xbdad,0x13d3,0xb877,0x94e7,0xfcb8]);
        let buf = iter::repeat_with(|| lfsr16_table_barret.prev(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xfcb8,0x94e7,0xb877,0x13d3,0xbdad,0x0451,0x002d,0x0001]);

        let mut lfsr32_table_barret = Lfsr32TableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr32_table_barret.next(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x00000001,0x000000af,0x00004455,0x00295f23,0x1010111b,0xfafa511e,0x11579360,0x7d21bf13]);
        let buf = iter::repeat_with(|| lfsr32_table_barret.prev(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7d21bf13,0x11579360,0xfafa511e,0x1010111b,0x00295f23,0x00004455,0x000000af,0x00000001]);

        let mut lfsr64_table_barret = Lfsr64TableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr64_table_barret.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0000000000000001,0x000000000000001b,0x0000000000000145,0x0000000000001db7,0x0000000000011011,0x00000000001ab1ab,0x0000000001514515,0x000000001c6db6c7]);
        let buf = iter::repeat_with(|| lfsr64_table_barret.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    #[test]
    fn lfsr_small_table_barret() {
        let mut lfsr8_small_table_barret = Lfsr8SmallTableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr8_small_table_barret.next(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x01,0x1c,0x4b,0x81,0x92,0x6e,0x41,0x5b]);
        let buf = iter::repeat_with(|| lfsr8_small_table_barret.prev(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x5b,0x41,0x6e,0x92,0x81,0x4b,0x1c,0x01]);

        let mut lfsr16_small_table_barret = Lfsr16SmallTableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr16_small_table_barret.next(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0001,0x002d,0x0451,0xbdad,0x13d3,0xb877,0x94e7,0xfcb8]);
        let buf = iter::repeat_with(|| lfsr16_small_table_barret.prev(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xfcb8,0x94e7,0xb877,0x13d3,0xbdad,0x0451,0x002d,0x0001]);

        let mut lfsr32_small_table_barret = Lfsr32SmallTableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr32_small_table_barret.next(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x00000001,0x000000af,0x00004455,0x00295f23,0x1010111b,0xfafa511e,0x11579360,0x7d21bf13]);
        let buf = iter::repeat_with(|| lfsr32_small_table_barret.prev(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7d21bf13,0x11579360,0xfafa511e,0x1010111b,0x00295f23,0x00004455,0x000000af,0x00000001]);

        let mut lfsr64_small_table_barret = Lfsr64SmallTableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr64_small_table_barret.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0000000000000001,0x000000000000001b,0x0000000000000145,0x0000000000001db7,0x0000000000011011,0x00000000001ab1ab,0x0000000001514515,0x000000001c6db6c7]);
        let buf = iter::repeat_with(|| lfsr64_small_table_barret.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    // test explicit skip modes
    #[test]
    fn lfsr_naive_skip() {
        let mut lfsr8_naive = Lfsr8Naive::new(1);
        lfsr8_naive.skip(8*8);
        let buf = iter::repeat_with(|| lfsr8_naive.prev(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x5b,0x41,0x6e,0x92,0x81,0x4b,0x1c,0x01]);

        let mut lfsr16_naive = Lfsr16Naive::new(1);
        lfsr16_naive.skip(16*8);
        let buf = iter::repeat_with(|| lfsr16_naive.prev(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xfcb8,0x94e7,0xb877,0x13d3,0xbdad,0x0451,0x002d,0x0001]);

        let mut lfsr32_naive = Lfsr32Naive::new(1);
        lfsr32_naive.skip(32*8);
        let buf = iter::repeat_with(|| lfsr32_naive.prev(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7d21bf13,0x11579360,0xfafa511e,0x1010111b,0x00295f23,0x00004455,0x000000af,0x00000001]);

        let mut lfsr64_naive = Lfsr64Naive::new(1);
        lfsr64_naive.skip(64*8);
        let buf = iter::repeat_with(|| lfsr64_naive.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    #[test]
    fn lfsr_table_skip() {
        let mut lfsr8_table = Lfsr8Table::new(1);
        lfsr8_table.skip(8*8);
        let buf = iter::repeat_with(|| lfsr8_table.prev(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x5b,0x41,0x6e,0x92,0x81,0x4b,0x1c,0x01]);

        let mut lfsr16_table = Lfsr16Table::new(1);
        lfsr16_table.skip(16*8);
        let buf = iter::repeat_with(|| lfsr16_table.prev(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xfcb8,0x94e7,0xb877,0x13d3,0xbdad,0x0451,0x002d,0x0001]);

        let mut lfsr32_table = Lfsr32Table::new(1);
        lfsr32_table.skip(32*8);
        let buf = iter::repeat_with(|| lfsr32_table.prev(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7d21bf13,0x11579360,0xfafa511e,0x1010111b,0x00295f23,0x00004455,0x000000af,0x00000001]);

        let mut lfsr64_table = Lfsr64Table::new(1);
        lfsr64_table.skip(64*8);
        let buf = iter::repeat_with(|| lfsr64_table.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    #[test]
    fn lfsr_small_table_skip() {
        let mut lfsr8_small_table = Lfsr8SmallTable::new(1);
        lfsr8_small_table.skip(8*8);
        let buf = iter::repeat_with(|| lfsr8_small_table.prev(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x5b,0x41,0x6e,0x92,0x81,0x4b,0x1c,0x01]);

        let mut lfsr16_small_table = Lfsr16SmallTable::new(1);
        lfsr16_small_table.skip(16*8);
        let buf = iter::repeat_with(|| lfsr16_small_table.prev(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xfcb8,0x94e7,0xb877,0x13d3,0xbdad,0x0451,0x002d,0x0001]);

        let mut lfsr32_small_table = Lfsr32SmallTable::new(1);
        lfsr32_small_table.skip(32*8);
        let buf = iter::repeat_with(|| lfsr32_small_table.prev(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7d21bf13,0x11579360,0xfafa511e,0x1010111b,0x00295f23,0x00004455,0x000000af,0x00000001]);

        let mut lfsr64_small_table = Lfsr64SmallTable::new(1);
        lfsr64_small_table.skip(64*8);
        let buf = iter::repeat_with(|| lfsr64_small_table.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    #[test]
    fn lfsr_barret_skip() {
        let mut lfsr8_barret = Lfsr8Barret::new(1);
        lfsr8_barret.skip(8*8);
        let buf = iter::repeat_with(|| lfsr8_barret.prev(8)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x5b,0x41,0x6e,0x92,0x81,0x4b,0x1c,0x01]);

        let mut lfsr16_barret = Lfsr16Barret::new(1);
        lfsr16_barret.skip(16*8);
        let buf = iter::repeat_with(|| lfsr16_barret.prev(16)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xfcb8,0x94e7,0xb877,0x13d3,0xbdad,0x0451,0x002d,0x0001]);

        let mut lfsr32_barret = Lfsr32Barret::new(1);
        lfsr32_barret.skip(32*8);
        let buf = iter::repeat_with(|| lfsr32_barret.prev(32)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7d21bf13,0x11579360,0xfafa511e,0x1010111b,0x00295f23,0x00004455,0x000000af,0x00000001]);

        let mut lfsr64_barret = Lfsr64Barret::new(1);
        lfsr64_barret.skip(64*8);
        let buf = iter::repeat_with(|| lfsr64_barret.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    // odd step sizes
    #[test]
    fn lfsr_odd_nexts() {
        let mut lfsr8 = Lfsr8Naive::new(1);
        let buf = iter::repeat_with(|| lfsr8.next(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0,0x1,0x1,0xc,0x4,0xb,0x8,0x1]);
        let buf = iter::repeat_with(|| lfsr8.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x1,0x8,0xb,0x4,0xc,0x1,0x1,0x0]);
        let mut lfsr16 = Lfsr16Naive::new(1);
        let buf = iter::repeat_with(|| lfsr16.next(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000,0x100,0x2d0,0x451,0xbda,0xd13,0xd3b,0x877]);
        let buf = iter::repeat_with(|| lfsr16.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x877,0xd3b,0xd13,0xbda,0x451,0x2d0,0x100,0x000]);
        let mut lfsr32 = Lfsr32Naive::new(1);
        let buf = iter::repeat_with(|| lfsr32.next(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000,0x004000,0x0015e0,0x000445,0x28014a,0x7c8c40,0x202237,0x7afa51]);
        let buf = iter::repeat_with(|| lfsr32.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7afa51,0x202237,0x7c8c40,0x28014a,0x000445,0x0015e0,0x004000,0x000000]);

        let mut lfsr8 = Lfsr8Table::new(1);
        let buf = iter::repeat_with(|| lfsr8.next(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0,0x1,0x1,0xc,0x4,0xb,0x8,0x1]);
        let buf = iter::repeat_with(|| lfsr8.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x1,0x8,0xb,0x4,0xc,0x1,0x1,0x0]);
        let mut lfsr16 = Lfsr16Table::new(1);
        let buf = iter::repeat_with(|| lfsr16.next(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000,0x100,0x2d0,0x451,0xbda,0xd13,0xd3b,0x877]);
        let buf = iter::repeat_with(|| lfsr16.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x877,0xd3b,0xd13,0xbda,0x451,0x2d0,0x100,0x000]);
        let mut lfsr32 = Lfsr32Table::new(1);
        let buf = iter::repeat_with(|| lfsr32.next(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000,0x004000,0x0015e0,0x000445,0x28014a,0x7c8c40,0x202237,0x7afa51]);
        let buf = iter::repeat_with(|| lfsr32.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7afa51,0x202237,0x7c8c40,0x28014a,0x000445,0x0015e0,0x004000,0x000000]);

        let mut lfsr8 = Lfsr8SmallTable::new(1);
        let buf = iter::repeat_with(|| lfsr8.next(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0,0x1,0x1,0xc,0x4,0xb,0x8,0x1]);
        let buf = iter::repeat_with(|| lfsr8.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x1,0x8,0xb,0x4,0xc,0x1,0x1,0x0]);
        let mut lfsr16 = Lfsr16SmallTable::new(1);
        let buf = iter::repeat_with(|| lfsr16.next(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000,0x100,0x2d0,0x451,0xbda,0xd13,0xd3b,0x877]);
        let buf = iter::repeat_with(|| lfsr16.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x877,0xd3b,0xd13,0xbda,0x451,0x2d0,0x100,0x000]);
        let mut lfsr32 = Lfsr32SmallTable::new(1);
        let buf = iter::repeat_with(|| lfsr32.next(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000,0x004000,0x0015e0,0x000445,0x28014a,0x7c8c40,0x202237,0x7afa51]);
        let buf = iter::repeat_with(|| lfsr32.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7afa51,0x202237,0x7c8c40,0x28014a,0x000445,0x0015e0,0x004000,0x000000]);

        let mut lfsr8 = Lfsr8Barret::new(1);
        let buf = iter::repeat_with(|| lfsr8.next(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0,0x1,0x1,0xc,0x4,0xb,0x8,0x1]);
        let buf = iter::repeat_with(|| lfsr8.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x1,0x8,0xb,0x4,0xc,0x1,0x1,0x0]);
        let mut lfsr16 = Lfsr16Barret::new(1);
        let buf = iter::repeat_with(|| lfsr16.next(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000,0x100,0x2d0,0x451,0xbda,0xd13,0xd3b,0x877]);
        let buf = iter::repeat_with(|| lfsr16.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x877,0xd3b,0xd13,0xbda,0x451,0x2d0,0x100,0x000]);
        let mut lfsr32 = Lfsr32Barret::new(1);
        let buf = iter::repeat_with(|| lfsr32.next(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000,0x004000,0x0015e0,0x000445,0x28014a,0x7c8c40,0x202237,0x7afa51]);
        let buf = iter::repeat_with(|| lfsr32.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7afa51,0x202237,0x7c8c40,0x28014a,0x000445,0x0015e0,0x004000,0x000000]);

        let mut lfsr8 = Lfsr8TableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr8.next(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0,0x1,0x1,0xc,0x4,0xb,0x8,0x1]);
        let buf = iter::repeat_with(|| lfsr8.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x1,0x8,0xb,0x4,0xc,0x1,0x1,0x0]);
        let mut lfsr16 = Lfsr16TableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr16.next(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000,0x100,0x2d0,0x451,0xbda,0xd13,0xd3b,0x877]);
        let buf = iter::repeat_with(|| lfsr16.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x877,0xd3b,0xd13,0xbda,0x451,0x2d0,0x100,0x000]);
        let mut lfsr32 = Lfsr32TableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr32.next(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000,0x004000,0x0015e0,0x000445,0x28014a,0x7c8c40,0x202237,0x7afa51]);
        let buf = iter::repeat_with(|| lfsr32.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7afa51,0x202237,0x7c8c40,0x28014a,0x000445,0x0015e0,0x004000,0x000000]);

        let mut lfsr8 = Lfsr8SmallTableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr8.next(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0,0x1,0x1,0xc,0x4,0xb,0x8,0x1]);
        let buf = iter::repeat_with(|| lfsr8.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x1,0x8,0xb,0x4,0xc,0x1,0x1,0x0]);
        let mut lfsr16 = Lfsr16SmallTableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr16.next(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000,0x100,0x2d0,0x451,0xbda,0xd13,0xd3b,0x877]);
        let buf = iter::repeat_with(|| lfsr16.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x877,0xd3b,0xd13,0xbda,0x451,0x2d0,0x100,0x000]);
        let mut lfsr32 = Lfsr32SmallTableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr32.next(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000,0x004000,0x0015e0,0x000445,0x28014a,0x7c8c40,0x202237,0x7afa51]);
        let buf = iter::repeat_with(|| lfsr32.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x7afa51,0x202237,0x7c8c40,0x28014a,0x000445,0x0015e0,0x004000,0x000000]);
    }

    // odd LFSR sizes
    #[lfsr(polynomial=0x13, naive, naive_skip)]               pub struct Lfsr4Naive {}
    #[lfsr(polynomial=0x13, table, table_skip)]               pub struct Lfsr4Table {}
    #[lfsr(polynomial=0x13, small_table, small_table_skip)]   pub struct Lfsr4SmallTable {}
    #[lfsr(polynomial=0x13, barret, barret_skip)]             pub struct Lfsr4Barret {}
    #[lfsr(polynomial=0x13, table_barret, barret_skip)]       pub struct Lfsr4TableBarret {}
    #[lfsr(polynomial=0x13, small_table_barret, barret_skip)] pub struct Lfsr4SmallTableBarret {}

    #[lfsr(polynomial=0x1053, naive, naive_skip)]               pub struct Lfsr12Naive {}
    #[lfsr(polynomial=0x1053, table, table_skip)]               pub struct Lfsr12Table {}
    #[lfsr(polynomial=0x1053, small_table, small_table_skip)]   pub struct Lfsr12SmallTable {}
    #[lfsr(polynomial=0x1053, barret, barret_skip)]             pub struct Lfsr12Barret {}
    #[lfsr(polynomial=0x1053, table_barret, barret_skip)]       pub struct Lfsr12TableBarret {}
    #[lfsr(polynomial=0x1053, small_table_barret, barret_skip)] pub struct Lfsr12SmallTableBarret {}

    #[lfsr(polynomial=0x800021, naive, naive_skip)]               pub struct Lfsr23Naive {}
    #[lfsr(polynomial=0x800021, table, table_skip)]               pub struct Lfsr23Table {}
    #[lfsr(polynomial=0x800021, small_table, small_table_skip)]   pub struct Lfsr23SmallTable {}
    #[lfsr(polynomial=0x800021, barret, barret_skip)]             pub struct Lfsr23Barret {}
    #[lfsr(polynomial=0x800021, table_barret, barret_skip)]       pub struct Lfsr23TableBarret {}
    #[lfsr(polynomial=0x800021, small_table_barret, barret_skip)] pub struct Lfsr23SmallTableBarret {}

    #[test]
    fn lfsr_odd_sizes() {
        let mut lfsr4 = Lfsr4Naive::new(1);
        let buf = iter::repeat_with(|| lfsr4.next(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x1,0x3,0x5,0xe,0x2,0x6,0xb,0xc]);
        let buf = iter::repeat_with(|| lfsr4.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xc,0xb,0x6,0x2,0xe,0x5,0x3,0x1]);
        let mut lfsr12 = Lfsr12Naive::new(1);
        let buf = iter::repeat_with(|| lfsr12.next(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x001,0x052,0x152,0x25d,0x462,0x20c,0x5c6,0x3a7]);
        let buf = iter::repeat_with(|| lfsr12.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x3a7,0x5c6,0x20c,0x462,0x25d,0x152,0x052,0x001]);
        let mut lfsr23 = Lfsr23Naive::new(1);
        let buf = iter::repeat_with(|| lfsr23.next(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000001,0x000021,0x000401,0x008421,0x100005,0x1000a1,0x101485,0x128421]);
        let buf = iter::repeat_with(|| lfsr23.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x128421,0x101485,0x1000a1,0x100005,0x008421,0x000401,0x000021,0x000001]);

        let mut lfsr4 = Lfsr4Table::new(1);
        let buf = iter::repeat_with(|| lfsr4.next(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x1,0x3,0x5,0xe,0x2,0x6,0xb,0xc]);
        let buf = iter::repeat_with(|| lfsr4.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xc,0xb,0x6,0x2,0xe,0x5,0x3,0x1]);
        let mut lfsr12 = Lfsr12Table::new(1);
        let buf = iter::repeat_with(|| lfsr12.next(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x001,0x052,0x152,0x25d,0x462,0x20c,0x5c6,0x3a7]);
        let buf = iter::repeat_with(|| lfsr12.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x3a7,0x5c6,0x20c,0x462,0x25d,0x152,0x052,0x001]);
        let mut lfsr23 = Lfsr23Table::new(1);
        let buf = iter::repeat_with(|| lfsr23.next(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000001,0x000021,0x000401,0x008421,0x100005,0x1000a1,0x101485,0x128421]);
        let buf = iter::repeat_with(|| lfsr23.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x128421,0x101485,0x1000a1,0x100005,0x008421,0x000401,0x000021,0x000001]);

        let mut lfsr4 = Lfsr4SmallTable::new(1);
        let buf = iter::repeat_with(|| lfsr4.next(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x1,0x3,0x5,0xe,0x2,0x6,0xb,0xc]);
        let buf = iter::repeat_with(|| lfsr4.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xc,0xb,0x6,0x2,0xe,0x5,0x3,0x1]);
        let mut lfsr12 = Lfsr12SmallTable::new(1);
        let buf = iter::repeat_with(|| lfsr12.next(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x001,0x052,0x152,0x25d,0x462,0x20c,0x5c6,0x3a7]);
        let buf = iter::repeat_with(|| lfsr12.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x3a7,0x5c6,0x20c,0x462,0x25d,0x152,0x052,0x001]);
        let mut lfsr23 = Lfsr23SmallTable::new(1);
        let buf = iter::repeat_with(|| lfsr23.next(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000001,0x000021,0x000401,0x008421,0x100005,0x1000a1,0x101485,0x128421]);
        let buf = iter::repeat_with(|| lfsr23.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x128421,0x101485,0x1000a1,0x100005,0x008421,0x000401,0x000021,0x000001]);

        let mut lfsr4 = Lfsr4Barret::new(1);
        let buf = iter::repeat_with(|| lfsr4.next(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x1,0x3,0x5,0xe,0x2,0x6,0xb,0xc]);
        let buf = iter::repeat_with(|| lfsr4.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xc,0xb,0x6,0x2,0xe,0x5,0x3,0x1]);
        let mut lfsr12 = Lfsr12Barret::new(1);
        let buf = iter::repeat_with(|| lfsr12.next(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x001,0x052,0x152,0x25d,0x462,0x20c,0x5c6,0x3a7]);
        let buf = iter::repeat_with(|| lfsr12.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x3a7,0x5c6,0x20c,0x462,0x25d,0x152,0x052,0x001]);
        let mut lfsr23 = Lfsr23Barret::new(1);
        let buf = iter::repeat_with(|| lfsr23.next(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000001,0x000021,0x000401,0x008421,0x100005,0x1000a1,0x101485,0x128421]);
        let buf = iter::repeat_with(|| lfsr23.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x128421,0x101485,0x1000a1,0x100005,0x008421,0x000401,0x000021,0x000001]);

        let mut lfsr4 = Lfsr4TableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr4.next(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x1,0x3,0x5,0xe,0x2,0x6,0xb,0xc]);
        let buf = iter::repeat_with(|| lfsr4.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xc,0xb,0x6,0x2,0xe,0x5,0x3,0x1]);
        let mut lfsr12 = Lfsr12TableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr12.next(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x001,0x052,0x152,0x25d,0x462,0x20c,0x5c6,0x3a7]);
        let buf = iter::repeat_with(|| lfsr12.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x3a7,0x5c6,0x20c,0x462,0x25d,0x152,0x052,0x001]);
        let mut lfsr23 = Lfsr23TableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr23.next(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000001,0x000021,0x000401,0x008421,0x100005,0x1000a1,0x101485,0x128421]);
        let buf = iter::repeat_with(|| lfsr23.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x128421,0x101485,0x1000a1,0x100005,0x008421,0x000401,0x000021,0x000001]);

        let mut lfsr4 = Lfsr4SmallTableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr4.next(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x1,0x3,0x5,0xe,0x2,0x6,0xb,0xc]);
        let buf = iter::repeat_with(|| lfsr4.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xc,0xb,0x6,0x2,0xe,0x5,0x3,0x1]);
        let mut lfsr12 = Lfsr12SmallTableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr12.next(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x001,0x052,0x152,0x25d,0x462,0x20c,0x5c6,0x3a7]);
        let buf = iter::repeat_with(|| lfsr12.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x3a7,0x5c6,0x20c,0x462,0x25d,0x152,0x052,0x001]);
        let mut lfsr23 = Lfsr23SmallTableBarret::new(1);
        let buf = iter::repeat_with(|| lfsr23.next(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000001,0x000021,0x000401,0x008421,0x100005,0x1000a1,0x101485,0x128421]);
        let buf = iter::repeat_with(|| lfsr23.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x128421,0x101485,0x1000a1,0x100005,0x008421,0x000401,0x000021,0x000001]);
    }

    #[test]
    fn lfsr_odd_sizes_skip() {
        let mut lfsr4 = Lfsr4Naive::new(1);
        lfsr4.skip(4*8);
        let buf = iter::repeat_with(|| lfsr4.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xc,0xb,0x6,0x2,0xe,0x5,0x3,0x1]);
        let mut lfsr12 = Lfsr12Naive::new(1);
        lfsr12.skip(12*8);
        let buf = iter::repeat_with(|| lfsr12.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x3a7,0x5c6,0x20c,0x462,0x25d,0x152,0x052,0x001]);
        let mut lfsr23 = Lfsr23Naive::new(1);
        lfsr23.skip(23*8);
        let buf = iter::repeat_with(|| lfsr23.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x128421,0x101485,0x1000a1,0x100005,0x008421,0x000401,0x000021,0x000001]);

        let mut lfsr4 = Lfsr4Table::new(1);
        lfsr4.skip(4*8);
        let buf = iter::repeat_with(|| lfsr4.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xc,0xb,0x6,0x2,0xe,0x5,0x3,0x1]);
        let mut lfsr12 = Lfsr12Table::new(1);
        lfsr12.skip(12*8);
        let buf = iter::repeat_with(|| lfsr12.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x3a7,0x5c6,0x20c,0x462,0x25d,0x152,0x052,0x001]);
        let mut lfsr23 = Lfsr23Table::new(1);
        lfsr23.skip(23*8);
        let buf = iter::repeat_with(|| lfsr23.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x128421,0x101485,0x1000a1,0x100005,0x008421,0x000401,0x000021,0x000001]);

        let mut lfsr4 = Lfsr4SmallTable::new(1);
        lfsr4.skip(4*8);
        let buf = iter::repeat_with(|| lfsr4.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xc,0xb,0x6,0x2,0xe,0x5,0x3,0x1]);
        let mut lfsr12 = Lfsr12SmallTable::new(1);
        lfsr12.skip(12*8);
        let buf = iter::repeat_with(|| lfsr12.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x3a7,0x5c6,0x20c,0x462,0x25d,0x152,0x052,0x001]);
        let mut lfsr23 = Lfsr23SmallTable::new(1);
        lfsr23.skip(23*8);
        let buf = iter::repeat_with(|| lfsr23.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x128421,0x101485,0x1000a1,0x100005,0x008421,0x000401,0x000021,0x000001]);

        let mut lfsr4 = Lfsr4Barret::new(1);
        lfsr4.skip(4*8);
        let buf = iter::repeat_with(|| lfsr4.prev(4)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xc,0xb,0x6,0x2,0xe,0x5,0x3,0x1]);
        let mut lfsr12 = Lfsr12Barret::new(1);
        lfsr12.skip(12*8);
        let buf = iter::repeat_with(|| lfsr12.prev(12)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x3a7,0x5c6,0x20c,0x462,0x25d,0x152,0x052,0x001]);
        let mut lfsr23 = Lfsr23Barret::new(1);
        lfsr23.skip(23*8);
        let buf = iter::repeat_with(|| lfsr23.prev(23)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x128421,0x101485,0x1000a1,0x100005,0x008421,0x000401,0x000021,0x000001]);
    }

    // bit-reflected LFSRs
    #[lfsr(polynomial=0x1000000000000001b, naive, naive_skip, reflected=true)]               pub struct Lfsr64NaiveReflected {}
    #[lfsr(polynomial=0x1000000000000001b, table, table_skip, reflected=true)]               pub struct Lfsr64TableReflected {}
    #[lfsr(polynomial=0x1000000000000001b, small_table, small_table_skip, reflected=true)]   pub struct Lfsr64SmallTableReflected {}
    #[lfsr(polynomial=0x1000000000000001b, barret, barret_skip, reflected=true)]             pub struct Lfsr64BarretReflected {}
    #[lfsr(polynomial=0x1000000000000001b, table_barret, barret_skip, reflected=true)]       pub struct Lfsr64TableBarretReflected {}
    #[lfsr(polynomial=0x1000000000000001b, small_table_barret, barret_skip, reflected=true)] pub struct Lfsr64SmallTableBarretReflected {}

    #[test]
    fn lfsr_reflected() {
        let mut lfsr64 = Lfsr64NaiveReflected::new(0x8000000000000000);
        let buf = iter::repeat_with(|| lfsr64.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x8000000000000000,0xd800000000000000,0xa280000000000000,0xedb8000000000000,0x8808800000000000,0xd58d580000000000,0xa8a28a8000000000,0xe36db63800000000]);
        let buf = iter::repeat_with(|| lfsr64.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xe36db63800000000,0xa8a28a8000000000,0xd58d580000000000,0x8808800000000000,0xedb8000000000000,0xa280000000000000,0xd800000000000000,0x8000000000000000]);

        let mut lfsr64 = Lfsr64TableReflected::new(0x8000000000000000);
        let buf = iter::repeat_with(|| lfsr64.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x8000000000000000,0xd800000000000000,0xa280000000000000,0xedb8000000000000,0x8808800000000000,0xd58d580000000000,0xa8a28a8000000000,0xe36db63800000000]);
        let buf = iter::repeat_with(|| lfsr64.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xe36db63800000000,0xa8a28a8000000000,0xd58d580000000000,0x8808800000000000,0xedb8000000000000,0xa280000000000000,0xd800000000000000,0x8000000000000000]);

        let mut lfsr64 = Lfsr64SmallTableReflected::new(0x8000000000000000);
        let buf = iter::repeat_with(|| lfsr64.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x8000000000000000,0xd800000000000000,0xa280000000000000,0xedb8000000000000,0x8808800000000000,0xd58d580000000000,0xa8a28a8000000000,0xe36db63800000000]);
        let buf = iter::repeat_with(|| lfsr64.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xe36db63800000000,0xa8a28a8000000000,0xd58d580000000000,0x8808800000000000,0xedb8000000000000,0xa280000000000000,0xd800000000000000,0x8000000000000000]);

        let mut lfsr64 = Lfsr64BarretReflected::new(0x8000000000000000);
        let buf = iter::repeat_with(|| lfsr64.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x8000000000000000,0xd800000000000000,0xa280000000000000,0xedb8000000000000,0x8808800000000000,0xd58d580000000000,0xa8a28a8000000000,0xe36db63800000000]);
        let buf = iter::repeat_with(|| lfsr64.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xe36db63800000000,0xa8a28a8000000000,0xd58d580000000000,0x8808800000000000,0xedb8000000000000,0xa280000000000000,0xd800000000000000,0x8000000000000000]);

        let mut lfsr64 = Lfsr64TableBarretReflected::new(0x8000000000000000);
        let buf = iter::repeat_with(|| lfsr64.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x8000000000000000,0xd800000000000000,0xa280000000000000,0xedb8000000000000,0x8808800000000000,0xd58d580000000000,0xa8a28a8000000000,0xe36db63800000000]);
        let buf = iter::repeat_with(|| lfsr64.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xe36db63800000000,0xa8a28a8000000000,0xd58d580000000000,0x8808800000000000,0xedb8000000000000,0xa280000000000000,0xd800000000000000,0x8000000000000000]);

        let mut lfsr64 = Lfsr64SmallTableBarretReflected::new(0x8000000000000000);
        let buf = iter::repeat_with(|| lfsr64.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x8000000000000000,0xd800000000000000,0xa280000000000000,0xedb8000000000000,0x8808800000000000,0xd58d580000000000,0xa8a28a8000000000,0xe36db63800000000]);
        let buf = iter::repeat_with(|| lfsr64.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xe36db63800000000,0xa8a28a8000000000,0xd58d580000000000,0x8808800000000000,0xedb8000000000000,0xa280000000000000,0xd800000000000000,0x8000000000000000]);
    }

    #[test]
    fn lfsr_reflected_skip() {
        let mut lfsr64 = Lfsr64NaiveReflected::new(0x8000000000000000);
        lfsr64.skip(64*8);
        let buf = iter::repeat_with(|| lfsr64.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xe36db63800000000,0xa8a28a8000000000,0xd58d580000000000,0x8808800000000000,0xedb8000000000000,0xa280000000000000,0xd800000000000000,0x8000000000000000]);

        let mut lfsr64 = Lfsr64TableReflected::new(0x8000000000000000);
        lfsr64.skip(64*8);
        let buf = iter::repeat_with(|| lfsr64.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xe36db63800000000,0xa8a28a8000000000,0xd58d580000000000,0x8808800000000000,0xedb8000000000000,0xa280000000000000,0xd800000000000000,0x8000000000000000]);

        let mut lfsr64 = Lfsr64SmallTableReflected::new(0x8000000000000000);
        lfsr64.skip(64*8);
        let buf = iter::repeat_with(|| lfsr64.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xe36db63800000000,0xa8a28a8000000000,0xd58d580000000000,0x8808800000000000,0xedb8000000000000,0xa280000000000000,0xd800000000000000,0x8000000000000000]);

        let mut lfsr64 = Lfsr64BarretReflected::new(0x8000000000000000);
        lfsr64.skip(64*8);
        let buf = iter::repeat_with(|| lfsr64.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0xe36db63800000000,0xa8a28a8000000000,0xd58d580000000000,0x8808800000000000,0xedb8000000000000,0xa280000000000000,0xd800000000000000,0x8000000000000000]);
    }

    // all LFSR params
    #[lfsr(
        polynomial=0x1000000000000001b,
        u=u64,
        u2=u128,
        nzu=NonZeroU64,
        nzu2=NonZeroU128,
        p=p64,
        p2=p128,
        reflected=false,
    )]
    struct Lfsr64AllParams {}

    #[test]
    fn lfsr_all_params() {
        let mut lfsr64_all_params = Lfsr64AllParams::new(1);
        let buf = iter::repeat_with(|| lfsr64_all_params.next(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x0000000000000001,0x000000000000001b,0x0000000000000145,0x0000000000001db7,0x0000000000011011,0x00000000001ab1ab,0x0000000001514515,0x000000001c6db6c7]);
        let buf = iter::repeat_with(|| lfsr64_all_params.prev(64)).take(8).collect::<Vec<_>>();
        assert_eq!(buf, &[0x000000001c6db6c7,0x0000000001514515,0x00000000001ab1ab,0x0000000000011011,0x0000000000001db7,0x0000000000000145,0x000000000000001b,0x0000000000000001]);
    }

    // other LFSR things

    #[test]
    fn lfsr_rng_consistency() {
        // normal order
        let mut lfsr = Lfsr8::new(1);
        let next_bytes = iter::repeat_with(|| lfsr.next(8) as u8).take(100).collect::<Vec<_>>();
        let mut rng_bytes = vec![0u8; 100];
        let mut lfsr = Lfsr8::new(1);
        lfsr.fill(&mut rng_bytes[..]);
        assert_eq!(&next_bytes, &rng_bytes);

        let mut lfsr = Lfsr16::new(1);
        let next_bytes = iter::repeat_with(|| lfsr.next(8) as u8).take(100).collect::<Vec<_>>();
        let mut rng_bytes = vec![0u8; 100];
        let mut lfsr = Lfsr16::new(1);
        lfsr.fill(&mut rng_bytes[..]);
        assert_eq!(&next_bytes, &rng_bytes);

        let mut lfsr = Lfsr32::new(1);
        let next_bytes = iter::repeat_with(|| lfsr.next(8) as u8).take(100).collect::<Vec<_>>();
        let mut rng_bytes = vec![0u8; 100];
        let mut lfsr = Lfsr32::new(1);
        lfsr.fill(&mut rng_bytes[..]);
        assert_eq!(&next_bytes, &rng_bytes);

        let mut lfsr = Lfsr64::new(1);
        let next_bytes = iter::repeat_with(|| lfsr.next(8) as u8).take(100).collect::<Vec<_>>();
        let mut rng_bytes = vec![0u8; 100];
        let mut lfsr = Lfsr64::new(1);
        lfsr.fill(&mut rng_bytes[..]);
        assert_eq!(&next_bytes, &rng_bytes);

        let mut lfsr = Lfsr4Table::new(1);
        let next_bytes = iter::repeat_with(|| (lfsr.next(4) << 4) | lfsr.next(4)).take(100).collect::<Vec<_>>();
        let mut rng_bytes = vec![0u8; 100];
        let mut lfsr = Lfsr4Table::new(1);
        lfsr.fill(&mut rng_bytes[..]);
        assert_eq!(&next_bytes, &rng_bytes);

        let mut lfsr = Lfsr12Table::new(1);
        let next_bytes = iter::repeat_with(|| lfsr.next(8) as u8).take(100).collect::<Vec<_>>();
        let mut rng_bytes = vec![0u8; 100];
        let mut lfsr = Lfsr12Table::new(1);
        lfsr.fill(&mut rng_bytes[..]);
        assert_eq!(&next_bytes, &rng_bytes);

        let mut lfsr = Lfsr23Table::new(1);
        let next_bytes = iter::repeat_with(|| lfsr.next(8) as u8).take(100).collect::<Vec<_>>();
        let mut rng_bytes = vec![0u8; 100];
        let mut lfsr = Lfsr23Table::new(1);
        lfsr.fill(&mut rng_bytes[..]);
        assert_eq!(&next_bytes, &rng_bytes);

        // reflected order
        let mut lfsr = Lfsr64TableReflected::new(1);
        let next_bytes = iter::repeat_with(|| lfsr.next(8) as u8).take(100).collect::<Vec<_>>();
        let mut rng_bytes = vec![0u8; 100];
        let mut lfsr = Lfsr64TableReflected::new(1);
        lfsr.fill(&mut rng_bytes[..]);
        assert_eq!(&next_bytes, &rng_bytes);
    }

    #[test]
    fn lfsr_uniqueness() {
        let mut lfsr = Lfsr8::new(1);
        let unique = BTreeSet::from_iter(iter::repeat_with(|| lfsr.next(8)).take(255));
        assert_eq!(unique.len(), 255);

        let mut lfsr = Lfsr64::new(1);
        let unique = BTreeSet::from_iter(iter::repeat_with(|| lfsr.next(64)).take(255));
        assert_eq!(unique.len(), 255);
    }
}
