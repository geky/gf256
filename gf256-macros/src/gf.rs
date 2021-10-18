///! Template for polynomial types

use std::ops::*;
use std::iter::*;
use std::fmt;
use std::str::FromStr;
use std::num::TryFromIntError;
use std::num::ParseIntError;
use __crate::p8;
use __crate::p16;
use __crate::p32;
use __crate::traits::TryFrom;
use __crate::traits::FromLossy;
use __crate::internal::cfg_if::cfg_if;


/// A gf(256) field
#[allow(non_camel_case_types)]
#[derive(Default, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct __gf(pub u8);

impl __gf {
    /// Primitive polynomial that defines the field
    pub const POLYNOMIAL: p16 = p16(__polynomial);

    /// Generator polynomial in the field
    pub const GENERATOR: __gf = __gf(__generator);

    // Generate log/antilog tables using our generator
    // if we're in table mode
    //
    #[cfg(__if(__table))]
    const LOG_TABLE: [u8; 256] = Self::LOG_EXP_TABLES.0;
    #[cfg(__if(__table))]
    const EXP_TABLE: [u8; 256] = Self::LOG_EXP_TABLES.1;
    #[cfg(__if(__table))]
    const LOG_EXP_TABLES: ([u8; 256], [u8; 256]) = {
        let mut log_table = [0u8; 256];
        let mut exp_table = [0u8; 256];

        let mut x = 1u16;
        let mut i = 0u16;
        while i < 256 {
            log_table[x as usize] = i as u8;
            exp_table[i as usize] = x as u8;

            x = p16(x).naive_mul(p16(__generator)).0;
            if x >= 256 {
                x ^= __polynomial;
            }

            i += 1;
        }

        log_table[0] = 0xff; // log(0) is undefined
        log_table[1] = 0x00; // log(1) is 0
        (log_table, exp_table)
    };

    // Generate constant for Barret's reduction if we're
    // in Barret mode
    #[cfg(__if(__barret))]
    const BARRET_CONSTANT: p16 = {
        p16(p32(0x10000).naive_div(p32(__polynomial)).0 as u16)
    };

    /// Addition over gf(256), aka xor
    #[inline]
    pub const fn add(self, other: __gf) -> __gf {
        __gf(self.0 ^ other.0)
    }

    /// Subtraction over gf(256), aka xor
    #[inline]
    pub const fn sub(self, other: __gf) -> __gf {
        __gf(self.0 ^ other.0)
    }

    /// Naive multiplication over gf(256)
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts
    ///
    #[inline]
    pub const fn naive_mul(self, other: __gf) -> __gf {
        __gf(
            p16(self.0 as u16)
                .naive_mul(p16(other.0 as u16))
                .naive_rem(p16(__polynomial))
                .0 as u8
        )
    }

    /// Naive exponentiation over gf(256)
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts
    ///
    #[inline]
    pub const fn naive_pow(self, exp: u32) -> __gf {
        let mut a = self;
        let mut exp = exp;
        let mut x = __gf(1);
        loop {
            if exp & 1 != 0 {
                x = x.naive_mul(a);
            }

            exp >>= 1;
            if exp == 0 {
                return x;
            }
            a = a.naive_mul(a);
        }
    }

    /// Naive multiplicative inverse over gf(256)
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts
    ///
    #[inline]
    pub const fn checked_naive_recip(self) -> Option<__gf> {
        if self.0 == 0 {
            return None;
        }

        // x^-1 = x^255-1 = x^254
        Some(self.naive_pow(254))
    }

    /// Naive multiplicative inverse over gf(256)
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts
    ///
    /// This will panic if b == 0
    ///
    #[inline]
    pub const fn naive_recip(self) -> __gf {
        match self.checked_naive_recip() {
            Some(x) => x,
            None => __gf(1 / 0),
        }
    }

    /// Naive division over gf(256)
    ///
    #[inline]
    pub const fn checked_naive_div(self, other: __gf) -> Option<__gf> {
        match other.checked_naive_recip() {
            Some(other_recip) => Some(self.naive_mul(other_recip)),
            None => None,
        }
    }

    /// Naive division over gf(256)
    ///
    /// This will panic if b == 0
    ///
    #[inline]
    pub const fn naive_div(self, other: __gf) -> __gf {
        match self.checked_naive_div(other) {
            Some(x) => x,
            None => __gf(self.0 / 0),
        }
    }

