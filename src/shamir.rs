//! ## Shamir's secret-sharing scheme
//!
//! [Shamir's secret-sharing scheme][shamir-wiki] is an algorithm for splitting a
//! secret into some number of shares `n`, such that you need at minimum some number
//! of shares `k` to reconstruct the original secret.
//!
//! ``` rust
//! use gf256::shamir::shamir;
//! 
//! // generate shares
//! let shares = shamir::generate(b"secret secret secret!", 5, 4);
//! 
//! // <4 can't reconstruct secret
//! assert_ne!(shamir::reconstruct(&shares[..1]), b"secret secret secret!");
//! assert_ne!(shamir::reconstruct(&shares[..2]), b"secret secret secret!");
//! assert_ne!(shamir::reconstruct(&shares[..3]), b"secret secret secret!");
//! 
//! // >=4 can reconstruct secret
//! assert_eq!(shamir::reconstruct(&shares[..4]), b"secret secret secret!");
//! assert_eq!(shamir::reconstruct(&shares[..5]), b"secret secret secret!");
//! ```
//!
//! Note this module requires feature `shamir`. You may also want to enable the
//! feature `thread-rng`, which is required for the default rng.
//!
//! A fully featured implementation of Shamir's secret sharing can be found in
//! [`examples/shamir.rs`][shamir-example]:
//!
//! ``` bash
//! $ RUSTFLAGS="-Ctarget-cpu=native" cargo run --features thread-rng,lfsr,crc,shamir,raid,rs --example shamir
//!
//! testing shamir("Hello World!")
//! generate share1 => .....uT4.z.O.  019ddb829d755434f77ae84ffd
//! generate share2 => .Y4..Nq......  025934c8a74e711c05f4aeb7d0
//! generate share3 => ...H7.....?v.  03c2be4837b908b4ad113f7607
//! generate share4 => .y......]..x.  0479b0bbf09cb8e85de497780f
//! generate share5 => ...,Z...e.m..  0515b62c5ad2e20b65b86d15e9
//! reconstruct 1 shares => ....uT4.z.O.  9ddb829d755434f77ae84ffd
//! reconstruct 2 shares => *uO...,R.!..  2a754f8b97bc2c520021ece6
//! reconstruct 3 shares => .Q...-._.y.*  0651020d822d9c5f9f798e2a
//! reconstruct 4 shares => Hello World!  48656c6c6f20576f726c6421
//! reconstruct 5 shares => Hello World!  48656c6c6f20576f726c6421
//! ```
//!
//! ## How does Shamir's secret sharing scheme work?
//!
//! The underlying theory of Shamir's secret sharing is actually relatively easy
//! to visualize.
//!
//! Consider some 2-degree polynomial:
//!
//! ``` text
//!                 .
//!                 . ......
//!                ..'      ''.
//!               ' .          '.
//!             .'  .            '
//!            .    .             '.
//!.  . . . . : . . . . . . . . . . .
//!          '      .
//!         '       .
//!        '        .
//!       '         .
//!      .          .
//!                 .
//! ```
//!
//! Because our polynomial is 2-degree, we need at minimum 3 points to uniquely
//! define the polynomial. If we only have 2 points:
//!
//! ``` text
//!                 .
//!                 .
//!                 .
//!                 .           o
//!             o   .           
//!                 .
//! . . . . . . . . . . . . . . . . .
//!                 .
//!                 .
//!                 .
//!                 .
//!                 .
//!                 .
//! ```
//!
//! There are any number of polynomials that intersect these 2 points! With only
//! 2 points, it's impossible to figure out the original polynomial.
//!
//! ``` text
//!    '.           .
//!      .    '     . ......     .  .
//!       '.       ..'      ''.   .'
//!         '. '  ' .          'o'
//!           ''o'  .        ..' '
//!            . '''......'''  '  '.
//! . . . . . : .'. . . . . . . . . .
//!          '    ' .        .
//!         '      ..       .
//!        '        :      .
//!       '         .'....'
//!      .          .
//!                 .
//! ```
//!
//! But with 3 points, there is only one 2-degree polynomial that hits all three:
//!
//! ``` text
//!                 .
//!                 . .o....
//!                ..'      ''.
//!               ' .          'o
//!             o'  .            '
//!            .    .             '.
//!.  . . . . : . . . . . . . . . . .
//!          '      .
//!         '       .
//!        '        .
//!       '         .
//!      .          .
//!                 .
//! ```
//!
//! We can store a secret value on a polynomial by creating a polynomial where
//! the intersection at some arbitrary x-coordinate give us our secret value.
//! Choosing the arbitrary coordinate `x=0` is convenient because creating the
//! secret polynomial is as easy as choosing random values for the non-constant
//! coefficients. Say we wanted to store the secret value [4][xkcd-4]:
//!
//! ``` text
//! f(x) = 4 + 32x + 12x^2
//!        ^   \----+----/
//!        |        '-- random coefficients
//!        '----------- our secret value
//! ```
//!
//! We can then create any number of shares by evaluating the secret polynomial
//! at arbitrary coordinates (except zero!):
//!
//! ``` rust
//! // our random polynomial
//! let f = |x: f64| { 4.0 + 32.0*x + 12.0*x.powf(2.0) };
//!
//! // generate 4 shares
//! assert_eq!(f(1.0), 48.0);
//! assert_eq!(f(2.0), 116.0);
//! assert_eq!(f(3.0), 208.0);
//! assert_eq!(f(4.0), 324.0);
//! ```
//!
//! So our shares would be (`x=1`, `y=48`), (`x=2`, `y=116`), (`x=3`, `y=208`),
//! and (`x=4`, `y=324`). 
//!
//! Since we used a 2-degree polynomial, we need at minimum any 3 of the shares to
//! find the original polynomial. Any fewer and finding the original polynomial
//! would be impossible. If we wanted a different threshold, say `k` shares, we would
//! just need to use a `k-1` degree polynomial.
//!
//! If we have at least 3 of the shares, we can find the original secret using
//! a technique called [Lagrange interpolation][lagrange-interpolation] (Wikipedia
//! will going to do a better job of explaining the math than I can):
//!
//! ``` rust
//! // we need >= 3 shares
//! let shares = [
//!     (1.0, 48.0),
//!     (2.0, 116.0),
//!     (4.0, 324.0)
//! ];
//!
//! // find f(0) using Lagrange interpolation
//! let mut y = 0.0;
//! for (i, (x0, y0)) in shares.iter().enumerate() {
//!     let mut li = 1.0;
//!     for (j, (x1, _y1)) in shares.iter().enumerate() {
//!         if i != j {
//!             li *= x1 / (x1-x0);
//!         }
//!     }
//!
//!     y += li*y0;
//! }
//!
//! // y should now equal our secret value!
//! assert_eq!(y, 4.0);
//! ```
//!
//! That sure is great, but using floats everywhere sure is annoying. Fortunately
//! this math works perfectly fine in finite field!
//!
//! ``` rust
//! # use ::gf256::*;
//! #
//! // our random polynomial
//! let f = |x: gf256| { gf256(4) + gf256(32)*x + gf256(12)*x.pow(2) };
//!
//! // generate 4 shares
//! assert_eq!(f(gf256(1)), gf256(40));
//! assert_eq!(f(gf256(2)), gf256(116));
//! assert_eq!(f(gf256(3)), gf256(88));
//! assert_eq!(f(gf256(4)), gf256(68));
//! ```
//!
//! ``` rust
//! # use ::gf256::*;
//! #
//! // we need >= 3 shares
//! let shares = [
//!     (gf256(1), gf256(40)),
//!     (gf256(2), gf256(116)),
//!     (gf256(4), gf256(68)),
//! ];
//!
//! // find f(0) using Lagrange interpolation
//! let mut y = gf256(0);
//! for (i, (x0, y0)) in shares.iter().enumerate() {
//!     let mut li = gf256(1);
//!     for (j, (x1, _y1)) in shares.iter().enumerate() {
//!         if i != j {
//!             li *= x1 / (x1-x0);
//!         }
//!     }
//!
//!     y += li*y0;
//! }
//!
//! // y should now equal our secret value!
//! assert_eq!(y, gf256(4));
//! ```
//!
//! And this is how our Shamir's secret sharing scheme works:
//!
//! ``` rust
//! use gf256::shamir::shamir;
//!
//! let shares = [[1, 40], [2, 116], [4, 68]];
//! assert_eq!(shamir::reconstruct(&shares), &[4]);
//! ```
//!
//! Of course, we usually want to distribute secrets that are more
//! than a byte large. We can expand this scheme to any number of bytes
//! by choosing a different random polynomial for each byte in the secret
//! value. Though we can at least share the same x-coordinate for all generated
//! points in a given share.
//!
//! ``` rust
//! # // using a fixed-rng (bad!) so this example is reproducible/testable
//! # pub use ::gf256::*; // these imports are a hack around doctest namespacing issues
//! # pub use ::gf256::gf;
//! # #[::gf256::shamir::shamir(rng=gf256::lfsr::Lfsr64::new(0x123456789abcdef1))]
//! # mod shamir {}
//! #
//! # fn main() {
//! // generate shares
//! let shares = shamir::generate(b"secret secret secret!", 4, 3);
//!
//! fn hex(xs: &[u8]) -> String {
//!     xs.iter()
//!         .map(|x| format!("{:02x}", x))
//!         .collect()
//! }
//!
//! assert_eq!(hex(&shares[0]), "01fb3cdc338aed9bc436218f52788f5768e1d282042a");
//! assert_eq!(hex(&shares[1]), "0264be77c1902132faa6661c7c7f9c8b00ec15d89fd7");
//! assert_eq!(hex(&shares[2]), "03ece7c8807fb8894df524e14b7333af0d6eb53fefdc");
//! assert_eq!(hex(&shares[3]), "0435778acd4a2bfdb37757b0962e9e644e0254a79377");
//! //                            ^\-------------------+--------------------/
//! //                            |                    |
//! //                 arbitrary x-coordinate    y-coordinates
//!
//! // reconstruct our secret
//! assert_eq!(shamir::reconstruct(&shares), b"secret secret secret!");
//! # }
//! ```
//!
//! Note that using a different polynomial for each byte is quite important.
//! Shamir's secret sharing scheme is a generalization of a [one-time pad][one-time-pad],
//! and sharing a polynomial for all bytes reduces the one-time pad into a simple
//! substitution cipher, opening the scheme up to attacks.
//!
//! ## Limitations
//!
//! It may be a surprise, but it turns out that finite-fields are finite. This means
//! there are only a finite number of elements to choose from when choosing our
//! the arbitrary x-coordinates for our shares.
//!
//! Because of this, Shamir's secret sharing scheme is limited to the number of non-zero
//! elements in our field. In the case of `GF(256)`, this limits us to 255 shares.
//!
//! ## Constant-time
//!
//! The default Shamir's secret-sharing implementation internally uses a custom
//! Galois-field type in `barret` mode and should be constant-time.
//!
//! ## Security notes
//!
//! It's worth emphasizing that the gf256 was implemented primarily as an
//! educational project. I would not suggest using this library for security-related
//! applications without first evaluating externally. You use this library at your
//! own risk.
//!
//!
//! [shamir-wiki]: https://en.wikipedia.org/wiki/Shamir%27s_Secret_Sharing
//! [xkcd-4]: https://xkcd.com/221/
//! [lagrange-interpolation]: https://en.wikipedia.org/wiki/Lagrange_polynomial
//! [one-time-pad]: https://en.wikipedia.org/wiki/One-time_pad
//! [shamir-example]: https://github.com/geky/gf256/blob/master/examples/shamir.rs


