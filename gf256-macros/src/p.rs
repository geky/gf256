///! Template for polynomial types

use std::mem::size_of;
use std::ops::*;
use std::iter::*;
use __crate::internal::cfg_if::cfg_if;

/// A type representing a gf(2) polynomial
#[allow(non_camel_case_types)]
#[derive(Default, Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct __p(pub __u);

impl __p {
    /// Polynomial addition, aka xor
    #[inline]
    pub const fn add(self, other: __p) -> __p {
        __p(self.0 ^ other.0)
    }

    /// Polynomial subtraction, aka xor
    #[inline]
    pub const fn sub(self, other: __p) -> __p {
        __p(self.0 ^ other.0)
    }

    /// Naive polynomial multiplication
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts
    ///
    /// Note this wraps around the boundary of the type, and returns
    /// a flag indicating of overflow occured
    ///
    #[inline]
    pub const fn overflowing_naive_mul(self, other: __p) -> (__p, bool) {
        // x bits * y bits = x+y-1 bits, if this is more bits than the
        // width we will overflow
        let o = self.0.leading_zeros() + other.0.leading_zeros() < __width-1;
        (self.wrapping_naive_mul(other), o)
    }

    /// Naive polynomial multiplication
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts
    ///
    /// Note this returns None if an overflow occured
    ///
    #[inline]
    pub const fn checked_naive_mul(self, other: __p) -> Option<__p> {
        match self.overflowing_naive_mul(other) {
            (_, true ) => None,
            (x, false) => Some(x),
        }
    }

    /// Naive polynomial multiplication
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts
    ///
    /// Note this wraps around the boundary of the type
    ///
    #[inline]
    pub const fn wrapping_naive_mul(self, other: __p) -> __p {
        let a = self.0;
        let b = other.0;
        let mut x = 0;
        let mut i = 0;
        while i < 8*size_of::<__u>() {
            // TODO should this be constant-time?
            x ^= if a & (1 << i) != 0 { b << i } else { 0 };
            i += 1;
        }
        __p(x)
    }