    /// Multiplication over gf(256)
    ///
    /// TODO doc more?
    ///
    #[inline]
    pub fn mul(self, other: __gf) -> __gf {
        cfg_if! {
            if #[cfg(__if(__table))] {
                // multiplication over gf(256) using log/antilog tables
                if self.0 == 0 || other.0 == 0 {
                    // special case for 0, this can't be constant-time
                    // anyways because tables are involved
                    __gf(0)
                } else {
                    // a*b = g^(log_g(a) + log_g(b))
                    //
                    // note our addition can overflow, and there are only
                    // 255 elements in multiplication so this is a bit awkward
                    //
                    let x = match
                        Self::LOG_TABLE[self.0 as usize]
                            .overflowing_add(Self::LOG_TABLE[other.0 as usize])
                    {
                        (x, false) => x,
                        (x, true)  => x.wrapping_sub(255),
                    };
                    __gf(Self::EXP_TABLE[x as usize])
                }
            } else if #[cfg(__if(__barret))] {
                // multiplication over gf(256) using Barret reduction
                //
                // Barret reduction is a method for turning division/remainder
                // by a constant into multiplication by a couple constants. It's
                // useful here if we have hardware xmul instructions, though
                // it may be more expensive if xmul is naive.
                //
                let x = p16(self.0 as u16) * p16(other.0 as u16);
                let q = (p16::mul(x >> 8, Self::BARRET_CONSTANT) >> 8);
                __gf((p16::mul(q, Self::POLYNOMIAL) + x).0 as u8)
            } else {
                // fallback to naive multiplication over gf(256)
                self.naive_mul(other)
            }
        }
    }

    /// Exponentiation over gf(256)
    ///
    /// TODO doc more?
    ///
    #[inline]
    pub fn pow(self, exp: u32) -> __gf {
        cfg_if! {
            if #[cfg(__if(__table))] {
                // another shortcut! if we are in table mode, the log/antilog
                // tables let us compute the pow with traditional integer
                // operations. Expensive integer operations, but less expensive
                // than looping.
                //
                if exp == 0 {
                    __gf(1)
                } else if self.0 == 0 {
                    __gf(0)
                } else {
                    let x = ((Self::LOG_TABLE[self.0 as usize] as u32) * exp) % 255;
                    __gf(Self::EXP_TABLE[x as usize])
                }
            } else {
                let mut a = self;
                let mut exp = exp;
                let mut x = __gf(1);
                loop {
                    if exp & 1 != 0 {
                        x = x.mul(a);
                    }

                    exp >>= 1;
                    if exp == 0 {
                        return x;
                    }
                    a = a.mul(a);
                }
            }
        }
    }

    /// Multiplicative inverse over gf(256)
    ///
    /// TODO doc more?
    ///
    #[inline]
    pub fn checked_recip(self) -> Option<__gf> {
        if self.0 == 0 {
            return None;
        }

        cfg_if! {
            if #[cfg(__if(__table))] {
                // we can take a shortcut here if we are in table mode, by
                // directly using the log/antilog tables to find the reciprocal
                //
                // x^-1 = g^log_g(x^-1) = g^-log_g(x) = g^(255-log_g(x))
                //
                let x = 255 - Self::LOG_TABLE[self.0 as usize];
                Some(__gf(Self::EXP_TABLE[x as usize]))
            } else {
                // x^-1 = x^255-1 = x^254
                //
                Some(self.pow(254))
            }
        }
    }

    /// Multiplicative inverse over gf(256)
    ///
    /// TODO doc more?
    ///
    /// This will panic if b == 0
    ///
    #[inline]
    pub fn recip(self) -> __gf {
        self.checked_recip()
            .expect("gf256 division by zero")
    }

    /// Division over gf(256)
    ///
    /// TODO doc more?
    ///
    #[inline]
    pub fn checked_div(self, other: __gf) -> Option<__gf> {
        if other.0 == 0 {
            return None;
        }

        cfg_if! {
            if #[cfg(__if(__table))] {
                // more table mode shortcuts, this just shaves off a pair of lookups
                //
                // a/b = a*b^-1 = g^(log_g(a)+log_g(b^-1)) = g^(log_g(a)-log_g(b)) = g^(log_g(a)+255-log_g(b))
                //
                if self.0 == 0 {
                    Some(__gf(0))
                } else {
                    let x = match
                        Self::LOG_TABLE[self.0 as usize]
                            .overflowing_add(255 - Self::LOG_TABLE[other.0 as usize])
                    {
                        (x, false) => x,
                        (x, true)  => x.wrapping_sub(255),
                    };
                    Some(__gf(Self::EXP_TABLE[x as usize]))
                }
            } else {
                // a/b = a*b^1
                //
                Some(self * other.recip())
            }
        }
    }

    /// Division over gf(256)
    ///
    /// TODO doc more?
    ///
    /// This will panic if b == 0
    ///
    #[inline]
    pub fn div(self, other: __gf) -> __gf {
        self.checked_div(other)
            .expect("gf256 division by zero")
    }
}


//// Conversions into __gf ////

impl From<p8> for __gf {
    #[inline]
    fn from(x: p8) -> __gf {
        __gf(x.0)
    }
}

impl From<u8> for __gf {
    #[inline]
    fn from(x: u8) -> __gf {
        __gf(x)
    }
}

impl From<bool> for __gf {
    #[inline]
    fn from(x: bool) -> __gf {
        __gf(u8::from(x))
    }
}

impl TryFrom<u16> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: u16) -> Result<__gf, Self::Error> {
        Ok(__gf(u8::try_from(x)?))
    }
}

impl TryFrom<u32> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: u32) -> Result<__gf, Self::Error> {
        Ok(__gf(u8::try_from(x)?))
    }
}

impl TryFrom<u64> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: u64) -> Result<__gf, Self::Error> {
        Ok(__gf(u8::try_from(x)?))
    }
}

impl TryFrom<u128> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: u128) -> Result<__gf, Self::Error> {
        Ok(__gf(u8::try_from(x)?))
    }
}

impl TryFrom<usize> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: usize) -> Result<__gf, Self::Error> {
        Ok(__gf(u8::try_from(x)?))
    }
}

impl TryFrom<__crate::p16> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __crate::p16) -> Result<__gf, Self::Error> {
        Ok(__gf(u8::try_from(x.0)?))
    }
}

impl TryFrom<__crate::p32> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __crate::p32) -> Result<__gf, Self::Error> {
        Ok(__gf(u8::try_from(x.0)?))
    }
}

impl TryFrom<__crate::p64> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __crate::p64) -> Result<__gf, Self::Error> {
        Ok(__gf(u8::try_from(x.0)?))
    }
}

impl TryFrom<__crate::p128> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __crate::p128) -> Result<__gf, Self::Error> {
        Ok(__gf(u8::try_from(x.0)?))
    }
}

impl TryFrom<__crate::psize> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __crate::psize) -> Result<__gf, Self::Error> {
        Ok(__gf(u8::try_from(x.0)?))
    }
}

impl FromLossy<u16> for __gf {
    #[inline]
    fn from_lossy(x: u16) -> __gf {
        __gf(x as u8)
    }
}

impl FromLossy<u32> for __gf {
    #[inline]
    fn from_lossy(x: u32) -> __gf {
        __gf(x as u8)
    }
}

impl FromLossy<u64> for __gf {
    #[inline]
    fn from_lossy(x: u64) -> __gf {
        __gf(x as u8)
    }
}

impl FromLossy<u128> for __gf {
    #[inline]
    fn from_lossy(x: u128) -> __gf {
        __gf(x as u8)
    }
}

impl FromLossy<__crate::p16> for __gf {
    #[inline]
    fn from_lossy(x: __crate::p16) -> __gf {
        __gf(x.0 as u8)
    }
}

impl FromLossy<__crate::p32> for __gf {
    #[inline]
    fn from_lossy(x: __crate::p32) -> __gf {
        __gf(x.0 as u8)
    }
}

impl FromLossy<__crate::p64> for __gf {
    #[inline]
    fn from_lossy(x: __crate::p64) -> __gf {
        __gf(x.0 as u8)
    }
}

