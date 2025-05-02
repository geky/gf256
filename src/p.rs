//! ## Polynomial types
//!
//! Types representing a binary polynomial.
//!
//! These types act as a building block for most of the math in gf256.
//!
//! ``` rust
//! use ::gf256::*;
//!
//! let a = p32(0x1234);
//! let b = p32(0x5678);
//! assert_eq!(a+b, p32(0x444c));
//! assert_eq!(a*b, p32(0x05c58160));
//! ```
//!
//! ## What are binary polynomials?
//!
//! Consider a binary number:
//!
//! ``` text
//! a = 0b1011
//! ```
//!
//! Normally we would view this as the binary representation of the decimal
//! number `11` (that's `11 = 1*10^1 + 1*10^0`, ok maybe a bad choice for an
//! example number).
//!
//! Instead, lets view it as a polynomial for some made-up variable `x`, where
//! each coefficient is a binary `1` or `0`:
//!
//! ``` text
//! a = 0b1011 = 1*x^3 + 0*x^2 + 1*x^1 + 1*x^0
//! ```
//!
//! We can add polynomials together, as long as we mod each coefficient by `2`
//! so they remain binary:
//!
//! ``` text
//! a   = 0b1011 = 1*x^3 + 0*x^2 + 1*x^1 + 1*x^0
//! b   = 0b1101 = 1*x^3 + 1*x^2 + 0*x^1 + 1*x^0
//!
//! a+b = ((1+1)%2)*x^3 + ((0+1)%2)*x^2 + ((1+0)%2)*x^1 + ((1+1)%2)*x^0
//!     = 0*x^3 + 1*x^2 + 1*x^1 + 0*x^0
//!     = 0b0110
//! ```
//!
//! You may recognize that this is actually xor!
//!
//! But there's more, we can also multiply polynomials together:
//!
//! ``` text
//! a   = 0b1011 = 1*x^3 + 0*x^2 + 1*x^1 + 1*x^0
//! b   = 0b1101 = 1*x^3 + 1*x^2 + 0*x^1 + 1*x^0
//!
//! a*b = 1*x^6 + 0*x^5 + 1*x^4 + 1*x^3
//!             + 1*x^5 + 0*x^4 + 1*x^3 + 1*x^2
//!                             + 1*x^3 + 0*x^2 + 1*x^1 + 1*x^0
//!     = 1*x^6 + ((0+1)%2)*x^5 + ((1+0)%2)*x^4 + ((1+1+1)%2)*x^3 + ((1+0)%2)*x^2 + 1*x^1 + 1*x^0
//!     = 1*x^6 + 1*x^5 + 1*x^4 + 1*x^3 + 1*x^2 + 1*x^1 + 1*x^0
//!     = 0b1111111
//! ```
//!
//! It's worth emphasizing that the `x` in these polynomials is a variable that we
//! never actually evaluate. We just use it to create a view of the underlying
//! binary numbers that we can do polynomial operations on.
//!
//! These operations, on bits viewed as binary polynomials, are available in gf256's
//! polynomial types:
//!
//! ``` rust
//! use ::gf256::*;
//!
//! assert_eq!(p8(0b1011) + p8(0b1101), p8(0b0110));
//! assert_eq!(p8(0b1011) * p8(0b1101), p8(0b1111111));
//! ```
//!
//! These polynomial types also provide division and remainder on binary polynomials,
//! using similar algorithms. However these are more expensive than polynomial
//! multiplication and rarely have hardware support. Still, they are quite useful
//! for certain calculations:
//!
//! ``` rust
//! use ::gf256::*;
//!
//! assert_eq!(p8(0b1111111) / p8(0b1011), p8(0b1101));
//! assert_eq!(p8(0b1111111) % p8(0b1011), p8(0b0000));
//!
//! assert_eq!(p8(0b1111110) / p8(0b1011), p8(0b1101));
//! assert_eq!(p8(0b1111110) % p8(0b1011), p8(0b0001));
//! ```
//!
//! ## Hardware support
//!
//! The polynomial types leverage [carry-less multiplication][xmul] instructions
//! when available, otherwise falling back to a more expensive, branch-less naive
//! implementation.
//!
//! Note that at the time of writing, aarch64 [`pmull`][pmull] support is only
//! available on a [nightly][nightly] compiler.
//!
//! gf256 also exposes the flag [`HAS_XMUL`], which can be used to choose
//! algorithms based on whether or not hardware accelerated carry-less
//! multiplication is available:
//!
//! ``` rust
//! # use gf256::p::p32;
//! #
//! let a = p32(0b1011);
//! let b = if gf256::HAS_XMUL {
//!     a * p32(0b11)
//! } else {
//!     (a << 1) ^ a
//! };
//! ```
//!
//! Note, there currently is no hardware support for polynomial division and
//! remainder. These are expensive, branching, loop-based implementations and
//! should generally be avoided in performance-sensitive code.
//!
//! ## `const fn` support
//!
//! Due to the use of traits and intrinsics, it's not possible to use the
//! polynomial operators in [`const fns`][const-fn].
//!
//! As an alternative, the polynomial types preovide a set of "naive"
//! functions, which provide less efficient, well, naive, implementations,
//! that can be used in const fns.
//!
//! These are very useful for calculating complex constants at compile-time:
//!
//! ``` rust
//! # use ::gf256::*;
//! #
//! const POLYNOMIAL: p64 = p64(0x104c11db7);
//! const CRC_TABLE: [u32; 256] = {
//!     let mut table = [0; 256];
//!     let mut i = 0;
//!     while i < table.len() {
//!         let x = (i as u32).reverse_bits();
//!         let x = p64((x as u64) << 8).naive_rem(POLYNOMIAL).0 as u32;
//!         table[i] = x.reverse_bits();
//!         i += 1;
//!     }
//!     table
//! };
//! ```
//!
//! ## Constant-time
//!
//! gf256 provides "best-effort" constant-time implementations for certain
//! useful operations.
//!
//! For polynomial types, addition (xor), subtraction (xor), and multiplication
//! should always be constant-time.
//!
//! Note that division and remainder are NOT constant-time. These are expensive,
//! branching, loop-based implementations, which should generally be avoided for
//! performance reasons anyway (outside of constant generation).
//!
//!
//! [xmul]: https://en.wikipedia.org/wiki/Carry-less_product
//! [xor]: https://en.wikipedia.org/wiki/Bitwise_operation#XOR
//! [pclmulqdq]: https://www.felixcloutier.com/x86/pclmulqdq
//! [pmull]: https://developer.arm.com/documentation/ddi0596/2021-06/SIMD-FP-Instructions/PMULL--PMULL2--Polynomial-Multiply-Long-
//! [nightly]: https://doc.rust-lang.org/book/appendix-07-nightly-rust.html
//! [const-fn]: https://doc.rust-lang.org/reference/const_eval.html


