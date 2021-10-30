
use crate::macros::p;

// polynomial types
#[p(u="u8")]    pub type p8;
#[p(u="u16")]   pub type p16;
#[p(u="u32")]   pub type p32;
#[p(u="u64")]   pub type p64;
#[p(u="u128")]  pub type p128;
#[p(u="usize")] pub type psize;


#[cfg(test)]
mod test {
    use super::*;
    use core::convert::TryFrom;

    #[test]
    fn add() {
        assert_eq!(p8(0xfe) + p8(0x87), p8(0x79));
        assert_eq!(p16(0xfedc) + p16(0x8765), p16(0x79b9));
        assert_eq!(p32(0xfedcba98) + p32(0x87654321), p32(0x79b9f9b9));
        assert_eq!(p64(0xfedcba9876543210) + p64(0x8765432100000000), p64(0x79b9f9b976543210));
        assert_eq!(p128(0xfedcba98765432100000000000000000) + p128(0x87654321000000000000000000000000), p128(0x79b9f9b9765432100000000000000000));
    }

    #[test]
    fn sub() {
        assert_eq!(p8(0xfe) - p8(0x87), p8(0x79));
        assert_eq!(p16(0xfedc) - p16(0x8765), p16(0x79b9));
        assert_eq!(p32(0xfedcba98) - p32(0x87654321), p32(0x79b9f9b9));
        assert_eq!(p64(0xfedcba9876543210) - p64(0x8765432100000000), p64(0x79b9f9b976543210));
        assert_eq!(p128(0xfedcba98765432100000000000000000) - p128(0x87654321000000000000000000000000), p128(0x79b9f9b9765432100000000000000000));
    }

    #[test]
    fn mul() {
        assert_eq!(p8(0xfe).naive_wrapping_mul(p8(0x87)), p8(0xfa));
        assert_eq!(p16(0xfedc).naive_wrapping_mul(p16(0x8765)), p16(0x7d2c));
        assert_eq!(p32(0xfedcba98).naive_wrapping_mul(p32(0x87654321)), p32(0x03da4198));
        assert_eq!(p64(0xfedcba9876543210).naive_wrapping_mul(p64(0x8765432100000000)), p64(0x0050401000000000));
        assert_eq!(p128(0xfedcba98765432100000000000000000).naive_wrapping_mul(p128(0x87654321000000000000000000000000)), p128(0x00000000000000000000000000000000));

        assert_eq!(p8(0xfe).wrapping_mul(p8(0x87)), p8(0xfa));
        assert_eq!(p16(0xfedc).wrapping_mul(p16(0x8765)), p16(0x7d2c));
        assert_eq!(p32(0xfedcba98).wrapping_mul(p32(0x87654321)), p32(0x03da4198));
        assert_eq!(p64(0xfedcba9876543210).wrapping_mul(p64(0x8765432100000000)), p64(0x0050401000000000));
        assert_eq!(p128(0xfedcba98765432100000000000000000).wrapping_mul(p128(0x87654321000000000000000000000000)), p128(0x00000000000000000000000000000000));
    }

    #[test]
    fn div() {
        assert_eq!(p8(0xfe).naive_div(p8(0x87)), p8(0x01));
        assert_eq!(p16(0xfedc).naive_div(p16(0x8765)), p16(0x0001));
        assert_eq!(p32(0xfedcba98).naive_div(p32(0x87654321)), p32(0x00000001));
        assert_eq!(p64(0xfedcba9876543210).naive_div(p64(0x8765432100000000)), p64(0x0000000000000001));
        assert_eq!(p128(0xfedcba98765432100000000000000000).naive_div(p128(0x87654321000000000000000000000000)), p128(0x000000000000000000000001));

        assert_eq!(p8(0xfe) / p8(0x87), p8(0x01));
        assert_eq!(p16(0xfedc) / p16(0x8765), p16(0x0001));
        assert_eq!(p32(0xfedcba98) / p32(0x87654321), p32(0x00000001));
        assert_eq!(p64(0xfedcba9876543210) / p64(0x8765432100000000), p64(0x0000000000000001));
        assert_eq!(p128(0xfedcba98765432100000000000000000) / p128(0x87654321000000000000000000000000), p128(0x00000000000000000000000000000001));
    }

    #[test]
    fn rem() {
        assert_eq!(p8(0xfe).naive_rem(p8(0x87)), p8(0x79));
        assert_eq!(p16(0xfedc).naive_rem(p16(0x8765)), p16(0x79b9));
        assert_eq!(p32(0xfedcba98).naive_rem(p32(0x87654321)), p32(0x79b9f9b9));
        assert_eq!(p64(0xfedcba9876543210).naive_rem(p64(0x8765432100000000)), p64(0x79b9f9b976543210));
        assert_eq!(p128(0xfedcba98765432100000000000000000).naive_rem(p128(0x87654321000000000000000000000000)), p128(0x79b9f9b9765432100000000000000000));

        assert_eq!(p8(0xfe) % p8(0x87), p8(0x79));
        assert_eq!(p16(0xfedc) % p16(0x8765), p16(0x79b9));
        assert_eq!(p32(0xfedcba98) % p32(0x87654321), p32(0x79b9f9b9));
        assert_eq!(p64(0xfedcba9876543210) % p64(0x8765432100000000), p64(0x79b9f9b976543210));
        assert_eq!(p128(0xfedcba98765432100000000000000000) % p128(0x87654321000000000000000000000000), p128(0x79b9f9b9765432100000000000000000));
    }

    #[test]
    fn naive_mul() {
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
}