impl FromLossy<__crate::p128> for __gf {
    #[inline]
    fn from_lossy(x: __crate::p128) -> __gf {
        __gf(x.0 as u8)
    }
}

impl FromLossy<__crate::psize> for __gf {
    #[inline]
    fn from_lossy(x: __crate::psize) -> __gf {
        __gf(x.0 as u8)
    }
}


//// Conversions from __gf ////

impl From<__gf> for p8 {
    #[inline]
    fn from(x: __gf) -> p8 {
        p8(x.0)
    }
}

impl From<__gf> for u8 {
    #[inline]
    fn from(x: __gf) -> u8 {
        x.0
    }
}

impl From<__gf> for u16 {
    #[inline]
    fn from(x: __gf) -> u16 {
        u16::from(x.0)
    }
}

impl From<__gf> for u32 {
    #[inline]
    fn from(x: __gf) -> u32 {
        u32::from(x.0)
    }
}

impl From<__gf> for u64 {
    #[inline]
    fn from(x: __gf) -> u64 {
        u64::from(x.0)
    }
}

impl From<__gf> for u128 {
    #[inline]
    fn from(x: __gf) -> u128 {
        u128::from(x.0)
    }
}

impl From<__gf> for usize {
    #[inline]
    fn from(x: __gf) -> usize {
        usize::from(x.0)
    }
}

impl From<__gf> for i16 {
    #[inline]
    fn from(x: __gf) -> i16 {
        i16::from(x.0)
    }
}

impl From<__gf> for i32 {
    #[inline]
    fn from(x: __gf) -> i32 {
        i32::from(x.0)
    }
}

impl From<__gf> for i64 {
    #[inline]
    fn from(x: __gf) -> i64 {
        i64::from(x.0)
    }
}

impl From<__gf> for i128 {
    #[inline]
    fn from(x: __gf) -> i128 {
        i128::from(x.0)
    }
}

impl From<__gf> for isize {
    #[inline]
    fn from(x: __gf) -> isize {
        isize::from(x.0)
    }
}

impl TryFrom<__gf> for i8 {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __gf) -> Result<i8, Self::Error> {
        i8::try_from(x.0)
    }
}

impl FromLossy<__gf> for i8 {
    #[inline]
    fn from_lossy(x: __gf) -> i8 {
        x.0 as i8
    }
}


//// Negate ////

impl Neg for __gf {
    type Output = __gf;

    /// Negate is a noop for polynomials
    #[inline]
    fn neg(self) -> __gf {
        self
    }
}

impl Neg for &__gf {
    type Output = __gf;

    /// Negate is a noop for polynomials
    #[inline]
    fn neg(self) -> __gf {
        *self
    }
}


//// Addition ////

impl Add<__gf> for __gf {
    type Output = __gf;

    /// Polynomial addition, aka xor
    #[inline]
    fn add(self, other: __gf) -> __gf {
        __gf::add(self, other)
    }
}

impl Add<__gf> for &__gf {
    type Output = __gf;

    /// Polynomial addition, aka xor
    #[inline]
    fn add(self, other: __gf) -> __gf {
        __gf::add(*self, other)
    }
}

impl Add<&__gf> for __gf {
    type Output = __gf;

    /// Polynomial addition, aka xor
    #[inline]
    fn add(self, other: &__gf) -> __gf {
        __gf::add(self, *other)
    }
}

impl Add<&__gf> for &__gf {
    type Output = __gf;

    /// Polynomial addition, aka xor
    #[inline]
    fn add(self, other: &__gf) -> __gf {
        __gf::add(*self, *other)
    }
}

impl AddAssign<__gf> for __gf {
    #[inline]
    fn add_assign(&mut self, other: __gf) {
        *self = self.add(other)
    }
}

impl AddAssign<&__gf> for __gf {
    #[inline]
    fn add_assign(&mut self, other: &__gf) {
        *self = self.add(*other)
    }
}

impl Sum<__gf> for __gf {
    #[inline]
    fn sum<I>(iter: I) -> __gf
    where
        I: Iterator<Item=__gf>
    {
        iter.fold(__gf(0), |a, x| a + x)
    }
}

impl<'a> Sum<&'a __gf> for __gf {
    #[inline]
    fn sum<I>(iter: I) -> __gf
    where
        I: Iterator<Item=&'a __gf>
    {
        iter.fold(__gf(0), |a, x| a + *x)
    }
}


//// Subtraction ////

impl Sub for __gf {
    type Output = __gf;

    /// Polynomial subtraction, aka xor
    #[inline]
    fn sub(self, other: __gf) -> __gf {
        __gf::sub(self, other)
    }
}

impl Sub<__gf> for &__gf {
    type Output = __gf;

    /// Polynomial subtraction, aka xor
    #[inline]
    fn sub(self, other: __gf) -> __gf {
        __gf::sub(*self, other)
    }
}

impl Sub<&__gf> for __gf {
    type Output = __gf;

    /// Polynomial subtraction, aka xor
    #[inline]
    fn sub(self, other: &__gf) -> __gf {
        __gf::sub(self, *other)
    }
}

impl Sub<&__gf> for &__gf {
    type Output = __gf;

    /// Polynomial subtraction, aka xor
    #[inline]
    fn sub(self, other: &__gf) -> __gf {
        __gf::sub(*self, *other)
    }
}

impl SubAssign<__gf> for __gf {
    #[inline]
    fn sub_assign(&mut self, other: __gf) {
        *self = self.sub(other)
    }
}

impl SubAssign<&__gf> for __gf {
    #[inline]
    fn sub_assign(&mut self, other: &__gf) {
        *self = self.sub(*other)
    }
}


//// Multiplication ////

impl Mul for __gf {
    type Output = __gf;

    /// Polynomial multiplication
    ///
    /// This attempts to use carry-less multiplication
    /// instructions when available (pclmulqdq on x86_64,
    /// pmull on aarch64), otherwise falls back to the expensive
    /// naive implementation
    ///
    #[inline]
    fn mul(self, other: __gf) -> __gf {
        __gf::mul(self, other)
    }
}

impl Mul<__gf> for &__gf {
    type Output = __gf;