/// A macro for generating custom polynomial types.
///
/// ``` rust,ignore
/// # use ::gf256::*;
/// # use ::gf256::p::p;
/// #[p(u=u32)] pub type my_p32;
///
/// # fn main() {
/// let a = my_p32(0x1234);
/// let b = my_p32(0x5678);
/// assert_eq!(a+b, my_p32(0x444c));
/// assert_eq!(a*b, my_p32(0x05c58160));
/// # }
/// ```
///
/// The `p` macro accepts a number of configuration options:
///
///
/// - `width` - Width of the polynomial type in bits, defaults to the
///   width of the `u` type.
/// - `usize` - Indicate if the width is dependent on the usize width,
///   defaults to true if the `u` type is `usize`.
/// - `u` - The underlying unsigned type.
/// - `i` - The underlying signed type, defaults to the signed version
///   of the `u` type.
/// - `naive` - Use a naive bitwise implementation.
/// - `xmul` - Optionally provide a custom implementation of polynomial
///   multiplication.
///
/// ``` rust
/// # use ::gf256::*;
/// # use ::gf256::p::p;
/// fn custom_xmul(a: u32, b: u32) -> (u32, u32) {
///     let (lo, hi) = p32(a).widening_mul(p32(b));
///     (u32::from(lo), u32::from(hi))
/// }
///
/// #[p(
///     width=32,
///     usize=false,
///     u=u32,
///     i=i32,
///     // naive,
///     xmul=custom_xmul,
/// )]
/// type my_p32;
///
/// # fn main() {
/// let a = my_p32(0x1234);
/// let b = my_p32(0x5678);
/// assert_eq!(a+b, my_p32(0x444c));
/// assert_eq!(a*b, my_p32(0x05c58160));
/// # }
/// ```
///
pub use gf256_macros::p;

