
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
}