    /// Polynomial multiplication
    ///
    /// This attempts to use carry-less multiplication
    /// instructions when available (pclmulqdq on x86_64,
    /// pmull on aarch64), otherwise falls back to the expensive
    /// naive implementation
    ///
    #[inline]
    fn mul(self, other: __gf) -> __gf {
        __gf::mul(*self, other)
    }
}

impl Mul<&__gf> for __gf {
    type Output = __gf;

    /// Polynomial multiplication
    ///
    /// This attempts to use carry-less multiplication
    /// instructions when available (pclmulqdq on x86_64,
    /// pmull on aarch64), otherwise falls back to the expensive
    /// naive implementation
    ///
    #[inline]
    fn mul(self, other: &__gf) -> __gf {
        __gf::mul(self, *other)
    }
}

impl Mul<&__gf> for &__gf {
    type Output = __gf;

    /// Polynomial multiplication
    ///
    /// This attempts to use carry-less multiplication
    /// instructions when available (pclmulqdq on x86_64,
    /// pmull on aarch64), otherwise falls back to the expensive
    /// naive implementation
    ///
    #[inline]
    fn mul(self, other: &__gf) -> __gf {
        __gf::mul(*self, *other)
    }
}

impl MulAssign<__gf> for __gf {
    #[inline]
    fn mul_assign(&mut self, other: __gf) {
        *self = self.mul(other)
    }
}

impl MulAssign<&__gf> for __gf {
    #[inline]
    fn mul_assign(&mut self, other: &__gf) {
        *self = self.mul(*other)
    }
}

impl Product<__gf> for __gf {
    #[inline]
    fn product<I>(iter: I) -> __gf
    where
        I: Iterator<Item=__gf>
    {
        iter.fold(__gf(0), |a, x| a * x)
    }
}

impl<'a> Product<&'a __gf> for __gf {
    #[inline]
    fn product<I>(iter: I) -> __gf
    where
        I: Iterator<Item=&'a __gf>
    {
        iter.fold(__gf(0), |a, x| a * *x)
    }
}


//// Division ////

impl Div for __gf {
    type Output = __gf;

    #[inline]
    fn div(self, other: __gf) -> __gf {
        __gf::div(self, other)
    }
}

impl Div<__gf> for &__gf {
    type Output = __gf;

    #[inline]
    fn div(self, other: __gf) -> __gf {
        __gf::div(*self, other)
    }
}

impl Div<&__gf> for __gf {
    type Output = __gf;

    #[inline]
    fn div(self, other: &__gf) -> __gf {
        __gf::div(self, *other)
    }
}

impl Div<&__gf> for &__gf {
    type Output = __gf;

    #[inline]
    fn div(self, other: &__gf) -> __gf {
        __gf::div(*self, *other)
    }
}

impl DivAssign<__gf> for __gf {
    #[inline]
    fn div_assign(&mut self, other: __gf) {
        *self = self.div(other)
    }
}

impl DivAssign<&__gf> for __gf {
    #[inline]
    fn div_assign(&mut self, other: &__gf) {
        *self = self.div(*other)
    }
}


//// Bitwise operations ////

impl Not for __gf {
    type Output = __gf;
    #[inline]
    fn not(self) -> __gf {
        __gf(!self.0)
    }
}

impl Not for &__gf {
    type Output = __gf;
    #[inline]
    fn not(self) -> __gf {
        __gf(!self.0)
    }
}

impl BitAnd<__gf> for __gf {
    type Output = __gf;
    #[inline]
    fn bitand(self, other: __gf) -> __gf {
        __gf(self.0 & other.0)
    }
}

impl BitAnd<__gf> for &__gf {
    type Output = __gf;
    #[inline]
    fn bitand(self, other: __gf) -> __gf {
        __gf(self.0 & other.0)
    }
}

impl BitAnd<&__gf> for __gf {
    type Output = __gf;
    #[inline]
    fn bitand(self, other: &__gf) -> __gf {
        __gf(self.0 & other.0)
    }
}

impl BitAnd<&__gf> for &__gf {
    type Output = __gf;
    #[inline]
    fn bitand(self, other: &__gf) -> __gf {
        __gf(self.0 & other.0)
    }
}

impl BitAndAssign<__gf> for __gf {
    #[inline]
    fn bitand_assign(&mut self, other: __gf) {
        *self = *self & other;
    }
}

impl BitAndAssign<&__gf> for __gf {
    #[inline]
    fn bitand_assign(&mut self, other: &__gf) {
        *self = *self & *other;
    }
}

impl BitAnd<__gf> for u8 {
    type Output = __gf;
    #[inline]
    fn bitand(self, other: __gf) -> __gf {
        __gf(self & other.0)
    }
}

impl BitAnd<__gf> for &u8 {
    type Output = __gf;
    #[inline]
    fn bitand(self, other: __gf) -> __gf {
        __gf(self & other.0)
    }
}

impl BitAnd<&__gf> for u8 {
    type Output = __gf;
    #[inline]
    fn bitand(self, other: &__gf) -> __gf {
        __gf(self & other.0)
    }
}

impl BitAnd<&__gf> for &u8 {
    type Output = __gf;
    #[inline]
    fn bitand(self, other: &__gf) -> __gf {
        __gf(self & other.0)
    }
}

impl BitAnd<u8> for __gf {
    type Output = __gf;
    #[inline]
    fn bitand(self, other: u8) -> __gf {
        __gf(self.0 & other)
    }
}

impl BitAnd<u8> for &__gf {
    type Output = __gf;
    #[inline]
    fn bitand(self, other: u8) -> __gf {
        __gf(self.0 & other)
    }
}

impl BitAnd<&u8> for __gf {
    type Output = __gf;
    #[inline]
    fn bitand(self, other: &u8) -> __gf {
        __gf(self.0 & other)
    }
}

impl BitAnd<&u8> for &__gf {
    type Output = __gf;
    #[inline]
    fn bitand(self, other: &u8) -> __gf {
        __gf(self.0 & other)
    }
}

impl BitAndAssign<u8> for __gf {
    #[inline]
    fn bitand_assign(&mut self, other: u8) {
        *self = *self & other;
    }
}

impl BitAndAssign<&u8> for __gf {
    #[inline]
    fn bitand_assign(&mut self, other: &u8) {
        *self = *self & *other;
    }
}