    /// Naive polynomial multiplication
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts
    ///
    /// Note this panics if an overflow occured and debug_assertions
    /// are enabled
    ///
    #[inline]
    pub const fn naive_mul(self, other: __p) -> __p {
        cfg_if! {
            // TODO feature flag for overflow-checks?
            if #[cfg(debug_assertions)] {
                match self.checked_naive_mul(other) {
                    Some(x) => x,
                    None => __p(self.0 / 0),
                }
            } else {
                self.wrapping_naive_mul(other)
            }
        }
    }

    /// Polynomial multiplication
    ///
    /// This attempts to use carry-less multiplication
    /// instructions when available (pclmulqdq on x86_64,
    /// pmul on aarch64), otherwise falls back to the expensive
    /// naive implementation
    ///
    /// Note this wraps around the boundary of the type, and returns
    /// a flag indicating of overflow occured
    ///
    #[inline]
    pub fn overflowing_mul(self, other: __p) -> (__p, bool) {
        // x bits * y bits = x+y-1 bits, if this is more bits than the
        // width we will overflow
        let o = self.0.leading_zeros() + other.0.leading_zeros() < __width-1;
        (self.wrapping_mul(other), o)
    }

    /// Polynomial multiplication
    ///
    /// This attempts to use carry-less multiplication
    /// instructions when available (pclmulqdq on x86_64,
    /// pmul on aarch64), otherwise falls back to the expensive
    /// naive implementation
    ///
    /// Note this returns None if an overflow occured
    ///
    #[inline]
    pub fn checked_mul(self, other: __p) -> Option<__p> {
        match self.overflowing_mul(other) {
            (_, true ) => None,
            (x, false) => Some(x),
        }
    }

    /// Polynomial multiplication
    ///
    /// This attempts to use carry-less multiplication
    /// instructions when available (pclmulqdq on x86_64,
    /// pmul on aarch64), otherwise falls back to the expensive
    /// naive implementation
    ///
    /// Note this wraps around the boundary of the type
    ///
    #[inline]
    pub fn wrapping_mul(self, other: __p) -> __p {
        cfg_if! {
            if #[cfg(all(target_arch="x86_64", target_feature="pclmulqdq"))] {
                cfg_if! {
                    if #[cfg(__if(__width <= 64))] {
                        unsafe {
                            // x86_64 provides 64-bit xmul via the pclmulqdq instruction
                            use core::arch::x86_64::*;
                            let a = _mm_set_epi64x(0, self.0 as i64);
                            let b = _mm_set_epi64x(0, other.0 as i64);
                            let x = _mm_clmulepi64_si128::<0>(a, b);
                            __p(_mm_extract_epi64::<0>(x) as __u)
                        }
                    } else {
                        unsafe {
                            // x86_64 provides 64-bit xmul via the pclmulqdq instruction
                            use core::arch::x86_64::*;
                            let a = _mm_set_epi64x((self.0 >> 64) as i64, self.0 as i64);
                            let b = _mm_set_epi64x((other.0 >> 64) as i64, other.0 as i64);
                            let x0 = _mm_clmulepi64_si128::<0>(a, b);
                            let x1 = _mm_slli_si128::<64>(_mm_clmulepi64_si128::<1>(a, b));
                            let x2 = _mm_slli_si128::<64>(_mm_clmulepi64_si128::<4>(a, b));
                            let x = _mm_xor_si128(x0, _mm_xor_si128(x1, x2));
                            let x0 = _mm_extract_epi64::<0>(x);
                            let x1 = _mm_extract_epi64::<1>(x);
                            __p(((x1 as u128) << 64) | (x0 as u128))
                        }
                    }
                }
            } else if #[cfg(all(target_arch="aarch64", target_feature="neon,crypto"))] {
                // TODO does this work on aarch64?
                cfg_if! {
                    if #[cfg(__if(__width <= 64))] {
                        unsafe {
                            // aarch64 provides 64-bit xmul via the pmul instruction
                            use core::arch::aarch64::*;
                            let a = self.0 as u64;
                            let b = other.0 as u64;
                            __p(vmull_p64(a, b) as __u)
                        }
                    } else {
                        unsafe {
                            // aarch64 provides 64-bit xmul via the pmul instruction
                            use core::arch::x86_64::*;
                            let a0 = self.0 as u64;
                            let a1 = (self.0 >> 64) as u64;
                            let b0 = other.0 as u64;
                            let b1 = (other.0 >> 64) as u64;
                            let x0 = vmull_p64(a0, b0);
                            let x1 = vmull_p64(a1, b0);
                            let x2 = vmull_p64(a0, b1);
                            __p(x0 ^ (x1 << 64) ^ (x2 << 64))
                        }
                    }
                }
            } else {
                self.naive_mul(other)
            }
        }
    }

    /// Polynomial multiplication
    ///
    /// This attempts to use carry-less multiplication
    /// instructions when available (pclmulqdq on x86_64,
    /// pmul on aarch64), otherwise falls back to the expensive
    /// naive implementation
    ///
    /// Note this panics if an overflow occured and debug_assertions
    /// are enabled
    ///
    #[inline]
    pub fn mul(self, other: __p) -> __p {
        cfg_if! {
            // TODO feature flag for overflow-checks?
            if #[cfg(debug_assertions)] {
                self.checked_mul(other)
                    .expect("overflow in polynomial multiply")
            } else {
                self.wrapping_mul(other)
            }
        }
    }

    /// Naive polynomial division
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts
    ///
    #[inline]
    pub const fn checked_naive_div(self, other: __p) -> Option<__p> {
        if other.0 == 0 {
            None
        } else {
            // TODO should this be constant-time?
            let mut a = self.0;
            let b = other.0;
            let mut x = 0;
            while a.leading_zeros() <= b.leading_zeros() {
                x ^= 1 << (b.leading_zeros()-a.leading_zeros());
                a ^= b << (b.leading_zeros()-a.leading_zeros());
            }
            Some(__p(x))
        }
    }

    /// Naive polynomial division
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts
    ///
    /// This will panis if b == 0
    ///
    #[inline]
    pub const fn naive_div(self, other: __p) -> __p {
        match self.checked_naive_div(other) {
            Some(x) => x,
            None => __p(self.0 / 0),
        }
    }

    /// Polynomial division
    ///
    /// Note, this is always expensive. There isn't much hardware for
    /// polynomial division, so we need to always use the naive implementation
    ///
    #[inline]
    pub fn checked_div(self, other: __p) -> Option<__p> {
        self.checked_naive_div(other)
    }

    /// Polynomial division
    ///
    /// Note, this is always expensive. There isn't much hardware for
    /// polynomial division, so we need to always use the naive implementation
    ///
    /// This will panis if b == 0
    ///
    #[inline]
    pub fn div(self, other: __p) -> __p {
        self.checked_div(other).expect("polynomial division by zero")
    }

    /// Naive polynomial remainder
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts
    ///
    #[inline]
    pub const fn checked_naive_rem(self, other: __p) -> Option<__p> {
        if other.0 == 0 {
            None
        } else {
            // TODO should this be constant-time?
            let mut a = self.0;
            let b = other.0;
            let mut x = 0;
            while a.leading_zeros() <= b.leading_zeros() {
                a ^= b << (b.leading_zeros()-a.leading_zeros());
            }
            Some(__p(a))
        }
    }

    /// Naive polynomial remainder
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts
    ///
    /// This will panis if b == 0
    ///
    #[inline]
    pub const fn naive_rem(self, other: __p) -> __p {
        match self.checked_naive_rem(other) {
            Some(x) => x,
            None => __p(self.0 / 0),
        }
    }

    /// Polynomial remainder
    ///
    /// Note, this is always expensive. There isn't much hardware for
    /// polynomial division, so we need to always use the naive implementation
    ///
    #[inline]
    pub fn checked_rem(self, other: __p) -> Option<__p> {
        self.checked_naive_rem(other)
    }

    /// Polynomial remainder
    ///
    /// Note, this is always expensive. There isn't much hardware for
    /// polynomial division, so we need to always use the naive implementation
    ///
    /// This will panis if b == 0
    ///
    #[inline]
    pub fn rem(self, other: __p) -> __p {
        self.checked_rem(other).expect("polynomial division by zero")
    }
}