// polynomial types
#[p(u=u8)]    pub type p8;
#[p(u=u16)]   pub type p16;
#[p(u=u32)]   pub type p32;
#[p(u=u64)]   pub type p64;
#[p(u=u128)]  pub type p128;
#[p(u=usize)] pub type psize;


#[cfg(test)]
mod test {
    use super::*;
    use core::convert::TryFrom;

    #[test]
    fn add() {
        assert_eq!(p8(0x12).naive_add(p8(0x34)), p8(0x26));
        assert_eq!(p16(0x1234).naive_add(p16(0x5678)), p16(0x444c));
        assert_eq!(p32(0x12345678).naive_add(p32(0x9abcdef1)), p32(0x88888889));
        assert_eq!(p64(0x123456789abcdef1).naive_add(p64(0x23456789abcdef12)), p64(0x317131f1317131e3));
        assert_eq!(p128(0x123456789abcdef123456789abcdef12).naive_add(p128(0x3456789abcdef123456789abcdef1234)), p128(0x26622ee226622fd26622ee226622fd26));

        assert_eq!(p8(0x12) + p8(0x34), p8(0x26));
        assert_eq!(p16(0x1234) + p16(0x5678), p16(0x444c));
        assert_eq!(p32(0x12345678) + p32(0x9abcdef1), p32(0x88888889));
        assert_eq!(p64(0x123456789abcdef1) + p64(0x23456789abcdef12), p64(0x317131f1317131e3));
        assert_eq!(p128(0x123456789abcdef123456789abcdef12) + p128(0x3456789abcdef123456789abcdef1234), p128(0x26622ee226622fd26622ee226622fd26));
    }

    #[test]
    fn sub() {
        assert_eq!(p8(0x12).naive_sub(p8(0x34)), p8(0x26));
        assert_eq!(p16(0x1234).naive_sub(p16(0x5678)), p16(0x444c));
        assert_eq!(p32(0x12345678).naive_sub(p32(0x9abcdef1)), p32(0x88888889));
        assert_eq!(p64(0x123456789abcdef1).naive_sub(p64(0x23456789abcdef12)), p64(0x317131f1317131e3));
        assert_eq!(p128(0x123456789abcdef123456789abcdef12).naive_sub(p128(0x3456789abcdef123456789abcdef1234)), p128(0x26622ee226622fd26622ee226622fd26));

        assert_eq!(p8(0x12) - p8(0x34), p8(0x26));
        assert_eq!(p16(0x1234) - p16(0x5678), p16(0x444c));
        assert_eq!(p32(0x12345678) - p32(0x9abcdef1), p32(0x88888889));
        assert_eq!(p64(0x123456789abcdef1) - p64(0x23456789abcdef12), p64(0x317131f1317131e3));
        assert_eq!(p128(0x123456789abcdef123456789abcdef12) - p128(0x3456789abcdef123456789abcdef1234), p128(0x26622ee226622fd26622ee226622fd26));
    }