impl BitOr<__gf> for __gf {
    type Output = __gf;
    #[inline]
    fn bitor(self, other: __gf) -> __gf {
        __gf(self.0 | other.0)
    }
}

impl BitOr<__gf> for &__gf {
    type Output = __gf;
    #[inline]
    fn bitor(self, other: __gf) -> __gf {
        __gf(self.0 | other.0)
    }
}

impl BitOr<&__gf> for __gf {
    type Output = __gf;
    #[inline]
    fn bitor(self, other: &__gf) -> __gf {
        __gf(self.0 | other.0)
    }
}

impl BitOr<&__gf> for &__gf {
    type Output = __gf;
    #[inline]
    fn bitor(self, other: &__gf) -> __gf {
        __gf(self.0 | other.0)
    }
}

impl BitOrAssign<__gf> for __gf {
    #[inline]
    fn bitor_assign(&mut self, other: __gf) {
        *self = *self | other;
    }
}

impl BitOrAssign<&__gf> for __gf {
    #[inline]
    fn bitor_assign(&mut self, other: &__gf) {
        *self = *self | *other;
    }
}

impl BitOr<__gf> for u8 {
    type Output = __gf;
    #[inline]
    fn bitor(self, other: __gf) -> __gf {
        __gf(self | other.0)
    }
}

impl BitOr<__gf> for &u8 {
    type Output = __gf;
    #[inline]
    fn bitor(self, other: __gf) -> __gf {
        __gf(self | other.0)
    }
}

impl BitOr<&__gf> for u8 {
    type Output = __gf;
    #[inline]
    fn bitor(self, other: &__gf) -> __gf {
        __gf(self | other.0)
    }
}

impl BitOr<&__gf> for &u8 {
    type Output = __gf;
    #[inline]
    fn bitor(self, other: &__gf) -> __gf {
        __gf(self | other.0)
    }
}

impl BitOr<u8> for __gf {
    type Output = __gf;
    #[inline]
    fn bitor(self, other: u8) -> __gf {
        __gf(self.0 | other)
    }
}

impl BitOr<u8> for &__gf {
    type Output = __gf;
    #[inline]
    fn bitor(self, other: u8) -> __gf {
        __gf(self.0 | other)
    }
}

impl BitOr<&u8> for __gf {
    type Output = __gf;
    #[inline]
    fn bitor(self, other: &u8) -> __gf {
        __gf(self.0 | other)
    }
}

impl BitOr<&u8> for &__gf {
    type Output = __gf;
    #[inline]
    fn bitor(self, other: &u8) -> __gf {
        __gf(self.0 | other)
    }
}

impl BitOrAssign<u8> for __gf {
    #[inline]
    fn bitor_assign(&mut self, other: u8) {
        *self = *self | other;
    }
}

impl BitOrAssign<&u8> for __gf {
    #[inline]
    fn bitor_assign(&mut self, other: &u8) {
        *self = *self | *other;
    }
}

impl BitXor<__gf> for __gf {
    type Output = __gf;
    #[inline]
    fn bitxor(self, other: __gf) -> __gf {
        __gf(self.0 ^ other.0)
    }
}

impl BitXor<__gf> for &__gf {
    type Output = __gf;
    #[inline]
    fn bitxor(self, other: __gf) -> __gf {
        __gf(self.0 ^ other.0)
    }
}

impl BitXor<&__gf> for __gf {
    type Output = __gf;
    #[inline]
    fn bitxor(self, other: &__gf) -> __gf {
        __gf(self.0 ^ other.0)
    }
}

impl BitXor<&__gf> for &__gf {
    type Output = __gf;
    #[inline]
    fn bitxor(self, other: &__gf) -> __gf {
        __gf(self.0 ^ other.0)
    }
}

impl BitXorAssign<__gf> for __gf {
    #[inline]
    fn bitxor_assign(&mut self, other: __gf) {
        *self = *self ^ other;
    }
}

impl BitXorAssign<&__gf> for __gf {
    #[inline]
    fn bitxor_assign(&mut self, other: &__gf) {
        *self = *self ^ *other;
    }
}

impl BitXor<__gf> for u8 {
    type Output = __gf;
    #[inline]
    fn bitxor(self, other: __gf) -> __gf {
        __gf(self ^ other.0)
    }
}

impl BitXor<__gf> for &u8 {
    type Output = __gf;
    #[inline]
    fn bitxor(self, other: __gf) -> __gf {
        __gf(self ^ other.0)
    }
}

impl BitXor<&__gf> for u8 {
    type Output = __gf;
    #[inline]
    fn bitxor(self, other: &__gf) -> __gf {
        __gf(self ^ other.0)
    }
}

impl BitXor<&__gf> for &u8 {
    type Output = __gf;
    #[inline]
    fn bitxor(self, other: &__gf) -> __gf {
        __gf(self ^ other.0)
    }
}

impl BitXor<u8> for __gf {
    type Output = __gf;
    #[inline]
    fn bitxor(self, other: u8) -> __gf {
        __gf(self.0 ^ other)
    }
}

impl BitXor<u8> for &__gf {
    type Output = __gf;
    #[inline]
    fn bitxor(self, other: u8) -> __gf {
        __gf(self.0 ^ other)
    }
}

impl BitXor<&u8> for __gf {
    type Output = __gf;
    #[inline]
    fn bitxor(self, other: &u8) -> __gf {
        __gf(self.0 ^ other)
    }
}

impl BitXor<&u8> for &__gf {
    type Output = __gf;
    #[inline]
    fn bitxor(self, other: &u8) -> __gf {
        __gf(self.0 ^ other)
    }
}

impl BitXorAssign<u8> for __gf {
    #[inline]
    fn bitxor_assign(&mut self, other: u8) {
        *self = *self ^ other;
    }
}

impl BitXorAssign<&u8> for __gf {
    #[inline]
    fn bitxor_assign(&mut self, other: &u8) {
        *self = *self ^ *other;
    }
}


//// Other bit things ////

impl __gf {
    #[inline]
    pub const fn reverse_bits(self) -> __gf {
        __gf(self.0.reverse_bits())
    }

    #[inline]
    pub const fn count_ones(self) -> u32 {
        self.0.count_ones()
    }

    #[inline]
    pub const fn count_zeros(self) -> u32 {
        self.0.count_zeros()
    }