//// Conversions into __p ////

impl From<__u> for __p {
    #[inline]
    fn from(x: __u) -> __p {
        __p(x)
    }
}

//// Conversions from __p ////

impl From<__p> for __u {
    #[inline]
    fn from(x: __p) -> __u {
        x.0
    }
}

//// Addition ////

impl Add<__p> for __p {
    type Output = __p;

    /// Polynomial addition, aka xor
    #[inline]
    fn add(self, other: __p) -> __p {
        __p::add(self, other)
    }
}

impl<'a> Add<__p> for &'a __p {
    type Output = __p;

    /// Polynomial addition, aka xor
    #[inline]
    fn add(self, other: __p) -> __p {
        __p::add(*self, other)
    }
}

impl<'b> Add<&'b __p> for __p {
    type Output = __p;

    /// Polynomial addition, aka xor
    #[inline]
    fn add(self, other: &'b __p) -> __p {
        __p::add(self, *other)
    }
}

impl<'a, 'b> Add<&'b __p> for &'a __p {
    type Output = __p;

    /// Polynomial addition, aka xor
    #[inline]
    fn add(self, other: &'b __p) -> __p {
        __p::add(*self, *other)
    }
}

impl AddAssign<__p> for __p {
    #[inline]
    fn add_assign(&mut self, other: __p) {
        *self = self.add(other)
    }
}

impl<'b> AddAssign<&'b __p> for __p {
    #[inline]
    fn add_assign(&mut self, other: &'b __p) {
        *self = self.add(*other)
    }
}