    #[test]
    fn mul() {
        assert_eq!(p8(0x12).naive_wrapping_mul(p8(0x3)), p8(0x36));
        assert_eq!(p16(0x123).naive_wrapping_mul(p16(0x45)), p16(0x4d6f));
        assert_eq!(p32(0x12345).naive_wrapping_mul(p32(0x6789)), p32(0x6bc8a0ed));
        assert_eq!(p64(0x123456789).naive_wrapping_mul(p64(0xabcdef12)), p64(0xbf60cfc95524a082));
        assert_eq!(p128(0x123456789abcdef12).naive_wrapping_mul(p128(0x3456789abcdef123)), p128(0x328db698aa112b13219aad8fb9062176));

        assert_eq!(p8(0x12) * p8(0x3), p8(0x36));
        assert_eq!(p16(0x123) * p16(0x45), p16(0x4d6f));
        assert_eq!(p32(0x12345) * p32(0x6789), p32(0x6bc8a0ed));
        assert_eq!(p64(0x123456789) * p64(0xabcdef12), p64(0xbf60cfc95524a082));
        assert_eq!(p128(0x123456789abcdef12) * p128(0x3456789abcdef123), p128(0x328db698aa112b13219aad8fb9062176));
    }

    #[test]
    fn div() {
        assert_eq!(p8(0x36).naive_div(p8(0x12)), p8(0x3));
        assert_eq!(p16(0x4d6f).naive_div(p16(0x123)), p16(0x45));
        assert_eq!(p32(0x6bc8a0ed).naive_div(p32(0x12345)), p32(0x6789));
        assert_eq!(p64(0xbf60cfc95524a082).naive_div(p64(0x123456789)), p64(0xabcdef12));
        assert_eq!(p128(0x328db698aa112b13219aad8fb9062176).naive_div(p128(0x123456789abcdef12)), p128(0x3456789abcdef123));

        assert_eq!(p8(0x36) / p8(0x12), p8(0x3));
        assert_eq!(p16(0x4d6f) / p16(0x123), p16(0x45));
        assert_eq!(p32(0x6bc8a0ed) / p32(0x12345), p32(0x6789));
        assert_eq!(p64(0xbf60cfc95524a082) / p64(0x123456789), p64(0xabcdef12));
        assert_eq!(p128(0x328db698aa112b13219aad8fb9062176) / p128(0x123456789abcdef12), p128(0x3456789abcdef123));
    }

    #[test]
    fn rem() {
        assert_eq!(p8(0x37).naive_rem(p8(0x12)), p8(0x1));
        assert_eq!(p16(0x4d6e).naive_rem(p16(0x123)), p16(0x1));
        assert_eq!(p32(0x6bc8a0ec).naive_rem(p32(0x12345)), p32(0x1));
        assert_eq!(p64(0xbf60cfc95524a083).naive_rem(p64(0x123456789)), p64(0x1));
        assert_eq!(p128(0x328db698aa112b13219aad8fb9062177).naive_rem(p128(0x123456789abcdef12)), p128(0x1));

        assert_eq!(p8(0x37) % p8(0x12), p8(0x1));
        assert_eq!(p16(0x4d6e) % p16(0x123), p16(0x1));
        assert_eq!(p32(0x6bc8a0ec) % p32(0x12345), p32(0x1));
        assert_eq!(p64(0xbf60cfc95524a083) % p64(0x123456789), p64(0x1));
        assert_eq!(p128(0x328db698aa112b13219aad8fb9062177) % p128(0x123456789abcdef12), p128(0x1));
    }

    #[test]
    fn all_mul() {
        for a in (0..=255).map(p8) {
            for b in (0..=255).map(p8) {
                let naive_res = a.naive_wrapping_mul(b);
                let res_xmul = a.wrapping_mul(b);
                assert_eq!(naive_res, res_xmul);
            }
        }
    }