    #[inline]
    pub const fn leading_ones(self) -> u32 {
        self.0.leading_ones()
    }

    #[inline]
    pub const fn leading_zeros(self) -> u32 {
        self.0.leading_zeros()
    }

    #[inline]
    pub const fn trailing_ones(self) -> u32 {
        self.0.trailing_ones()
    }

    #[inline]
    pub const fn trailing_zeros(self) -> u32 {
        self.0.trailing_zeros()
    }
}


//// Shifts ////

impl __gf {
    #[inline]
    pub const fn checked_shl(self, other: u32) -> Option<__gf> {
        match self.0.checked_shl(other) {
            Some(x) => Some(__gf(x)),
            None => None,
        }
    }

    #[inline]
    pub const fn checked_shr(self, other: u32) -> Option<__gf> {
        match self.0.checked_shr(other) {
            Some(x) => Some(__gf(x)),
            None => None,
        }
    }

    #[inline]
    pub const fn overflowing_shl(self, other: u32) -> (__gf, bool) {
        let (x, o) = self.0.overflowing_shl(other);
        (__gf(x), o)
    }

    #[inline]
    pub const fn overflowing_shr(self, other: u32) -> (__gf, bool) {
        let (x, o) = self.0.overflowing_shr(other);
        (__gf(x), o)
    }

    #[inline]
    pub const fn wrapping_shl(self, other: u32) -> __gf {
        __gf(self.0.wrapping_shl(other))
    }

    #[inline]
    pub const fn wrapping_shr(self, other: u32) -> __gf {
        __gf(self.0.wrapping_shr(other))
    }

    #[inline]
    pub const fn rotate_left(self, other: u32) -> __gf {
        __gf(self.0.rotate_left(other))
    }

    #[inline]
    pub const fn rotate_right(self, other: u32) -> __gf {
        __gf(self.0.rotate_right(other))
    }
}