/// A macro for generating custom Shamir secret-sharing modules.
///
/// ``` rust,ignore
/// # use ::gf256::*;
/// # use ::gf256::shamir::shamir;
/// #[shamir]
/// pub mod my_shamir {}
///
/// # fn main() {
/// // generate shares                                                      
/// let shares = my_shamir::generate(b"secret secret secret!", 5, 4);          
///                                                                         
/// // <4 can't reconstruct secret                                          
/// assert_ne!(my_shamir::reconstruct(&shares[..1]), b"secret secret secret!");
/// assert_ne!(my_shamir::reconstruct(&shares[..2]), b"secret secret secret!");
/// assert_ne!(my_shamir::reconstruct(&shares[..3]), b"secret secret secret!");
///                                                                         
/// // >=4 can reconstruct secret                                           
/// assert_eq!(my_shamir::reconstruct(&shares[..4]), b"secret secret secret!");
/// assert_eq!(my_shamir::reconstruct(&shares[..5]), b"secret secret secret!");
/// # }
/// ```
///
/// The `shamir` macro accepts a number of configuration options:
///
/// - `gf` - The finite-field we are implemented over, defaults to
///   [`gf256`](crate::gf256) in Barret mode.
/// - `u` - The unsigned type to operate on, defaults to [`u8`].
/// - `rng` - The random-number generator to use for generating shares, defaults
///   to [`ThreadRng`][thread-rng].
///
/// ``` rust,ignore
/// # use ::gf256::*;
/// # use ::gf256::shamir::shamir;
/// use rand::rngs::ThreadRng;
///
/// #[shamir(
///     gf=gf256,
///     u=u8,
///     rng=ThreadRng::default(),
/// )]
/// pub mod my_shamir {}
///
/// # fn main() {
/// // generate shares                                                      
/// let shares = my_shamir::generate(b"secret secret secret!", 5, 4);          
///                                                                         
/// // <4 can't reconstruct secret                                          
/// assert_ne!(my_shamir::reconstruct(&shares[..1]), b"secret secret secret!");
/// assert_ne!(my_shamir::reconstruct(&shares[..2]), b"secret secret secret!");
/// assert_ne!(my_shamir::reconstruct(&shares[..3]), b"secret secret secret!");
///                                                                         
/// // >=4 can reconstruct secret                                           
/// assert_eq!(my_shamir::reconstruct(&shares[..4]), b"secret secret secret!");
/// assert_eq!(my_shamir::reconstruct(&shares[..5]), b"secret secret secret!");
/// # }
/// ```
///
/// [thread-rng]: https://docs.rs/rand/latest/rand/rngs/struct.ThreadRng.html
///
pub use gf256_macros::shamir;


