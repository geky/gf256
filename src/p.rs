

use gf256_macros::p;

#[p(u="u8")]   type p8;
#[p(u="u16")]  type p16;
#[p(u="u32")]  type p32;
#[p(u="u64")]  type p64;
#[p(u="u128")] type p128;


#[cfg(test)]
mod test {
    use super::*;

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
        assert_eq!(p8(0xfe).naive_mul(p8(0x87)), p8(0xfa));
        assert_eq!(p16(0xfedc).naive_mul(p16(0x8765)), p16(0x7d2c));
        assert_eq!(p32(0xfedcba98).naive_mul(p32(0x87654321)), p32(0x03da4198));
        assert_eq!(p64(0xfedcba9876543210).naive_mul(p64(0x8765432100000000)), p64(0x0050401000000000));
        assert_eq!(p128(0xfedcba98765432100000000000000000).naive_mul(p128(0x87654321000000000000000000000000)), p128(0x00000000000000000000000000000000));

        assert_eq!(p8(0xfe) * p8(0x87), p8(0xfa));
        assert_eq!(p16(0xfedc) * p16(0x8765), p16(0x7d2c));
        assert_eq!(p32(0xfedcba98) * p32(0x87654321), p32(0x03da4198));
        assert_eq!(p64(0xfedcba9876543210) * p64(0x8765432100000000), p64(0x0050401000000000));
        assert_eq!(p128(0xfedcba98765432100000000000000000) * p128(0x87654321000000000000000000000000), p128(0x00000000000000000000000000000000));
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
    fn hardware_xmul() {
        for a in 0..=255 {
            for b in 0..=255 {
                let res_naive = p8(a).naive_mul(p8(b));
                let res_hardware = p8(a) * p8(b);
                assert_eq!(res_naive, res_hardware);
            }
        }
    }
}



//p!(p=p8, u=u8);