impl Shl<u8> for __gf {
    type Output = __gf;
    fn shl(self, other: u8) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<u8> for &__gf {
    type Output = __gf;
    fn shl(self, other: u8) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&u8> for __gf {
    type Output = __gf;
    fn shl(self, other: &u8) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&u8> for &__gf {
    type Output = __gf;
    fn shl(self, other: &u8) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<u16> for __gf {
    type Output = __gf;
    fn shl(self, other: u16) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<u16> for &__gf {
    type Output = __gf;
    fn shl(self, other: u16) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&u16> for __gf {
    type Output = __gf;
    fn shl(self, other: &u16) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&u16> for &__gf {
    type Output = __gf;
    fn shl(self, other: &u16) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<u32> for __gf {
    type Output = __gf;
    fn shl(self, other: u32) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<u32> for &__gf {
    type Output = __gf;
    fn shl(self, other: u32) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&u32> for __gf {
    type Output = __gf;
    fn shl(self, other: &u32) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&u32> for &__gf {
    type Output = __gf;
    fn shl(self, other: &u32) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<u64> for __gf {
    type Output = __gf;
    fn shl(self, other: u64) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<u64> for &__gf {
    type Output = __gf;
    fn shl(self, other: u64) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&u64> for __gf {
    type Output = __gf;
    fn shl(self, other: &u64) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&u64> for &__gf {
    type Output = __gf;
    fn shl(self, other: &u64) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<u128> for __gf {
    type Output = __gf;
    fn shl(self, other: u128) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<u128> for &__gf {
    type Output = __gf;
    fn shl(self, other: u128) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&u128> for __gf {
    type Output = __gf;
    fn shl(self, other: &u128) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&u128> for &__gf {
    type Output = __gf;
    fn shl(self, other: &u128) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<usize> for __gf {
    type Output = __gf;
    fn shl(self, other: usize) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<usize> for &__gf {
    type Output = __gf;
    fn shl(self, other: usize) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&usize> for __gf {
    type Output = __gf;
    fn shl(self, other: &usize) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&usize> for &__gf {
    type Output = __gf;
    fn shl(self, other: &usize) -> __gf {
        __gf(self.0 << other)
    }
}

impl ShlAssign<u8> for __gf {
    fn shl_assign(&mut self, other: u8) {
        *self = *self << other;
    }
}

impl ShlAssign<&u8> for __gf {
    fn shl_assign(&mut self, other: &u8) {
        *self = *self << other;
    }
}

impl ShlAssign<u16> for __gf {
    fn shl_assign(&mut self, other: u16) {
        *self = *self << other;
    }
}

impl ShlAssign<&u16> for __gf {
    fn shl_assign(&mut self, other: &u16) {
        *self = *self << other;
    }
}

impl ShlAssign<u32> for __gf {
    fn shl_assign(&mut self, other: u32) {
        *self = *self << other;
    }
}

impl ShlAssign<&u32> for __gf {
    fn shl_assign(&mut self, other: &u32) {
        *self = *self << other;
    }
}

impl ShlAssign<u64> for __gf {
    fn shl_assign(&mut self, other: u64) {
        *self = *self << other;
    }
}

impl ShlAssign<&u64> for __gf {
    fn shl_assign(&mut self, other: &u64) {
        *self = *self << other;
    }
}

impl ShlAssign<u128> for __gf {
    fn shl_assign(&mut self, other: u128) {
        *self = *self << other;
    }
}

impl ShlAssign<&u128> for __gf {
    fn shl_assign(&mut self, other: &u128) {
        *self = *self << other;
    }
}

impl ShlAssign<usize> for __gf {
    fn shl_assign(&mut self, other: usize) {
        *self = *self << other;
    }
}

impl ShlAssign<&usize> for __gf {
    fn shl_assign(&mut self, other: &usize) {
        *self = *self << other;
    }
}

impl Shr<u8> for __gf {
    type Output = __gf;
    fn shr(self, other: u8) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<u8> for &__gf {
    type Output = __gf;
    fn shr(self, other: u8) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&u8> for __gf {
    type Output = __gf;
    fn shr(self, other: &u8) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&u8> for &__gf {
    type Output = __gf;
    fn shr(self, other: &u8) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<u16> for __gf {
    type Output = __gf;
    fn shr(self, other: u16) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<u16> for &__gf {
    type Output = __gf;
    fn shr(self, other: u16) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&u16> for __gf {
    type Output = __gf;
    fn shr(self, other: &u16) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&u16> for &__gf {
    type Output = __gf;
    fn shr(self, other: &u16) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<u32> for __gf {
    type Output = __gf;
    fn shr(self, other: u32) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<u32> for &__gf {
    type Output = __gf;
    fn shr(self, other: u32) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&u32> for __gf {
    type Output = __gf;
    fn shr(self, other: &u32) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&u32> for &__gf {
    type Output = __gf;
    fn shr(self, other: &u32) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<u64> for __gf {
    type Output = __gf;
    fn shr(self, other: u64) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<u64> for &__gf {
    type Output = __gf;
    fn shr(self, other: u64) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&u64> for __gf {
    type Output = __gf;
    fn shr(self, other: &u64) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&u64> for &__gf {
    type Output = __gf;
    fn shr(self, other: &u64) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<u128> for __gf {
    type Output = __gf;
    fn shr(self, other: u128) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<u128> for &__gf {
    type Output = __gf;
    fn shr(self, other: u128) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&u128> for __gf {
    type Output = __gf;
    fn shr(self, other: &u128) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&u128> for &__gf {
    type Output = __gf;
    fn shr(self, other: &u128) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<usize> for __gf {
    type Output = __gf;
    fn shr(self, other: usize) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<usize> for &__gf {
    type Output = __gf;
    fn shr(self, other: usize) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&usize> for __gf {
    type Output = __gf;
    fn shr(self, other: &usize) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&usize> for &__gf {
    type Output = __gf;
    fn shr(self, other: &usize) -> __gf {
        __gf(self.0 >> other)
    }
}

impl ShrAssign<u8> for __gf {
    fn shr_assign(&mut self, other: u8) {
        *self = *self >> other;
    }
}

impl ShrAssign<&u8> for __gf {
    fn shr_assign(&mut self, other: &u8) {
        *self = *self >> other;
    }
}

impl ShrAssign<u16> for __gf {
    fn shr_assign(&mut self, other: u16) {
        *self = *self >> other;
    }
}

impl ShrAssign<&u16> for __gf {
    fn shr_assign(&mut self, other: &u16) {
        *self = *self >> other;
    }
}

impl ShrAssign<u32> for __gf {
    fn shr_assign(&mut self, other: u32) {
        *self = *self >> other;
    }
}

impl ShrAssign<&u32> for __gf {
    fn shr_assign(&mut self, other: &u32) {
        *self = *self >> other;
    }
}

impl ShrAssign<u64> for __gf {
    fn shr_assign(&mut self, other: u64) {
        *self = *self >> other;
    }
}

impl ShrAssign<&u64> for __gf {
    fn shr_assign(&mut self, other: &u64) {
        *self = *self >> other;
    }
}

impl ShrAssign<u128> for __gf {
    fn shr_assign(&mut self, other: u128) {
        *self = *self >> other;
    }
}

impl ShrAssign<&u128> for __gf {
    fn shr_assign(&mut self, other: &u128) {
        *self = *self >> other;
    }
}

impl ShrAssign<usize> for __gf {
    fn shr_assign(&mut self, other: usize) {
        *self = *self >> other;
    }
}

impl ShrAssign<&usize> for __gf {
    fn shr_assign(&mut self, other: &usize) {
        *self = *self >> other;
    }
}

impl Shl<i8> for __gf {
    type Output = __gf;
    fn shl(self, other: i8) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<i8> for &__gf {
    type Output = __gf;
    fn shl(self, other: i8) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&i8> for __gf {
    type Output = __gf;
    fn shl(self, other: &i8) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&i8> for &__gf {
    type Output = __gf;
    fn shl(self, other: &i8) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<i16> for __gf {
    type Output = __gf;
    fn shl(self, other: i16) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<i16> for &__gf {
    type Output = __gf;
    fn shl(self, other: i16) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&i16> for __gf {
    type Output = __gf;
    fn shl(self, other: &i16) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&i16> for &__gf {
    type Output = __gf;
    fn shl(self, other: &i16) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<i32> for __gf {
    type Output = __gf;
    fn shl(self, other: i32) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<i32> for &__gf {
    type Output = __gf;
    fn shl(self, other: i32) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&i32> for __gf {
    type Output = __gf;
    fn shl(self, other: &i32) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&i32> for &__gf {
    type Output = __gf;
    fn shl(self, other: &i32) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<i64> for __gf {
    type Output = __gf;
    fn shl(self, other: i64) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<i64> for &__gf {
    type Output = __gf;
    fn shl(self, other: i64) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&i64> for __gf {
    type Output = __gf;
    fn shl(self, other: &i64) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&i64> for &__gf {
    type Output = __gf;
    fn shl(self, other: &i64) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<i128> for __gf {
    type Output = __gf;
    fn shl(self, other: i128) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<i128> for &__gf {
    type Output = __gf;
    fn shl(self, other: i128) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&i128> for __gf {
    type Output = __gf;
    fn shl(self, other: &i128) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&i128> for &__gf {
    type Output = __gf;
    fn shl(self, other: &i128) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<isize> for __gf {
    type Output = __gf;
    fn shl(self, other: isize) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<isize> for &__gf {
    type Output = __gf;
    fn shl(self, other: isize) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&isize> for __gf {
    type Output = __gf;
    fn shl(self, other: &isize) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&isize> for &__gf {
    type Output = __gf;
    fn shl(self, other: &isize) -> __gf {
        __gf(self.0 << other)
    }
}

impl ShlAssign<i8> for __gf {
    fn shl_assign(&mut self, other: i8) {
        *self = *self << other;
    }
}

impl ShlAssign<&i8> for __gf {
    fn shl_assign(&mut self, other: &i8) {
        *self = *self << other;
    }
}

impl ShlAssign<i16> for __gf {
    fn shl_assign(&mut self, other: i16) {
        *self = *self << other;
    }
}

impl ShlAssign<&i16> for __gf {
    fn shl_assign(&mut self, other: &i16) {
        *self = *self << other;
    }
}

impl ShlAssign<i32> for __gf {
    fn shl_assign(&mut self, other: i32) {
        *self = *self << other;
    }
}

impl ShlAssign<&i32> for __gf {
    fn shl_assign(&mut self, other: &i32) {
        *self = *self << other;
    }
}

impl ShlAssign<i64> for __gf {
    fn shl_assign(&mut self, other: i64) {
        *self = *self << other;
    }
}

impl ShlAssign<&i64> for __gf {
    fn shl_assign(&mut self, other: &i64) {
        *self = *self << other;
    }
}

impl ShlAssign<i128> for __gf {
    fn shl_assign(&mut self, other: i128) {
        *self = *self << other;
    }
}

impl ShlAssign<&i128> for __gf {
    fn shl_assign(&mut self, other: &i128) {
        *self = *self << other;
    }
}

impl ShlAssign<isize> for __gf {
    fn shl_assign(&mut self, other: isize) {
        *self = *self << other;
    }
}

impl ShlAssign<&isize> for __gf {
    fn shl_assign(&mut self, other: &isize) {
        *self = *self << other;
    }
}

impl Shr<i8> for __gf {
    type Output = __gf;
    fn shr(self, other: i8) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<i8> for &__gf {
    type Output = __gf;
    fn shr(self, other: i8) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&i8> for __gf {
    type Output = __gf;
    fn shr(self, other: &i8) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&i8> for &__gf {
    type Output = __gf;
    fn shr(self, other: &i8) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<i16> for __gf {
    type Output = __gf;
    fn shr(self, other: i16) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<i16> for &__gf {
    type Output = __gf;
    fn shr(self, other: i16) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&i16> for __gf {
    type Output = __gf;
    fn shr(self, other: &i16) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&i16> for &__gf {
    type Output = __gf;
    fn shr(self, other: &i16) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<i32> for __gf {
    type Output = __gf;
    fn shr(self, other: i32) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<i32> for &__gf {
    type Output = __gf;
    fn shr(self, other: i32) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&i32> for __gf {
    type Output = __gf;
    fn shr(self, other: &i32) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&i32> for &__gf {
    type Output = __gf;
    fn shr(self, other: &i32) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<i64> for __gf {
    type Output = __gf;
    fn shr(self, other: i64) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<i64> for &__gf {
    type Output = __gf;
    fn shr(self, other: i64) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&i64> for __gf {
    type Output = __gf;
    fn shr(self, other: &i64) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&i64> for &__gf {
    type Output = __gf;
    fn shr(self, other: &i64) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<i128> for __gf {
    type Output = __gf;
    fn shr(self, other: i128) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<i128> for &__gf {
    type Output = __gf;
    fn shr(self, other: i128) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&i128> for __gf {
    type Output = __gf;
    fn shr(self, other: &i128) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&i128> for &__gf {
    type Output = __gf;
    fn shr(self, other: &i128) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<isize> for __gf {
    type Output = __gf;
    fn shr(self, other: isize) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<isize> for &__gf {
    type Output = __gf;
    fn shr(self, other: isize) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&isize> for __gf {
    type Output = __gf;
    fn shr(self, other: &isize) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&isize> for &__gf {
    type Output = __gf;
    fn shr(self, other: &isize) -> __gf {
        __gf(self.0 >> other)
    }
}

impl ShrAssign<i8> for __gf {
    fn shr_assign(&mut self, other: i8) {
        *self = *self >> other;
    }
}

impl ShrAssign<&i8> for __gf {
    fn shr_assign(&mut self, other: &i8) {
        *self = *self >> other;
    }
}

impl ShrAssign<i16> for __gf {
    fn shr_assign(&mut self, other: i16) {
        *self = *self >> other;
    }
}

impl ShrAssign<&i16> for __gf {
    fn shr_assign(&mut self, other: &i16) {
        *self = *self >> other;
    }
}

impl ShrAssign<i32> for __gf {
    fn shr_assign(&mut self, other: i32) {
        *self = *self >> other;
    }
}

impl ShrAssign<&i32> for __gf {
    fn shr_assign(&mut self, other: &i32) {
        *self = *self >> other;
    }
}

impl ShrAssign<i64> for __gf {
    fn shr_assign(&mut self, other: i64) {
        *self = *self >> other;
    }
}

impl ShrAssign<&i64> for __gf {
    fn shr_assign(&mut self, other: &i64) {
        *self = *self >> other;
    }
}

impl ShrAssign<i128> for __gf {
    fn shr_assign(&mut self, other: i128) {
        *self = *self >> other;
    }
}

impl ShrAssign<&i128> for __gf {
    fn shr_assign(&mut self, other: &i128) {
        *self = *self >> other;
    }
}

impl ShrAssign<isize> for __gf {
    fn shr_assign(&mut self, other: isize) {
        *self = *self >> other;
    }
}

impl ShrAssign<&isize> for __gf {
    fn shr_assign(&mut self, other: &isize) {
        *self = *self >> other;
    }
}


//// To/from strings ////

impl fmt::Debug for __gf {
    /// Note, we use LowerHex for Debug, since this is a more useful
    /// representation of binary polynomials
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.pad(&format!("0x{:02x}", self.0))
    }
}

impl fmt::Display for __gf {
    /// Note, we use LowerHex for Display since this is a more useful
    /// representation of binary polynomials
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.pad(&format!("0x{:02x}", self.0))
    }
}

impl fmt::Binary for __gf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        <u8 as fmt::Binary>::fmt(&self.0, f)
    }
}

impl fmt::Octal for __gf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        <u8 as fmt::Octal>::fmt(&self.0, f)
    }
}

impl fmt::LowerHex for __gf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        <u8 as fmt::LowerHex>::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for __gf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        <u8 as fmt::UpperHex>::fmt(&self.0, f)
    }
}

impl FromStr for __gf {
    type Err = ParseIntError;

    /// Note, in order to match Display, this from_str takes and only takes
    /// hexadecimal strings starting with "0x". If you need a different radix
    /// there is from_str_radix.
    ///
    fn from_str(s: &str) -> Result<__gf, ParseIntError> {
        if s.starts_with("0x") {
            Ok(__gf(u8::from_str_radix(&s[2..], 16)?))
        } else {
            "".parse::<u8>()?;
            unreachable!()
        }
    }
}

impl __gf {
    pub fn from_str_radix(s: &str, radix: u32) -> Result<__gf, ParseIntError> {
        Ok(__gf(u8::from_str_radix(s, radix)?))
    }
}