    #[test]
    fn overflowing_mul() {
        for a in (0..=255).map(p8) {
            for b in (0..=255).map(p8) {
                let (naive_wrapped, naive_overflow) = a.naive_overflowing_mul(b);
                let (wrapped_xmul, overflow_xmul) = a.overflowing_mul(b);
                let naive_res = p16::from(a).naive_mul(p16::from(b));
                let res_xmul = p16::from(a) * p16::from(b);

                // same results naive vs xmul?
                assert_eq!(naive_wrapped, wrapped_xmul);
                assert_eq!(naive_overflow, overflow_xmul);
                assert_eq!(naive_res, res_xmul);

                // same wrapped results?
                assert_eq!(naive_wrapped, p8::try_from(naive_res & 0xff).unwrap());
                assert_eq!(wrapped_xmul, p8::try_from(res_xmul & 0xff).unwrap());

                // overflow set if overflow occured?
                assert_eq!(naive_overflow, (p16::from(naive_wrapped) != naive_res));
                assert_eq!(overflow_xmul, (p16::from(wrapped_xmul) != res_xmul));
            }
        }
    }

    #[test]
    fn widening_mul() {
        for a in (0..=255).map(p8) {
            for b in (0..=255).map(p8) {
                let (naive_lo, naive_hi) = a.naive_widening_mul(b);
                let (lo_xmul, hi_xmul ) = a.widening_mul(b);
                let naive_res = p16::from(a).naive_mul(p16::from(b));
                let res_xmul = p16::from(a) * p16::from(b);

                // same results naive vs xmul?
                assert_eq!(naive_lo, lo_xmul);
                assert_eq!(naive_hi, hi_xmul);
                assert_eq!(naive_res, res_xmul);

                // same lo results?
                assert_eq!(naive_lo, p8::try_from(naive_res & 0xff).unwrap());
                assert_eq!(lo_xmul, p8::try_from(res_xmul & 0xff).unwrap());

                // same hi results?
                assert_eq!(naive_hi, p8::try_from(naive_res >> 8).unwrap());
                assert_eq!(hi_xmul, p8::try_from(res_xmul >> 8).unwrap());
            }
        }
    }

    #[test]
    fn mul_div() {
        for a in (1..=255).map(p16) {
            for b in (1..=255).map(p16) {
                // mul
                let x = a * b;
                // is div/rem correct?
                assert_eq!(x / a, b);
                assert_eq!(x % a, p16(0));
                assert_eq!(x / b, a);
                assert_eq!(x % b, p16(0));
            }   
        }
    }

    #[test]
    fn div_rem() {
        for a in (0..=255).map(p8) {
            for b in (1..=255).map(p8) {
                // find div + rem
                let q = a / b;
                let r = a % b;
                // mul and add to find original
                let x = q*b + r;
                assert_eq!(x, a);
            }   
        }
    }

    #[test]
    fn pow() {
        // p32::naive_pow just uses p32::naive_mul, we want
        // to test with a truely naive pow
        fn naive_pow(a: p64, exp: u32) -> p64 {
            let mut x = p64(1);
            for _ in 0..exp {
                x = x.wrapping_mul(a);
            }
            x
        }

        for a in (0..=255).map(p64) {
            for b in 0..=255 {
                // compare pow vs naive_pow
                assert_eq!(a.wrapping_pow(b), naive_pow(a, b));
            }   
        }
    }

    // all polynomial-type params
    #[p(
        width=8,
        usize=false,
        u=u8,
        i=i8,
        xmul=custom_xmul,
    )]
    type p8_all_params;

    fn custom_xmul(a: u8, b: u8) -> (u8, u8) {
        let (lo, hi) = p8(a).widening_mul(p8(b));
        (u8::from(lo), u8::from(hi))
    }

    #[test]
    fn p_all_params() {
        for a in (0..=255).map(p8_all_params) {
            for b in (0..=255).map(p8_all_params) {
                let naive_res = a.naive_wrapping_mul(b);
                let res_xmul = a.wrapping_mul(b);
                assert_eq!(naive_res, res_xmul);
            }
        }
    }
}