// Shamir secret-sharing functions
//
// Note we can only provide a default if we have ThreadRng available,
// otherwise we can only provide the shamir macro which accepts a
// custom Rng type
//
#[cfg(feature="thread-rng")]
#[shamir]
pub mod shamir {}


#[cfg(test)]
mod test {
    use super::shamir as gf256_shamir;
    use super::*;
    use crate::gf::*;
    use rand::rngs::ThreadRng;
    use core::convert::TryFrom;

    extern crate alloc;
    use alloc::vec::Vec;

    #[cfg(feature="thread-rng")]
    #[test]
    fn shamir5w4() {
        let input = b"Hello World!";
        let shares = gf256_shamir::generate(input, 5, 4);
        assert_eq!(shares.len(), 5);
        for i in 0..5 {
            let output = gf256_shamir::reconstruct(&shares[..i]);
            if i < 4 {
                assert_ne!(output, input);
            } else {
                assert_eq!(output, input);
            }
        }
    }

    #[cfg(feature="thread-rng")]
    #[test]
    fn shamir255w100() {
        let input = b"Hello World!";
        let shares = gf256_shamir::generate(input, 255, 100);
        assert_eq!(shares.len(), 255);
        for i in (0..255).step_by(51) {
            let output = gf256_shamir::reconstruct(&shares[..i]);
            if i < 100 {
                assert_ne!(output, input);
            } else {
                assert_eq!(output, input);
            }
        }
    }