impl Sum<__p> for __p {
    #[inline]
    fn sum<I>(iter: I) -> __p
    where
        I: Iterator<Item=__p>
    {
        iter.fold(__p(0), |a, x| a + x)
    }
}

impl<'a> Sum<&'a __p> for __p {
    #[inline]
    fn sum<I>(iter: I) -> __p
    where
        I: Iterator<Item=&'a __p>
    {
        iter.fold(__p(0), |a, x| a + *x)
    }
}


//// Subtraction ////

impl Sub for __p {
    type Output = __p;

    /// Polynomial subtraction, aka xor
    #[inline]
    fn sub(self, other: __p) -> __p {
        __p::sub(self, other)
    }
}

impl<'a> Sub<__p> for &'a __p {
    type Output = __p;

    /// Polynomial subtraction, aka xor
    #[inline]
    fn sub(self, other: __p) -> __p {
        __p::sub(*self, other)
    }
}

impl<'b> Sub<&'b __p> for __p {
    type Output = __p;

    /// Polynomial subtraction, aka xor
    #[inline]
    fn sub(self, other: &'b __p) -> __p {
        __p::sub(self, *other)
    }
}

impl<'a, 'b> Sub<&'b __p> for &'a __p {
    type Output = __p;

    /// Polynomial subtraction, aka xor
    #[inline]
    fn sub(self, other: &'b __p) -> __p {
        __p::sub(*self, *other)
    }
}

impl SubAssign<__p> for __p {
    #[inline]
    fn sub_assign(&mut self, other: __p) {
        *self = self.sub(other)
    }
}

impl<'b> SubAssign<&'b __p> for __p {
    #[inline]
    fn sub_assign(&mut self, other: &'b __p) {
        *self = self.sub(*other)
    }
}


//// Multiplication ////

impl Mul for __p {
    type Output = __p;

    /// Polynomial multiplication
    ///
    /// This attempts to use carry-less multiplication
    /// instructions when available (pclmulqdq on x86_64,
    /// pmul on aarch64), otherwise falls back to the expensive
    /// naive implementation
    ///
    #[inline]
    fn mul(self, other: __p) -> __p {
        __p::mul(self, other)
    }
}

impl<'a> Mul<__p> for &'a __p {
    type Output = __p;

    /// Polynomial multiplication
    ///
    /// This attempts to use carry-less multiplication
    /// instructions when available (pclmulqdq on x86_64,
    /// pmul on aarch64), otherwise falls back to the expensive
    /// naive implementation
    ///
    #[inline]
    fn mul(self, other: __p) -> __p {
        __p::mul(*self, other)
    }
}

impl<'b> Mul<&'b __p> for __p {
    type Output = __p;

    /// Polynomial multiplication
    ///
    /// This attempts to use carry-less multiplication
    /// instructions when available (pclmulqdq on x86_64,
    /// pmul on aarch64), otherwise falls back to the expensive
    /// naive implementation
    ///
    #[inline]
    fn mul(self, other: &'b __p) -> __p {
        __p::mul(self, *other)
    }
}

impl<'a, 'b> Mul<&'b __p> for &'a __p {
    type Output = __p;

    /// Polynomial multiplication
    ///
    /// This attempts to use carry-less multiplication
    /// instructions when available (pclmulqdq on x86_64,
    /// pmul on aarch64), otherwise falls back to the expensive
    /// naive implementation
    ///
    #[inline]
    fn mul(self, other: &'b __p) -> __p {
        __p::mul(*self, *other)
    }
}

impl MulAssign<__p> for __p {
    #[inline]
    fn mul_assign(&mut self, other: __p) {
        *self = self.mul(other)
    }
}

impl<'b> MulAssign<&'b __p> for __p {
    #[inline]
    fn mul_assign(&mut self, other: &'b __p) {
        *self = self.mul(*other)
    }
}

impl Product<__p> for __p {
    #[inline]
    fn product<I>(iter: I) -> __p
    where
        I: Iterator<Item=__p>
    {
        iter.fold(__p(0), |a, x| a * x)
    }
}