//
//
//#[allow(non_camel_case_types)]
//#[derive(Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
//pub struct p8(pub u8);
//
//impl p8 {
//    /// Polynomial addition, aka xor
//    #[inline]
//    const fn add(self: p8, other: p8) -> p8 {
//        p8(self.0 ^ other.0)
//    }
//
//    /// Polynomial subtraction, aka xor
//    #[inline]
//    const fn sub(self: p8, other: p8) -> p8 {
//        // polynomial subtraction is just xor
//        p8(self.0 ^ other.0)
//    }
//
//    /// Polynomial multiplication
//    ///
//    /// This attempts to use carry-less multiplication
//    /// instructions when available (pclmulqdq on x86_64,
//    /// pmul on aarch64), otherwise falls back to an expensive
//    /// manual implementation
//    ///
//    #[inline]
//    const fn mul(self: p8, other: p8) -> p8 {
//        p8(xmul8(self.0, other.0) as u8)
//    }
//
//    /// Polynomial division
//    ///
//    /// Note, this is expensive. There isn't really any hardware for polynomial
//    /// division, so we need to always use an expensive manual implementation
//    ///
//    #[inline]
//    const fn checked_div(self: p8, other: p8) -> Option<p8> {
//        match xdivmod8(self.0, other.0) {
//            Some((x, _)) => Some(p8(x)),
//            _ => None,
//        }
//    }
//
//    /// Polynomial remainder
//    ///
//    /// Note, this is expensive. There isn't really any hardware for polynomial
//    /// division, so we need to always use an expensive manual implementation
//    ///
//    #[inline]
//    const fn checked_rem(self: p8, other: p8) -> Option<p8> {
//        match xdivmod8(self.0, other.0) {
//            Some((_, x)) => Some(p8(x)),
//            _ => None,
//        }
//    }
//
//    /// Polynomial division
//    ///
//    /// Note, this is expensive. There isn't really any hardware for polynomial
//    /// division, so we need to always use an expensive manual implementation
//    ///
//    /// This will also panic if b == 0
//    ///
//    #[inline]
//    fn div(self: p8, other: p8) -> p8 {
//        p8::checked_div(self, other).unwrap()
//    }
//
//    /// Polynomial remainder
//    ///
//    /// Note, this is expensive. There isn't really any hardware for polynomial
//    /// division, so we need to always use an expensive manual implementation
//    ///
//    /// This will also panic if b == 0
//    ///
//    #[inline]
//    fn rem(self: p8, other: p8) -> p8 {
//        p8::checked_rem(self, other).unwrap()
//    }
//}
//
//
////// Conversions into p8 ////
//
//impl From<u8> for p8 {
//    #[inline]
//    fn from(x: u8) -> p8 {
//        p8(x)
//    }
//}
//
////// Conversions from p8 ////
//
//impl From<p8> for u8 {
//    #[inline]
//    fn from(x: p8) -> u8 {
//        x.0
//    }
//}
//
////// Addition ////
//
//impl Add<p8> for p8 {
//    type Output = p8;
//
//    /// Polynomial addition, aka xor
//    #[inline]
//    fn add(self: p8, other: p8) -> p8 {
//        p8::add(self, other)
//    }
//}
//
//impl<'a> Add<p8> for &'a p8 {
//    type Output = p8;
//
//    /// Polynomial addition, aka xor
//    #[inline]
//    fn add(self: &'a p8, other: p8) -> p8 {
//        p8::add(*self, other)
//    }
//}
//
//impl<'b> Add<&'b p8> for p8 {
//    type Output = p8;
//
//    /// Polynomial addition, aka xor
//    #[inline]
//    fn add(self: p8, other: &'b p8) -> p8 {
//        p8::add(self, *other)
//    }
//}
//
//impl<'a, 'b> Add<&'b p8> for &'a p8 {
//    type Output = p8;
//
//    /// Polynomial addition, aka xor
//    #[inline]
//    fn add(self: &'a p8, other: &'b p8) -> p8 {
//        p8::add(*self, *other)
//    }
//}
//
//impl AddAssign<p8> for p8 {
//    #[inline]
//    fn add_assign(&mut self, other: p8) {
//        *self = self.add(other)
//    }
//}
//
//impl<'b> AddAssign<&'b p8> for p8 {
//    #[inline]
//    fn add_assign(&mut self, other: &'b p8) {
//        *self = self.add(*other)
//    }
//}
//
//impl Sum<p8> for p8 {
//    #[inline]
//    fn sum<I>(iter: I) -> p8
//    where
//        I: Iterator<Item=p8>
//    {
//        iter.fold(p8(0), |a, x| a + x)
//    }
//}
//
//impl<'a> Sum<&'a p8> for p8 {
//    #[inline]
//    fn sum<I>(iter: I) -> p8
//    where
//        I: Iterator<Item=&'a p8>
//    {
//        iter.fold(p8(0), |a, x| a + *x)
//    }
//}
//
////// Subtraction ////
//
//impl Sub for p8 {
//    type Output = p8;
//
//    /// Polynomial subtraction, aka xor
//    #[inline]
//    fn sub(self: p8, other: p8) -> p8 {
//        // polynomial subtraction is just xor
//        p8::sub(self, other)
//    }
//}
//
////// Multiplication ////
//
//impl Mul for p8 {
//    type Output = p8;
//
//    /// Polynomial multiplication
//    ///
//    /// This attempts to use carry-less multiplication
//    /// instructions when available (pclmulqdq on x86_64,
//    /// pmul on aarch64), otherwise falls back to an expensive
//    /// manual implementation
//    ///
//    #[inline]
//    fn mul(self: p8, other: p8) -> p8 {
//        p8(xmul8(self.0, other.0) as u8)
//    }
//}
//
////// Division ////
//
//impl Div for p8 {
//    type Output = p8;
//
//    /// Polynomial division
//    ///
//    /// Note, this is expensive. There isn't really any hardware for polynomial
//    /// division, so we need to always use an expensive manual implementation
//    ///
//    /// This will also panic if b == 0
//    ///
//    #[inline]
//    fn div(self: p8, other: p8) -> p8 {
//        p8(xdivmod8(self.0, other.0).unwrap().0)
//    }
//}
//
//impl Rem for p8 {
//    type Output = p8;
//
//    /// Polynomial remainder
//    ///
//    /// Note, this is expensive. There isn't really any hardware for polynomial
//    /// division, so we need to always use an expensive manual implementation
//    ///
//    /// This will also panic if b == 0
//    ///
//    #[inline]
//    fn rem(self: p8, other: p8) -> p8 {
//        p8(xdivmod8(self.0, other.0).unwrap().1)
//    }
//}