    // multi-byte Shamir secrets
    #[cfg(feature="thread-rng")]
    #[shamir(gf=gf2p64, u=u64)]
    mod gf2p64_shamir {}

    #[cfg(feature="thread-rng")]
    #[test]
    fn gf2p64_shamir300w100() {
        let input = b"Hello World!\0\0\0\0"
            .chunks(8)
            .map(|chunk| u64::from_le_bytes(<_>::try_from(chunk).unwrap()))
            .collect::<Vec<_>>();
        let shares = gf2p64_shamir::generate(&input, 300, 100);
        assert_eq!(shares.len(), 300);
        for i in (0..300).step_by(50) {
            let output = gf2p64_shamir::reconstruct(&shares[..i]);
            if i < 100 {
                assert_ne!(output, input);
            } else {
                assert_eq!(output, input);
            }
        }
    }

    // Shamir with very odd sizes
    #[cfg(feature="thread-rng")]
    #[gf(polynomial=0x13, generator=0x2)]
    type gf16;
    #[cfg(feature="thread-rng")]
    #[shamir(gf=gf16, u=u8)]
    mod gf16_shamir {}

    #[cfg(feature="thread-rng")]
    #[gf(polynomial=0x800021, generator=0x2)]
    type gf2p23;
    #[cfg(feature="thread-rng")]
    #[shamir(gf=gf2p23, u=u32)]
    mod gf2p23_shamir {}

    #[cfg(feature="thread-rng")]
    #[test]
    fn gf16_shamir15w10() {
        let input = b"Hello World!"
            .iter()
            .map(|b| [(b >> 0) & 0xf, (b >> 4) & 0xf])
            .flatten()
            .collect::<Vec<_>>();
        let shares = gf16_shamir::generate(&input, 15, 10);
        assert_eq!(shares.len(), 15);
        for i in 0..15 {
            let output = gf16_shamir::reconstruct(&shares[..i]);
            if i < 10 {
                assert_ne!(output, input);
            } else {
                assert_eq!(output, input);
            }
        }
    }

    #[cfg(feature="thread-rng")]
    #[test]
    fn gf2p23_shamir300w100() {
        let input = b"Hello World!"
            .chunks(2)
            .map(|chunk| u32::from(u16::from_le_bytes(<_>::try_from(chunk).unwrap())))
            .collect::<Vec<_>>();
        let shares = gf2p23_shamir::generate(&input, 300, 100);
        assert_eq!(shares.len(), 300);
        for i in (0..300).step_by(50) {
            let output = gf2p23_shamir::reconstruct(&shares[..i]);
            if i < 100 {
                assert_ne!(output, input);
            } else {
                assert_eq!(output, input);
            }
        }
    }

    // TODO test this without ThreadRng?

    // all Shamir parameters 
    #[shamir(gf=gf256, u=u8, rng=ThreadRng::default())]
    mod shamir_all_params {}

    #[test]
    fn shamir_all_params() {
        let input = b"Hello World!";
        let shares = shamir_all_params::generate(input, 255, 100);
        assert_eq!(shares.len(), 255);
        for i in (0..255).step_by(10) {
            let output = shamir_all_params::reconstruct(&shares[..i]);
            if i < 100 {
                assert_ne!(output, input);
            } else {
                assert_eq!(output, input);
            }
        }
    }
}