impl<'a> Product<&'a __p> for __p {
    #[inline]
    fn product<I>(iter: I) -> __p
    where
        I: Iterator<Item=&'a __p>
    {
        iter.fold(__p(0), |a, x| a * *x)
    }
}

//// Division ////

impl Div for __p {
    type Output = __p;

    /// Polynomial division
    ///
    /// Note, this is always expensive. There isn't much hardware for
    /// polynomial division, so we need to always use the naive implementation
    ///
    /// This will panis if b == 0
    ///
    #[inline]
    fn div(self, other: __p) -> __p {
        __p::div(self, other)
    }
}

impl<'a> Div<__p> for &'a __p {
    type Output = __p;

    /// Polynomial division
    ///
    /// Note, this is always expensive. There isn't much hardware for
    /// polynomial division, so we need to always use the naive implementation
    ///
    /// This will panis if b == 0
    ///
    #[inline]
    fn div(self, other: __p) -> __p {
        __p::div(*self, other)
    }
}

impl<'b> Div<&'b __p> for __p {
    type Output = __p;

    /// Polynomial division
    ///
    /// Note, this is always expensive. There isn't much hardware for
    /// polynomial division, so we need to always use the naive implementation
    ///
    /// This will panis if b == 0
    ///
    #[inline]
    fn div(self, other: &'b __p) -> __p {
        __p::div(self, *other)
    }
}

impl<'a, 'b> Div<&'b __p> for &'a __p {
    type Output = __p;

    /// Polynomial division
    ///
    /// Note, this is always expensive. There isn't much hardware for
    /// polynomial division, so we need to always use the naive implementation
    ///
    /// This will panis if b == 0
    ///
    #[inline]
    fn div(self, other: &'b __p) -> __p {
        __p::div(*self, *other)
    }
}

impl DivAssign<__p> for __p {
    #[inline]
    fn div_assign(&mut self, other: __p) {
        *self = self.div(other)
    }
}

impl<'b> DivAssign<&'b __p> for __p {
    #[inline]
    fn div_assign(&mut self, other: &'b __p) {
        *self = self.div(*other)
    }
}

//// Remainder ////

impl Rem for __p {
    type Output = __p;

    /// Polynomial remainder
    ///
    /// Note, this is always expensive. There isn't much hardware for
    /// polynomial division, so we need to always use the naive implementation
    ///
    /// This will panis if b == 0
    ///
    #[inline]
    fn rem(self, other: __p) -> __p {
        __p::rem(self, other)
    }
}

impl<'a> Rem<__p> for &'a __p {
    type Output = __p;

    /// Polynomial remainder
    ///
    /// Note, this is always expensive. There isn't much hardware for
    /// polynomial division, so we need to always use the naive implementation
    ///
    /// This will panis if b == 0
    ///
    #[inline]
    fn rem(self, other: __p) -> __p {
        __p::rem(*self, other)
    }
}

impl<'b> Rem<&'b __p> for __p {
    type Output = __p;

    /// Polynomial remainder
    ///
    /// Note, this is always expensive. There isn't much hardware for
    /// polynomial division, so we need to always use the naive implementation
    ///
    /// This will panis if b == 0
    ///
    #[inline]
    fn rem(self, other: &'b __p) -> __p {
        __p::rem(self, *other)
    }
}

impl<'a, 'b> Rem<&'b __p> for &'a __p {
    type Output = __p;

    /// Polynomial remainder
    ///
    /// Note, this is always expensive. There isn't much hardware for
    /// polynomial division, so we need to always use the naive implementation
    ///
    /// This will panis if b == 0
    ///
    #[inline]
    fn rem(self, other: &'b __p) -> __p {
        __p::rem(*self, *other)
    }
}

impl RemAssign<__p> for __p {
    #[inline]
    fn rem_assign(&mut self, other: __p) {
        *self = self.rem(other)
    }
}

impl<'b> RemAssign<&'b __p> for __p {
    #[inline]
    fn rem_assign(&mut self, other: &'b __p) {
        *self = self.rem(*other)
    }
}
