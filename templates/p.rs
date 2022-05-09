///! Template for polynomial types

use core::mem::size_of;
use core::ops::*;
use core::iter::*;
use core::num::TryFromIntError;
use core::num::ParseIntError;
use core::fmt;
use core::str::FromStr;
use core::slice;
use __crate::traits::TryFrom;
use __crate::traits::FromLossy;
use __crate::internal::cfg_if::cfg_if;


/// A type representing a gf(2) polynomial.
///
/// ``` rust
/// use ::gf256::*;
///
/// let a = p32(0x1234);
/// let b = p32(0x5678);
/// assert_eq!(a+b, p32(0x444c));
/// assert_eq!(a*b, p32(0x05c58160));
/// ```
///
/// See the [module-level documentation](../p) for more info.
///
#[allow(non_camel_case_types)]
#[derive(Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct __p(pub __u);

impl __p {
    /// Create a gf(2) polynomial.
    #[inline]
    pub const fn new(x: __u) -> __p {
        __p(x)
    }

    /// Get the underlying primitive type.
    #[inline]
    pub const fn get(self) -> __u {
        self.0
    }

    /// Polynomial addition, aka xor.
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// const X: p8 = p8(0x12).naive_add(p8(0x34));
    /// assert_eq!(X, p8(0x26));
    /// ```
    ///
    #[inline]
    pub const fn naive_add(self, other: __p) -> __p {
        __p(self.0 ^ other.0)
    }

    /// Polynomial addition, aka xor.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// assert_eq!(p8(0x12) + p8(0x34), p8(0x26));
    /// ```
    ///
    #[inline]
    pub fn add(self, other: __p) -> __p {
        __p(self.0 ^ other.0)
    }

    /// Polynomial subtraction, aka xor.
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// const X: p8 = p8(0x12).naive_sub(p8(0x34));
    /// assert_eq!(X, p8(0x26));
    /// ```
    ///
    #[inline]
    pub const fn naive_sub(self, other: __p) -> __p {
        __p(self.0 ^ other.0)
    }

    /// Polynomial subtraction, aka xor.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// assert_eq!(p8(0x12) - p8(0x34), p8(0x26));
    /// ```
    ///
    #[inline]
    pub fn sub(self, other: __p) -> __p {
        __p(self.0 ^ other.0)
    }

    /// Naive polynomial multiplication.
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts.
    ///
    /// This returns a tuple containing the low and high parts in that order.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// const X: (p8, p8) = p8(0x02).naive_widening_mul(p8(0x34));
    /// const Y: (p8, p8) = p8(0x12).naive_widening_mul(p8(0x34));
    /// assert_eq!(X, (p8(0x68), p8(0x00)));
    /// assert_eq!(Y, (p8(0x28), p8(0x03)));
    /// ```
    ///
    #[inline]
    pub const fn naive_widening_mul(self, other: __p) -> (__p, __p) {
        let a = self.0;
        let b = other.0;
        let mut lo = 0;
        let mut hi = 0;
        let mut i = 0;
        while i < __width {
            let mask = (((a as __i) << (__width-1-i)) >> (__width-1)) as __u;
            lo ^= mask & (b << i);
            hi ^= mask & (b >> (__width-1-i));
            i += 1;
        }
        // note we adjust hi by one here, otherwise we'd need to handle
        // shifting > word size
        (__p(lo), __p(hi >> 1))
    }

    /// Naive polynomial multiplication.
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts.
    ///
    /// Note this wraps around the boundary of the type, and returns
    /// a flag indicating of overflow occured.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// const X: (p8, bool) = p8(0x02).naive_overflowing_mul(p8(0x34));
    /// const Y: (p8, bool) = p8(0x12).naive_overflowing_mul(p8(0x34));
    /// assert_eq!(X, (p8(0x68), false));
    /// assert_eq!(Y, (p8(0x28), true));
    /// ```
    ///
    #[inline]
    pub const fn naive_overflowing_mul(self, other: __p) -> (__p, bool) {
        let (lo, hi) = self.naive_widening_mul(other);
        (lo, hi.0 != 0)
    }

    /// Naive polynomial multiplication.
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts.
    ///
    /// Note this returns [`None`] if an overflow occured.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// const X: Option<p8> = p8(0x02).naive_checked_mul(p8(0x34));
    /// const Y: Option<p8> = p8(0x12).naive_checked_mul(p8(0x34));
    /// assert_eq!(X, Some(p8(0x68)));
    /// assert_eq!(Y, None);
    /// ```
    ///
    #[inline]
    pub const fn naive_checked_mul(self, other: __p) -> Option<__p> {
        match self.naive_overflowing_mul(other) {
            (_, true ) => None,
            (x, false) => Some(x),
        }
    }

    /// Naive polynomial multiplication.
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts.
    ///
    /// Note this wraps around the boundary of the type.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// const X: p8 = p8(0x02).naive_wrapping_mul(p8(0x34));
    /// const Y: p8 = p8(0x12).naive_wrapping_mul(p8(0x34));
    /// assert_eq!(X, p8(0x68));
    /// assert_eq!(Y, p8(0x28));
    /// ```
    ///
    #[inline]
    pub const fn naive_wrapping_mul(self, other: __p) -> __p {
        let a = self.0;
        let b = other.0;
        let mut x = 0;
        let mut i = 0;
        while i < __width {
            let mask = (((a as __i) << (__width-1-i)) >> (__width-1)) as __u;
            x ^= mask & (b << i);
            i += 1;
        }
        __p(x)
    }

    /// Naive polynomial multiplication.
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts.
    ///
    /// Note this panics if an overflow occured and debug_assertions
    /// are enabled.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// const X: p8 = p8(0x02).naive_wrapping_mul(p8(0x34));
    /// assert_eq!(X, p8(0x68));
    /// ```
    ///
    #[inline]
    pub const fn naive_mul(self, other: __p) -> __p {
        cfg_if! {
            // TODO feature flag for overflow-checks?
            if #[cfg(debug_assertions)] {
                match self.naive_checked_mul(other) {
                    Some(x) => x,
                    None => __p(self.0 / 0),
                }
            } else {
                self.naive_wrapping_mul(other)
            }
        }
    }

    /// Naive polynomial multiplication.
    ///
    /// This attempts to use carry-less multiplication instructions when
    /// available (`pclmulqdq` on x86_64, `pmull` on aarch64), otherwise falls
    /// back to the expensive naive implementation.
    ///
    /// This return a tuple containing the low and high parts in that order.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// assert_eq!(p8(0x02).widening_mul(p8(0x34)), (p8(0x68), p8(0x00)));
    /// assert_eq!(p8(0x12).widening_mul(p8(0x34)), (p8(0x28), p8(0x03)));
    /// ```
    ///
    #[inline]
    pub fn widening_mul(self, other: __p) -> (__p, __p) {
        cfg_if! {
            if #[cfg(__if(__has_xmul))] {
                let (lo, hi) = __xmul(self.0 as _, other.0 as _);
                (__p(lo as __u), __p(hi as __u))
            } else {
                self.naive_widening_mul(other)
            }
        }
    }

    /// Polynomial multiplication.
    ///
    /// This attempts to use carry-less multiplication instructions when
    /// available (`pclmulqdq` on x86_64, `pmull` on aarch64), otherwise falls
    /// back to the expensive naive implementation.
    ///
    /// Note this wraps around the boundary of the type, and returns
    /// a flag indicating of overflow occured.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// assert_eq!(p8(0x02).overflowing_mul(p8(0x34)), (p8(0x68), false));
    /// assert_eq!(p8(0x12).overflowing_mul(p8(0x34)), (p8(0x28), true));
    /// ```
    ///
    #[inline]
    pub fn overflowing_mul(self, other: __p) -> (__p, bool) {
        let (lo, hi) = self.widening_mul(other);
        (lo, hi.0 != 0)
    }

    /// Polynomial multiplication.
    ///
    /// This attempts to use carry-less multiplication instructions when
    /// available (`pclmulqdq` on x86_64, `pmull` on aarch64), otherwise falls
    /// back to the expensive naive implementation.
    ///
    /// Note this returns [`None`] if an overflow occured.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// assert_eq!(p8(0x02).checked_mul(p8(0x34)), Some(p8(0x68)));
    /// assert_eq!(p8(0x12).checked_mul(p8(0x34)), None);
    /// ```
    ///
    #[inline]
    pub fn checked_mul(self, other: __p) -> Option<__p> {
        match self.overflowing_mul(other) {
            (_, true ) => None,
            (x, false) => Some(x),
        }
    }

    /// Polynomial multiplication.
    ///
    /// This attempts to use carry-less multiplication instructions when
    /// available (`pclmulqdq` on x86_64, `pmull` on aarch64), otherwise falls
    /// back to the expensive naive implementation.
    ///
    /// Note this wraps around the boundary of the type.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// assert_eq!(p8(0x02).wrapping_mul(p8(0x34)), p8(0x68));
    /// assert_eq!(p8(0x12).wrapping_mul(p8(0x34)), p8(0x28));
    /// ```
    ///
    #[inline]
    pub fn wrapping_mul(self, other: __p) -> __p {
        cfg_if! {
            if #[cfg(__if(__has_xmul))] {
                __p(__xmul(self.0 as _, other.0 as _).0 as __u)
            } else {
                self.naive_wrapping_mul(other)
            }
        }
    }

    /// Polynomial multiplication.
    ///
    /// This attempts to use carry-less multiplication instructions when
    /// available (`pclmulqdq` on x86_64, `pmull` on aarch64), otherwise falls
    /// back to the expensive naive implementation.
    ///
    /// Note this panics if an overflow occured and debug_assertions
    /// are enabled.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// assert_eq!(p8(0x02) * p8(0x34), p8(0x68));
    /// ```
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

    /// Naive polynomial exponentiation.
    ///
    /// Performs exponentiation by squaring, where polynomial exponentiation
    /// is the same as repeated multiplication.
    ///
    /// Note this wraps around the boundary of the type, and returns
    /// a flag indicating of overflow occured.
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts.
    /// 
    /// ``` rust
    /// # use ::gf256::*;
    /// const X: (p8, bool) = p8(0x02).naive_overflowing_pow(3);
    /// const Y: (p8, bool) = p8(0x12).naive_overflowing_pow(3);
    /// assert_eq!(X, (p8(0x02)*p8(0x02)*p8(0x02), false));
    /// assert_eq!(X, (p8(0x08), false));
    /// assert_eq!(Y, (p8(0x48), true));
    /// ```
    ///
    #[inline]
    pub const fn naive_overflowing_pow(self, exp: u32) -> (__p, bool) {
        let mut a = self;
        let mut exp = exp;
        let mut x = __p(1);
        let mut o = false;
        loop {
            if exp & 1 != 0 {
                let (x_, o_) = x.naive_overflowing_mul(a);
                x = x_;
                o = o || o_;
            }

            exp >>= 1;
            if exp == 0 {
                return (x, o);
            }
            let (a_, o_) = a.naive_overflowing_mul(a);
            a = a_;
            o = o || o_;
        }
    }

    /// Naive polynomial exponentiation.
    ///
    /// Performs exponentiation by squaring, where polynomial exponentiation
    /// is the same as repeated multiplication.
    ///
    /// Note this returns [`None`] if an overflow occured.
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts.
    /// 
    /// ``` rust
    /// # use ::gf256::*;
    /// const X: Option<p8> = p8(0x02).naive_checked_pow(3);
    /// const Y: Option<p8> = p8(0x12).naive_checked_pow(3);
    /// assert_eq!(X, Some(p8(0x02)*p8(0x02)*p8(0x02)));
    /// assert_eq!(X, Some(p8(0x08)));
    /// assert_eq!(Y, None);
    /// ```
    ///
    #[inline]
    pub const fn naive_checked_pow(self, exp: u32) -> Option<__p> {
        let mut a = self;
        let mut exp = exp;
        let mut x = __p(1);
        loop {
            if exp & 1 != 0 {
                x = match x.naive_checked_mul(a) {
                    Some(x) => x,
                    None => return None,
                }
            }

            exp >>= 1;
            if exp == 0 {
                return Some(x);
            }
            a = match a.naive_checked_mul(a) {
                Some(a) => a,
                None => return None,
            }
        }
    }

    /// Naive polynomial exponentiation.
    ///
    /// Performs exponentiation by squaring, where polynomial exponentiation
    /// is the same as repeated multiplication.
    ///
    /// Note this wraps around the boundary of the type.
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts.
    /// 
    /// ``` rust
    /// # use ::gf256::*;
    /// const X: p8 = p8(0x02).naive_wrapping_pow(3);
    /// const Y: p8 = p8(0x12).naive_wrapping_pow(3);
    /// assert_eq!(X, p8(0x02)*p8(0x02)*p8(0x02));
    /// assert_eq!(X, p8(0x08));
    /// assert_eq!(Y, p8(0x48));
    /// ```
    ///
    #[inline]
    pub const fn naive_wrapping_pow(self, exp: u32) -> __p {
        let mut a = self;
        let mut exp = exp;
        let mut x = __p(1);
        loop {
            if exp & 1 != 0 {
                x = x.naive_wrapping_mul(a);
            }

            exp >>= 1;
            if exp == 0 {
                return x;
            }
            a = a.naive_wrapping_mul(a);
        }
    }

    /// Naive polynomial exponentiation.
    ///
    /// Performs exponentiation by squaring, where polynomial exponentiation
    /// is the same as repeated multiplication.
    ///
    /// Note this panics if an overflow occured and debug_assertions
    /// are enabled.
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts.
    /// 
    /// ``` rust
    /// # use ::gf256::*;
    /// const X: p8 = p8(0x02).naive_pow(3);
    /// assert_eq!(X, p8(0x02)*p8(0x02)*p8(0x02));
    /// assert_eq!(X, p8(0x08));
    /// ```
    ///
    #[inline]
    pub const fn naive_pow(self, exp: u32) -> __p {
        let mut a = self;
        let mut exp = exp;
        let mut x = __p(1);
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

    /// Polynomial exponentiation.
    ///
    /// Performs exponentiation by squaring, where polynomial exponentiation
    /// is the same as repeated multiplication.
    ///
    /// Note this wraps around the boundary of the type, and returns
    /// a flag indicating of overflow occured.
    /// 
    /// ``` rust
    /// # use ::gf256::*;
    /// assert_eq!(p8(0x02).overflowing_pow(3), (p8(0x02)*p8(0x02)*p8(0x02), false));
    /// assert_eq!(p8(0x02).overflowing_pow(3), (p8(0x08), false));
    /// assert_eq!(p8(0x12).overflowing_pow(3), (p8(0x48), true));
    /// ```
    ///
    #[inline]
    pub fn overflowing_pow(self, exp: u32) -> (__p, bool) {
        let mut a = self;
        let mut exp = exp;
        let mut x = __p(1);
        let mut o = false;
        loop {
            if exp & 1 != 0 {
                let (x_, o_) = x.overflowing_mul(a);
                x = x_;
                o = o || o_;
            }

            exp >>= 1;
            if exp == 0 {
                return (x, o);
            }
            let (a_, o_) = a.overflowing_mul(a);
            a = a_;
            o = o || o_;
        }
    }

    /// Polynomial exponentiation.
    ///
    /// Performs exponentiation by squaring, where polynomial exponentiation
    /// is the same as repeated multiplication.
    ///
    /// Note this returns [`None`] if an overflow occured.
    /// 
    /// ``` rust
    /// # use ::gf256::*;
    /// assert_eq!(p8(0x02).checked_pow(3), Some(p8(0x02)*p8(0x02)*p8(0x02)));
    /// assert_eq!(p8(0x02).checked_pow(3), Some(p8(0x08)));
    /// assert_eq!(p8(0x12).checked_pow(3), None);
    /// ```
    ///
    #[inline]
    pub fn checked_pow(self, exp: u32) -> Option<__p> {
        let mut a = self;
        let mut exp = exp;
        let mut x = __p(1);
        loop {
            if exp & 1 != 0 {
                x = match x.checked_mul(a) {
                    Some(x) => x,
                    None => return None,
                }
            }

            exp >>= 1;
            if exp == 0 {
                return Some(x);
            }
            a = match a.checked_mul(a) {
                Some(a) => a,
                None => return None,
            }
        }
    }

    /// Polynomial exponentiation.
    ///
    /// Performs exponentiation by squaring, where polynomial exponentiation
    /// is the same as repeated multiplication.
    ///
    /// Note this wraps around the boundary of the type.
    /// 
    /// ``` rust
    /// # use ::gf256::*;
    /// assert_eq!(p8(0x02).wrapping_pow(3), p8(0x02)*p8(0x02)*p8(0x02));
    /// assert_eq!(p8(0x02).wrapping_pow(3), p8(0x08));
    /// assert_eq!(p8(0x12).wrapping_pow(3), p8(0x48));
    /// ```
    ///
    #[inline]
    pub fn wrapping_pow(self, exp: u32) -> __p {
        let mut a = self;
        let mut exp = exp;
        let mut x = __p(1);
        loop {
            if exp & 1 != 0 {
                x = x.wrapping_mul(a);
            }

            exp >>= 1;
            if exp == 0 {
                return x;
            }
            a = a.wrapping_mul(a);
        }
    }

    /// Polynomial exponentiation.
    ///
    /// Performs exponentiation by squaring, where polynomial exponentiation
    /// is the same as repeated multiplication.
    ///
    /// Note this panics if an overflow occured and debug_assertions
    /// are enabled.
    /// 
    /// ``` rust
    /// # use ::gf256::*;
    /// assert_eq!(p8(0x02).pow(3), p8(0x02)*p8(0x02)*p8(0x02));
    /// assert_eq!(p8(0x02).pow(3), p8(0x08));
    /// ```
    ///
    #[inline]
    pub fn pow(self, exp: u32) -> __p {
        let mut a = self;
        let mut exp = exp;
        let mut x = __p(1);
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

    /// Naive polynomial division.
    ///
    /// Note there is rarely hardware support for polynomial division,
    /// so these always use relatively expensive bitwise operations.
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts.
    ///
    /// Returns [`None`] if `other == 0`.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// const X: Option<p8> = p8(0x68).naive_checked_div(p8(0x34));
    /// const Y: Option<p8> = p8(0x68).naive_checked_div(p8(0x00));
    /// assert_eq!(X, Some(p8(0x02)));
    /// assert_eq!(Y, None);
    /// ```
    ///
    #[inline]
    pub const fn naive_checked_div(self, other: __p) -> Option<__p> {
        if other.0 == 0 {
            None
        } else {
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

    /// Naive polynomial division.
    ///
    /// Note there is rarely hardware support for polynomial division,
    /// so these always use relatively expensive bitwise operations.
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts.
    ///
    /// This will panic if `other == 0`.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// const X: p8 = p8(0x68).naive_div(p8(0x34));
    /// assert_eq!(X, p8(0x02));
    /// ```
    ///
    #[inline]
    pub const fn naive_div(self, other: __p) -> __p {
        match self.naive_checked_div(other) {
            Some(x) => x,
            None => __p(self.0 / 0),
        }
    }

    /// Naive polynomial remainder.
    ///
    /// Note there is rarely hardware support for polynomial remainder,
    /// so these always use relatively expensive bitwise operations.
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts.
    ///
    /// Returns [`None`] if `other == 0`.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// const X: Option<p8> = p8(0x69).naive_checked_rem(p8(0x34));
    /// const Y: Option<p8> = p8(0x69).naive_checked_rem(p8(0x00));
    /// assert_eq!(X, Some(p8(0x01)));
    /// assert_eq!(Y, None);
    /// ```
    ///
    #[inline]
    pub const fn naive_checked_rem(self, other: __p) -> Option<__p> {
        if other.0 == 0 {
            None
        } else {
            let mut a = self.0;
            let b = other.0;
            while a.leading_zeros() <= b.leading_zeros() {
                a ^= b << (b.leading_zeros()-a.leading_zeros());
            }
            Some(__p(a))
        }
    }

    /// Naive polynomial remainder.
    ///
    /// Note there is rarely hardware support for polynomial remainder,
    /// so these always use relatively expensive bitwise operations.
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts.
    ///
    /// This will panic if `other == 0`
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// const X: p8 = p8(0x69).naive_rem(p8(0x34));
    /// assert_eq!(X, p8(0x01));
    /// ```
    ///
    #[inline]
    pub const fn naive_rem(self, other: __p) -> __p {
        match self.naive_checked_rem(other) {
            Some(x) => x,
            None => __p(self.0 / 0),
        }
    }

    /// Cast slice of unsigned-types to slice of polynomial-types.
    ///
    /// This is useful for when you want to view an array of bytes
    /// as an array of polynomials without an additional memory allocation
    /// or unsafe code.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// let x: &[u8] = &[0x01, 0x02, 0x03, 0x04, 0x05];
    /// let y: &[p8] = p8::slice_from_slice(x);
    /// assert_eq!(y, &[p8(0x01), p8(0x02), p8(0x03), p8(0x04), p8(0x05)]);
    /// ```
    ///
    #[inline]
    pub fn slice_from_slice(slice: &[__u]) -> &[__p] {
        unsafe {
            slice::from_raw_parts(
                slice.as_ptr() as *const __p,
                slice.len()
            )
        }
    }

    /// Cast mut slice of unsigned-types to mut slice of polynomial-types
    ///
    /// This is useful for when you want to view an array of bytes
    /// as an array of polynomials without an additional memory allocation
    /// or unsafe code.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// let x: &mut [u8] = &mut [0x01, 0x02, 0x03, 0x04, 0x05];
    /// let y: &mut [p8] = p8::slice_from_slice_mut(x);
    /// for i in 0..y.len() {
    ///     y[i] *= p8(0x05);
    /// }
    /// assert_eq!(x, &[0x05, 0x0a, 0x0f, 0x14, 0x11]);
    /// ```
    ///
    #[inline]
    pub fn slice_from_slice_mut(slice: &mut [__u]) -> &mut [__p] {
        unsafe {
            slice::from_raw_parts_mut(
                slice.as_mut_ptr() as *mut __p,
                slice.len()
            )
        }
    }
}


//// Conversions into __p ////

impl From<__u> for __p {
    #[inline]
    fn from(x: __u) -> __p {
        __p(x)
    }
}

impl From<bool> for __p {
    #[inline]
    fn from(x: bool) -> __p {
        __p(__u::from(x))
    }
}

#[cfg(__if(__width >= 32 && !__is_usize))]
impl From<char> for __p {
    #[inline]
    fn from(x: char) -> __p {
        __p(__u::from(x))
    }
}

#[cfg(__if(__width > 8))]
impl From<u8> for __p {
    #[inline]
    fn from(x: u8) -> __p {
        __p(__u::from(x))
    }
}

#[cfg(__if(__width > 16))]
impl From<u16> for __p {
    #[inline]
    fn from(x: u16) -> __p {
        __p(__u::from(x))
    }
}

#[cfg(__if(__width > 32 && !__is_usize))]
impl From<u32> for __p {
    #[inline]
    fn from(x: u32) -> __p {
        __p(__u::from(x))
    }
}

#[cfg(__if(__width > 64 && !__is_usize))]
impl From<u64> for __p {
    #[inline]
    fn from(x: u64) -> __p {
        __p(__u::from(x))
    }
}

#[cfg(__if(__width > 8))]
impl From<__crate::p8> for __p {
    #[inline]
    fn from(x: __crate::p8) -> __p {
        __p(__u::from(x.0))
    }
}

#[cfg(__if(__width > 16))]
impl From<__crate::p16> for __p {
    #[inline]
    fn from(x: __crate::p16) -> __p {
        __p(__u::from(x.0))
    }
}

#[cfg(__if(__width > 32 && !__is_usize))]
impl From<__crate::p32> for __p {
    #[inline]
    fn from(x: __crate::p32) -> __p {
        __p(__u::from(x.0))
    }
}

#[cfg(__if(__width > 64 && !__is_usize))]
impl From<__crate::p64> for __p {
    #[inline]
    fn from(x: __crate::p64) -> __p {
        __p(__u::from(x.0))
    }
}

#[cfg(__if(__width < 8))]
impl TryFrom<u8> for __p {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: u8) -> Result<__p, Self::Error> {
        Ok(__p(__u::try_from(x)?))
    }
}

#[cfg(__if(__width < 16))]
impl TryFrom<u16> for __p {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: u16) -> Result<__p, Self::Error> {
        Ok(__p(__u::try_from(x)?))
    }
}

#[cfg(__if(__width < 32 || __is_usize))]
impl TryFrom<u32> for __p {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: u32) -> Result<__p, Self::Error> {
        Ok(__p(__u::try_from(x)?))
    }
}

#[cfg(__if(__width < 64 || __is_usize))]
impl TryFrom<u64> for __p {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: u64) -> Result<__p, Self::Error> {
        Ok(__p(__u::try_from(x)?))
    }
}

#[cfg(__if(__width < 128 || __is_usize))]
impl TryFrom<u128> for __p {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: u128) -> Result<__p, Self::Error> {
        Ok(__p(__u::try_from(x)?))
    }
}

#[cfg(__if(!__is_usize))]
impl TryFrom<usize> for __p {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: usize) -> Result<__p, Self::Error> {
        Ok(__p(__u::try_from(x)?))
    }
}

#[cfg(__if(__width < 8))]
impl TryFrom<__crate::p8> for __p {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __crate::p8) -> Result<__p, Self::Error> {
        Ok(__p(__u::try_from(x.0)?))
    }
}

#[cfg(__if(__width < 16))]
impl TryFrom<__crate::p16> for __p {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __crate::p16) -> Result<__p, Self::Error> {
        Ok(__p(__u::try_from(x.0)?))
    }
}

#[cfg(__if(__width < 32 || __is_usize))]
impl TryFrom<__crate::p32> for __p {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __crate::p32) -> Result<__p, Self::Error> {
        Ok(__p(__u::try_from(x.0)?))
    }
}

#[cfg(__if(__width < 64 || __is_usize))]
impl TryFrom<__crate::p64> for __p {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __crate::p64) -> Result<__p, Self::Error> {
        Ok(__p(__u::try_from(x.0)?))
    }
}

#[cfg(__if(__width < 128 || __is_usize))]
impl TryFrom<__crate::p128> for __p {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __crate::p128) -> Result<__p, Self::Error> {
        Ok(__p(__u::try_from(x.0)?))
    }
}

#[cfg(__if(!__is_usize))]
impl TryFrom<__crate::psize> for __p {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __crate::psize) -> Result<__p, Self::Error> {
        Ok(__p(__u::try_from(x.0)?))
    }
}

#[cfg(__if(__width < 8))]
impl FromLossy<u8> for __p {
    #[inline]
    fn from_lossy(x: u8) -> __p {
        __p(x as __u)
    }
}

#[cfg(__if(__width < 16))]
impl FromLossy<u16> for __p {
    #[inline]
    fn from_lossy(x: u16) -> __p {
        __p(x as __u)
    }
}

#[cfg(__if(__width < 32 || __is_usize))]
impl FromLossy<u32> for __p {
    #[inline]
    fn from_lossy(x: u32) -> __p {
        __p(x as __u)
    }
}

#[cfg(__if(__width < 64 || __is_usize))]
impl FromLossy<u64> for __p {
    #[inline]
    fn from_lossy(x: u64) -> __p {
        __p(x as __u)
    }
}

#[cfg(__if(__width < 128 || __is_usize))]
impl FromLossy<u128> for __p {
    #[inline]
    fn from_lossy(x: u128) -> __p {
        __p(x as __u)
    }
}

#[cfg(__if(!__is_usize))]
impl FromLossy<usize> for __p {
    #[inline]
    fn from_lossy(x: usize) -> __p {
        __p(x as __u)
    }
}

#[cfg(__if(__width < 8))]
impl FromLossy<__crate::p8> for __p {
    #[inline]
    fn from_lossy(x: __crate::p8) -> __p {
        __p(x.0 as __u)
    }
}

#[cfg(__if(__width < 16))]
impl FromLossy<__crate::p16> for __p {
    #[inline]
    fn from_lossy(x: __crate::p16) -> __p {
        __p(x.0 as __u)
    }
}

#[cfg(__if(__width < 32 || __is_usize))]
impl FromLossy<__crate::p32> for __p {
    #[inline]
    fn from_lossy(x: __crate::p32) -> __p {
        __p(x.0 as __u)
    }
}

#[cfg(__if(__width < 64 || __is_usize))]
impl FromLossy<__crate::p64> for __p {
    #[inline]
    fn from_lossy(x: __crate::p64) -> __p {
        __p(x.0 as __u)
    }
}

#[cfg(__if(__width < 128 || __is_usize))]
impl FromLossy<__crate::p128> for __p {
    #[inline]
    fn from_lossy(x: __crate::p128) -> __p {
        __p(x.0 as __u)
    }
}

#[cfg(__if(!__is_usize))]
impl FromLossy<__crate::psize> for __p {
    #[inline]
    fn from_lossy(x: __crate::psize) -> __p {
        __p(x.0 as __u)
    }
}

impl TryFrom<i8> for __p {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: i8) -> Result<__p, Self::Error> {
        Ok(__p(__u::try_from(x)?))
    }
}

impl TryFrom<i16> for __p {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: i16) -> Result<__p, Self::Error> {
        Ok(__p(__u::try_from(x)?))
    }
}

impl TryFrom<i32> for __p {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: i32) -> Result<__p, Self::Error> {
        Ok(__p(__u::try_from(x)?))
    }
}

impl TryFrom<i64> for __p {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: i64) -> Result<__p, Self::Error> {
        Ok(__p(__u::try_from(x)?))
    }
}

impl TryFrom<i128> for __p {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: i128) -> Result<__p, Self::Error> {
        Ok(__p(__u::try_from(x)?))
    }
}

impl TryFrom<isize> for __p {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: isize) -> Result<__p, Self::Error> {
        Ok(__p(__u::try_from(x)?))
    }
}

impl FromLossy<i8> for __p {
    #[inline]
    fn from_lossy(x: i8) -> __p {
        __p(x as __u)
    }
}

impl FromLossy<i16> for __p {
    #[inline]
    fn from_lossy(x: i16) -> __p {
        __p(x as __u)
    }
}

impl FromLossy<i32> for __p {
    #[inline]
    fn from_lossy(x: i32) -> __p {
        __p(x as __u)
    }
}

impl FromLossy<i64> for __p {
    #[inline]
    fn from_lossy(x: i64) -> __p {
        __p(x as __u)
    }
}

impl FromLossy<i128> for __p {
    #[inline]
    fn from_lossy(x: i128) -> __p {
        __p(x as __u)
    }
}

impl FromLossy<isize> for __p {
    #[inline]
    fn from_lossy(x: isize) -> __p {
        __p(x as __u)
    }
}


//// Conversions from __p ////

impl From<__p> for __u {
    #[inline]
    fn from(x: __p) -> __u {
        x.0
    }
}

#[cfg(__if(__width < 8))]
impl From<__p> for u8 {
    #[inline]
    fn from(x: __p) -> u8 {
        u8::from(x.0)
    }
}

#[cfg(__if(__width < 16))]
impl From<__p> for u16 {
    #[inline]
    fn from(x: __p) -> u16 {
        u16::from(x.0)
    }
}

#[cfg(__if(__width < 32 && !__is_usize))]
impl From<__p> for u32 {
    #[inline]
    fn from(x: __p) -> u32 {
        u32::from(x.0)
    }
}

#[cfg(__if(__width < 64 && !__is_usize))]
impl From<__p> for u64 {
    #[inline]
    fn from(x: __p) -> u64 {
        u64::from(x.0)
    }
}

#[cfg(__if(__width < 128 && !__is_usize))]
impl From<__p> for u128 {
    #[inline]
    fn from(x: __p) -> u128 {
        u128::from(x.0)
    }
}

#[cfg(__if(__width <= 16 && !__is_usize))]
impl From<__p> for usize {
    #[inline]
    fn from(x: __p) -> usize {
        usize::from(x.0)
    }
}

#[cfg(__if(__width > 8))]
impl TryFrom<__p> for u8 {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __p) -> Result<u8, Self::Error> {
        u8::try_from(x.0)
    }
}

#[cfg(__if(__width > 16))]
impl TryFrom<__p> for u16 {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __p) -> Result<u16, Self::Error> {
        u16::try_from(x.0)
    }
}

#[cfg(__if(__width > 32 || __is_usize))]
impl TryFrom<__p> for u32 {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __p) -> Result<u32, Self::Error> {
        u32::try_from(x.0)
    }
}

#[cfg(__if(__width > 64 || __is_usize))]
impl TryFrom<__p> for u64 {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __p) -> Result<u64, Self::Error> {
        u64::try_from(x.0)
    }
}

#[cfg(__if(__width > 16 && !__is_usize))]
impl TryFrom<__p> for usize {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __p) -> Result<usize, Self::Error> {
        usize::try_from(x.0)
    }
}

#[cfg(__if(__width > 8))]
impl FromLossy<__p> for u8 {
    #[inline]
    fn from_lossy(x: __p) -> u8 {
        x.0 as u8
    }
}

#[cfg(__if(__width > 16))]
impl FromLossy<__p> for u16 {
    #[inline]
    fn from_lossy(x: __p) -> u16 {
        x.0 as u16
    }
}

#[cfg(__if(__width > 32 || __is_usize))]
impl FromLossy<__p> for u32 {
    #[inline]
    fn from_lossy(x: __p) -> u32 {
        x.0 as u32
    }
}

#[cfg(__if(__width > 64 || __is_usize))]
impl FromLossy<__p> for u64 {
    #[inline]
    fn from_lossy(x: __p) -> u64 {
        x.0 as u64
    }
}

#[cfg(__if(__width > 16 && !__is_usize))]
impl FromLossy<__p> for usize {
    #[inline]
    fn from_lossy(x: __p) -> usize {
        x.0 as usize
    }
}

#[cfg(__if(__width < 8))]
impl From<__p> for i8 {
    #[inline]
    fn from(x: __p) -> i8 {
        i8::from(x.0)
    }
}

#[cfg(__if(__width < 16))]
impl From<__p> for i16 {
    #[inline]
    fn from(x: __p) -> i16 {
        i16::from(x.0)
    }
}

#[cfg(__if(__width < 32 && !__is_usize))]
impl From<__p> for i32 {
    #[inline]
    fn from(x: __p) -> i32 {
        i32::from(x.0)
    }
}

#[cfg(__if(__width < 64 && !__is_usize))]
impl From<__p> for i64 {
    #[inline]
    fn from(x: __p) -> i64 {
        i64::from(x.0)
    }
}

#[cfg(__if(__width < 128 && !__is_usize))]
impl From<__p> for i128 {
    #[inline]
    fn from(x: __p) -> i128 {
        i128::from(x.0)
    }
}

#[cfg(__if(__width < 16 && !__is_usize))]
impl From<__p> for isize {
    #[inline]
    fn from(x: __p) -> isize {
        isize::from(x.0)
    }
}

#[cfg(__if(__width >= 8))]
impl TryFrom<__p> for i8 {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __p) -> Result<i8, Self::Error> {
        i8::try_from(x.0)
    }
}

#[cfg(__if(__width >= 16))]
impl TryFrom<__p> for i16 {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __p) -> Result<i16, Self::Error> {
        i16::try_from(x.0)
    }
}

#[cfg(__if(__width >= 32 || __is_usize))]
impl TryFrom<__p> for i32 {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __p) -> Result<i32, Self::Error> {
        i32::try_from(x.0)
    }
}

#[cfg(__if(__width >= 64 || __is_usize))]
impl TryFrom<__p> for i64 {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __p) -> Result<i64, Self::Error> {
        i64::try_from(x.0)
    }
}

#[cfg(__if(__width >= 128 || __is_usize))]
impl TryFrom<__p> for i128 {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __p) -> Result<i128, Self::Error> {
        i128::try_from(x.0)
    }
}

#[cfg(__if(__width >= 16))]
impl TryFrom<__p> for isize {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __p) -> Result<isize, Self::Error> {
        isize::try_from(x.0)
    }
}

#[cfg(__if(__width >= 8))]
impl FromLossy<__p> for i8 {
    #[inline]
    fn from_lossy(x: __p) -> i8 {
        x.0 as i8
    }
}

#[cfg(__if(__width >= 16))]
impl FromLossy<__p> for i16 {
    #[inline]
    fn from_lossy(x: __p) -> i16 {
        x.0 as i16
    }
}

#[cfg(__if(__width >= 32 || __is_usize))]
impl FromLossy<__p> for i32 {
    #[inline]
    fn from_lossy(x: __p) -> i32 {
        x.0 as i32
    }
}

#[cfg(__if(__width >= 64 || __is_usize))]
impl FromLossy<__p> for i64 {
    #[inline]
    fn from_lossy(x: __p) -> i64 {
        x.0 as i64
    }
}

#[cfg(__if(__width >= 128 || __is_usize))]
impl FromLossy<__p> for i128 {
    #[inline]
    fn from_lossy(x: __p) -> i128 {
        x.0 as i128
    }
}

#[cfg(__if(__width >= 16))]
impl FromLossy<__p> for isize {
    #[inline]
    fn from_lossy(x: __p) -> isize {
        x.0 as isize
    }
}


//// Negate ////

impl Neg for __p {
    type Output = __p;
    // Negate is a noop for polynomials
    #[inline]
    fn neg(self) -> __p {
        self
    }
}

impl Neg for &__p {
    type Output = __p;
    // Negate is a noop for polynomials
    #[inline]
    fn neg(self) -> __p {
        *self
    }
}


//// Addition ////

impl Add<__p> for __p {
    type Output = __p;
    #[inline]
    fn add(self, other: __p) -> __p {
        __p::add(self, other)
    }
}

impl Add<__p> for &__p {
    type Output = __p;
    #[inline]
    fn add(self, other: __p) -> __p {
        __p::add(*self, other)
    }
}

impl Add<&__p> for __p {
    type Output = __p;
    #[inline]
    fn add(self, other: &__p) -> __p {
        __p::add(self, *other)
    }
}

impl Add<&__p> for &__p {
    type Output = __p;
    #[inline]
    fn add(self, other: &__p) -> __p {
        __p::add(*self, *other)
    }
}

impl AddAssign<__p> for __p {
    #[inline]
    fn add_assign(&mut self, other: __p) {
        *self = self.add(other)
    }
}

impl AddAssign<&__p> for __p {
    #[inline]
    fn add_assign(&mut self, other: &__p) {
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
    #[inline]
    fn sub(self, other: __p) -> __p {
        __p::sub(self, other)
    }
}

impl Sub<__p> for &__p {
    type Output = __p;
    #[inline]
    fn sub(self, other: __p) -> __p {
        __p::sub(*self, other)
    }
}

impl Sub<&__p> for __p {
    type Output = __p;
    #[inline]
    fn sub(self, other: &__p) -> __p {
        __p::sub(self, *other)
    }
}

impl Sub<&__p> for &__p {
    type Output = __p;
    #[inline]
    fn sub(self, other: &__p) -> __p {
        __p::sub(*self, *other)
    }
}

impl SubAssign<__p> for __p {
    #[inline]
    fn sub_assign(&mut self, other: __p) {
        *self = self.sub(other)
    }
}

impl SubAssign<&__p> for __p {
    #[inline]
    fn sub_assign(&mut self, other: &__p) {
        *self = self.sub(*other)
    }
}


//// Multiplication ////

impl Mul for __p {
    type Output = __p;
    #[inline]
    fn mul(self, other: __p) -> __p {
        __p::mul(self, other)
    }
}

impl Mul<__p> for &__p {
    type Output = __p;
    #[inline]
    fn mul(self, other: __p) -> __p {
        __p::mul(*self, other)
    }
}

impl Mul<&__p> for __p {
    type Output = __p;
    #[inline]
    fn mul(self, other: &__p) -> __p {
        __p::mul(self, *other)
    }
}

impl Mul<&__p> for &__p {
    type Output = __p;
    #[inline]
    fn mul(self, other: &__p) -> __p {
        __p::mul(*self, *other)
    }
}

impl MulAssign<__p> for __p {
    #[inline]
    fn mul_assign(&mut self, other: __p) {
        *self = self.mul(other)
    }
}

impl MulAssign<&__p> for __p {
    #[inline]
    fn mul_assign(&mut self, other: &__p) {
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
    #[inline]
    fn div(self, other: __p) -> __p {
        __p::naive_div(self, other)
    }
}

impl Div<__p> for &__p {
    type Output = __p;
    #[inline]
    fn div(self, other: __p) -> __p {
        __p::naive_div(*self, other)
    }
}

impl Div<&__p> for __p {
    type Output = __p;
    #[inline]
    fn div(self, other: &__p) -> __p {
        __p::naive_div(self, *other)
    }
}

impl Div<&__p> for &__p {
    type Output = __p;
    #[inline]
    fn div(self, other: &__p) -> __p {
        __p::naive_div(*self, *other)
    }
}

impl DivAssign<__p> for __p {
    #[inline]
    fn div_assign(&mut self, other: __p) {
        *self = self.div(other)
    }
}

impl DivAssign<&__p> for __p {
    #[inline]
    fn div_assign(&mut self, other: &__p) {
        *self = self.div(*other)
    }
}


//// Remainder ////

impl Rem for __p {
    type Output = __p;
    #[inline]
    fn rem(self, other: __p) -> __p {
        __p::naive_rem(self, other)
    }
}

impl Rem<__p> for &__p {
    type Output = __p;
    #[inline]
    fn rem(self, other: __p) -> __p {
        __p::naive_rem(*self, other)
    }
}

impl Rem<&__p> for __p {
    type Output = __p;
    #[inline]
    fn rem(self, other: &__p) -> __p {
        __p::naive_rem(self, *other)
    }
}

impl Rem<&__p> for &__p {
    type Output = __p;
    #[inline]
    fn rem(self, other: &__p) -> __p {
        __p::naive_rem(*self, *other)
    }
}

impl RemAssign<__p> for __p {
    #[inline]
    fn rem_assign(&mut self, other: __p) {
        *self = self.rem(other)
    }
}

impl RemAssign<&__p> for __p {
    #[inline]
    fn rem_assign(&mut self, other: &__p) {
        *self = self.rem(*other)
    }
}


//// Bitwise operations ////

impl Not for __p {
    type Output = __p;
    #[inline]
    fn not(self) -> __p {
        __p(!self.0)
    }
}

impl Not for &__p {
    type Output = __p;
    #[inline]
    fn not(self) -> __p {
        __p(!self.0)
    }
}

impl BitAnd<__p> for __p {
    type Output = __p;
    #[inline]
    fn bitand(self, other: __p) -> __p {
        __p(self.0 & other.0)
    }
}

impl BitAnd<__p> for &__p {
    type Output = __p;
    #[inline]
    fn bitand(self, other: __p) -> __p {
        __p(self.0 & other.0)
    }
}

impl BitAnd<&__p> for __p {
    type Output = __p;
    #[inline]
    fn bitand(self, other: &__p) -> __p {
        __p(self.0 & other.0)
    }
}

impl BitAnd<&__p> for &__p {
    type Output = __p;
    #[inline]
    fn bitand(self, other: &__p) -> __p {
        __p(self.0 & other.0)
    }
}

impl BitAndAssign<__p> for __p {
    #[inline]
    fn bitand_assign(&mut self, other: __p) {
        *self = *self & other;
    }
}

impl BitAndAssign<&__p> for __p {
    #[inline]
    fn bitand_assign(&mut self, other: &__p) {
        *self = *self & *other;
    }
}

impl BitAnd<__p> for __u {
    type Output = __p;
    #[inline]
    fn bitand(self, other: __p) -> __p {
        __p(self & other.0)
    }
}

impl BitAnd<__p> for &__u {
    type Output = __p;
    #[inline]
    fn bitand(self, other: __p) -> __p {
        __p(self & other.0)
    }
}

impl BitAnd<&__p> for __u {
    type Output = __p;
    #[inline]
    fn bitand(self, other: &__p) -> __p {
        __p(self & other.0)
    }
}

impl BitAnd<&__p> for &__u {
    type Output = __p;
    #[inline]
    fn bitand(self, other: &__p) -> __p {
        __p(self & other.0)
    }
}

impl BitAnd<__u> for __p {
    type Output = __p;
    #[inline]
    fn bitand(self, other: __u) -> __p {
        __p(self.0 & other)
    }
}

impl BitAnd<__u> for &__p {
    type Output = __p;
    #[inline]
    fn bitand(self, other: __u) -> __p {
        __p(self.0 & other)
    }
}

impl BitAnd<&__u> for __p {
    type Output = __p;
    #[inline]
    fn bitand(self, other: &__u) -> __p {
        __p(self.0 & other)
    }
}

impl BitAnd<&__u> for &__p {
    type Output = __p;
    #[inline]
    fn bitand(self, other: &__u) -> __p {
        __p(self.0 & other)
    }
}

impl BitAndAssign<__u> for __p {
    #[inline]
    fn bitand_assign(&mut self, other: __u) {
        *self = *self & other;
    }
}

impl BitAndAssign<&__u> for __p {
    #[inline]
    fn bitand_assign(&mut self, other: &__u) {
        *self = *self & *other;
    }
}

impl BitOr<__p> for __p {
    type Output = __p;
    #[inline]
    fn bitor(self, other: __p) -> __p {
        __p(self.0 | other.0)
    }
}

impl BitOr<__p> for &__p {
    type Output = __p;
    #[inline]
    fn bitor(self, other: __p) -> __p {
        __p(self.0 | other.0)
    }
}

impl BitOr<&__p> for __p {
    type Output = __p;
    #[inline]
    fn bitor(self, other: &__p) -> __p {
        __p(self.0 | other.0)
    }
}

impl BitOr<&__p> for &__p {
    type Output = __p;
    #[inline]
    fn bitor(self, other: &__p) -> __p {
        __p(self.0 | other.0)
    }
}

impl BitOrAssign<__p> for __p {
    #[inline]
    fn bitor_assign(&mut self, other: __p) {
        *self = *self | other;
    }
}

impl BitOrAssign<&__p> for __p {
    #[inline]
    fn bitor_assign(&mut self, other: &__p) {
        *self = *self | *other;
    }
}

impl BitOr<__p> for __u {
    type Output = __p;
    #[inline]
    fn bitor(self, other: __p) -> __p {
        __p(self | other.0)
    }
}

impl BitOr<__p> for &__u {
    type Output = __p;
    #[inline]
    fn bitor(self, other: __p) -> __p {
        __p(self | other.0)
    }
}

impl BitOr<&__p> for __u {
    type Output = __p;
    #[inline]
    fn bitor(self, other: &__p) -> __p {
        __p(self | other.0)
    }
}

impl BitOr<&__p> for &__u {
    type Output = __p;
    #[inline]
    fn bitor(self, other: &__p) -> __p {
        __p(self | other.0)
    }
}

impl BitOr<__u> for __p {
    type Output = __p;
    #[inline]
    fn bitor(self, other: __u) -> __p {
        __p(self.0 | other)
    }
}

impl BitOr<__u> for &__p {
    type Output = __p;
    #[inline]
    fn bitor(self, other: __u) -> __p {
        __p(self.0 | other)
    }
}

impl BitOr<&__u> for __p {
    type Output = __p;
    #[inline]
    fn bitor(self, other: &__u) -> __p {
        __p(self.0 | other)
    }
}

impl BitOr<&__u> for &__p {
    type Output = __p;
    #[inline]
    fn bitor(self, other: &__u) -> __p {
        __p(self.0 | other)
    }
}

impl BitOrAssign<__u> for __p {
    #[inline]
    fn bitor_assign(&mut self, other: __u) {
        *self = *self | other;
    }
}

impl BitOrAssign<&__u> for __p {
    #[inline]
    fn bitor_assign(&mut self, other: &__u) {
        *self = *self | *other;
    }
}

impl BitXor<__p> for __p {
    type Output = __p;
    #[inline]
    fn bitxor(self, other: __p) -> __p {
        __p(self.0 ^ other.0)
    }
}

impl BitXor<__p> for &__p {
    type Output = __p;
    #[inline]
    fn bitxor(self, other: __p) -> __p {
        __p(self.0 ^ other.0)
    }
}

impl BitXor<&__p> for __p {
    type Output = __p;
    #[inline]
    fn bitxor(self, other: &__p) -> __p {
        __p(self.0 ^ other.0)
    }
}

impl BitXor<&__p> for &__p {
    type Output = __p;
    #[inline]
    fn bitxor(self, other: &__p) -> __p {
        __p(self.0 ^ other.0)
    }
}

impl BitXorAssign<__p> for __p {
    #[inline]
    fn bitxor_assign(&mut self, other: __p) {
        *self = *self ^ other;
    }
}

impl BitXorAssign<&__p> for __p {
    #[inline]
    fn bitxor_assign(&mut self, other: &__p) {
        *self = *self ^ *other;
    }
}

impl BitXor<__p> for __u {
    type Output = __p;
    #[inline]
    fn bitxor(self, other: __p) -> __p {
        __p(self ^ other.0)
    }
}

impl BitXor<__p> for &__u {
    type Output = __p;
    #[inline]
    fn bitxor(self, other: __p) -> __p {
        __p(self ^ other.0)
    }
}

impl BitXor<&__p> for __u {
    type Output = __p;
    #[inline]
    fn bitxor(self, other: &__p) -> __p {
        __p(self ^ other.0)
    }
}

impl BitXor<&__p> for &__u {
    type Output = __p;
    #[inline]
    fn bitxor(self, other: &__p) -> __p {
        __p(self ^ other.0)
    }
}

impl BitXor<__u> for __p {
    type Output = __p;
    #[inline]
    fn bitxor(self, other: __u) -> __p {
        __p(self.0 ^ other)
    }
}

impl BitXor<__u> for &__p {
    type Output = __p;
    #[inline]
    fn bitxor(self, other: __u) -> __p {
        __p(self.0 ^ other)
    }
}

impl BitXor<&__u> for __p {
    type Output = __p;
    #[inline]
    fn bitxor(self, other: &__u) -> __p {
        __p(self.0 ^ other)
    }
}

impl BitXor<&__u> for &__p {
    type Output = __p;
    #[inline]
    fn bitxor(self, other: &__u) -> __p {
        __p(self.0 ^ other)
    }
}

impl BitXorAssign<__u> for __p {
    #[inline]
    fn bitxor_assign(&mut self, other: __u) {
        *self = *self ^ other;
    }
}

impl BitXorAssign<&__u> for __p {
    #[inline]
    fn bitxor_assign(&mut self, other: &__u) {
        *self = *self ^ *other;
    }
}


//// Byte order ////

impl __p {
    #[inline]
    pub const fn swap_bytes(self) -> __p {
        __p(self.0.swap_bytes())
    }

    #[inline]
    pub const fn to_le(self) -> __p {
        __p(self.0.to_le())
    }

    #[inline]
    pub const fn from_le(self_: __p) -> __p {
        __p(__u::from_le(self_.0))
    }

    #[inline]
    pub const fn to_le_bytes(self) -> [u8; __width/8] {
        self.0.to_le_bytes()
    }

    #[inline]
    pub const fn from_le_bytes(bytes: [u8; __width/8]) -> __p {
        __p(__u::from_le_bytes(bytes))
    }

    #[inline]
    pub const fn to_be(self) -> __p {
        __p(self.0.to_be())
    }

    #[inline]
    pub const fn from_be(self_: __p) -> __p {
        __p(__u::from_be(self_.0))
    }

    #[inline]
    pub const fn to_be_bytes(self) -> [u8; __width/8] {
        self.0.to_be_bytes()
    }

    #[inline]
    pub const fn from_be_bytes(bytes: [u8; __width/8]) -> __p {
        __p(__u::from_be_bytes(bytes))
    }

    #[inline]
    pub const fn to_ne_bytes(self) -> [u8; __width/8] {
        self.0.to_ne_bytes()
    }

    #[inline]
    pub const fn from_ne_bytes(bytes: [u8; __width/8]) -> __p {
        __p(__u::from_ne_bytes(bytes))
    }
}


//// Other bit things ////

impl __p {
    #[inline]
    pub const fn reverse_bits(self) -> __p {
        __p(self.0.reverse_bits())
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

impl __p {
    #[inline]
    pub const fn checked_shl(self, other: u32) -> Option<__p> {
        match self.0.checked_shl(other) {
            Some(x) => Some(__p(x)),
            None => None,
        }
    }

    #[inline]
    pub const fn checked_shr(self, other: u32) -> Option<__p> {
        match self.0.checked_shr(other) {
            Some(x) => Some(__p(x)),
            None => None,
        }
    }

    #[inline]
    pub const fn overflowing_shl(self, other: u32) -> (__p, bool) {
        let (x, o) = self.0.overflowing_shl(other);
        (__p(x), o)
    }

    #[inline]
    pub const fn overflowing_shr(self, other: u32) -> (__p, bool) {
        let (x, o) = self.0.overflowing_shr(other);
        (__p(x), o)
    }

    #[inline]
    pub const fn wrapping_shl(self, other: u32) -> __p {
        __p(self.0.wrapping_shl(other))
    }

    #[inline]
    pub const fn wrapping_shr(self, other: u32) -> __p {
        __p(self.0.wrapping_shr(other))
    }

    #[inline]
    pub const fn rotate_left(self, other: u32) -> __p {
        __p(self.0.rotate_left(other))
    }

    #[inline]
    pub const fn rotate_right(self, other: u32) -> __p {
        __p(self.0.rotate_right(other))
    }
}

impl Shl<u8> for __p {
    type Output = __p;
    #[inline]
    fn shl(self, other: u8) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<u8> for &__p {
    type Output = __p;
    #[inline]
    fn shl(self, other: u8) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<&u8> for __p {
    type Output = __p;
    #[inline]
    fn shl(self, other: &u8) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<&u8> for &__p {
    type Output = __p;
    #[inline]
    fn shl(self, other: &u8) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<u16> for __p {
    type Output = __p;
    #[inline]
    fn shl(self, other: u16) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<u16> for &__p {
    type Output = __p;
    #[inline]
    fn shl(self, other: u16) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<&u16> for __p {
    type Output = __p;
    #[inline]
    fn shl(self, other: &u16) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<&u16> for &__p {
    type Output = __p;
    #[inline]
    fn shl(self, other: &u16) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<u32> for __p {
    type Output = __p;
    #[inline]
    fn shl(self, other: u32) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<u32> for &__p {
    type Output = __p;
    #[inline]
    fn shl(self, other: u32) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<&u32> for __p {
    type Output = __p;
    #[inline]
    fn shl(self, other: &u32) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<&u32> for &__p {
    type Output = __p;
    #[inline]
    fn shl(self, other: &u32) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<u64> for __p {
    type Output = __p;
    #[inline]
    fn shl(self, other: u64) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<u64> for &__p {
    type Output = __p;
    #[inline]
    fn shl(self, other: u64) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<&u64> for __p {
    type Output = __p;
    #[inline]
    fn shl(self, other: &u64) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<&u64> for &__p {
    type Output = __p;
    #[inline]
    fn shl(self, other: &u64) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<u128> for __p {
    type Output = __p;
    #[inline]
    fn shl(self, other: u128) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<u128> for &__p {
    type Output = __p;
    #[inline]
    fn shl(self, other: u128) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<&u128> for __p {
    type Output = __p;
    #[inline]
    fn shl(self, other: &u128) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<&u128> for &__p {
    type Output = __p;
    #[inline]
    fn shl(self, other: &u128) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<usize> for __p {
    type Output = __p;
    #[inline]
    fn shl(self, other: usize) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<usize> for &__p {
    type Output = __p;
    #[inline]
    fn shl(self, other: usize) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<&usize> for __p {
    type Output = __p;
    #[inline]
    fn shl(self, other: &usize) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<&usize> for &__p {
    type Output = __p;
    #[inline]
    fn shl(self, other: &usize) -> __p {
        __p(self.0 << other)
    }
}

impl ShlAssign<u8> for __p {
    #[inline]
    fn shl_assign(&mut self, other: u8) {
        *self = *self << other;
    }
}

impl ShlAssign<&u8> for __p {
    #[inline]
    fn shl_assign(&mut self, other: &u8) {
        *self = *self << other;
    }
}

impl ShlAssign<u16> for __p {
    #[inline]
    fn shl_assign(&mut self, other: u16) {
        *self = *self << other;
    }
}

impl ShlAssign<&u16> for __p {
    #[inline]
    fn shl_assign(&mut self, other: &u16) {
        *self = *self << other;
    }
}

impl ShlAssign<u32> for __p {
    #[inline]
    fn shl_assign(&mut self, other: u32) {
        *self = *self << other;
    }
}

impl ShlAssign<&u32> for __p {
    #[inline]
    fn shl_assign(&mut self, other: &u32) {
        *self = *self << other;
    }
}

impl ShlAssign<u64> for __p {
    #[inline]
    fn shl_assign(&mut self, other: u64) {
        *self = *self << other;
    }
}

impl ShlAssign<&u64> for __p {
    #[inline]
    fn shl_assign(&mut self, other: &u64) {
        *self = *self << other;
    }
}

impl ShlAssign<u128> for __p {
    #[inline]
    fn shl_assign(&mut self, other: u128) {
        *self = *self << other;
    }
}

impl ShlAssign<&u128> for __p {
    #[inline]
    fn shl_assign(&mut self, other: &u128) {
        *self = *self << other;
    }
}

impl ShlAssign<usize> for __p {
    #[inline]
    fn shl_assign(&mut self, other: usize) {
        *self = *self << other;
    }
}

impl ShlAssign<&usize> for __p {
    #[inline]
    fn shl_assign(&mut self, other: &usize) {
        *self = *self << other;
    }
}

impl Shr<u8> for __p {
    type Output = __p;
    #[inline]
    fn shr(self, other: u8) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<u8> for &__p {
    type Output = __p;
    #[inline]
    fn shr(self, other: u8) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<&u8> for __p {
    type Output = __p;
    #[inline]
    fn shr(self, other: &u8) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<&u8> for &__p {
    type Output = __p;
    #[inline]
    fn shr(self, other: &u8) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<u16> for __p {
    type Output = __p;
    #[inline]
    fn shr(self, other: u16) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<u16> for &__p {
    type Output = __p;
    #[inline]
    fn shr(self, other: u16) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<&u16> for __p {
    type Output = __p;
    #[inline]
    fn shr(self, other: &u16) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<&u16> for &__p {
    type Output = __p;
    #[inline]
    fn shr(self, other: &u16) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<u32> for __p {
    type Output = __p;
    #[inline]
    fn shr(self, other: u32) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<u32> for &__p {
    type Output = __p;
    #[inline]
    fn shr(self, other: u32) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<&u32> for __p {
    type Output = __p;
    #[inline]
    fn shr(self, other: &u32) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<&u32> for &__p {
    type Output = __p;
    #[inline]
    fn shr(self, other: &u32) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<u64> for __p {
    type Output = __p;
    #[inline]
    fn shr(self, other: u64) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<u64> for &__p {
    type Output = __p;
    #[inline]
    fn shr(self, other: u64) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<&u64> for __p {
    type Output = __p;
    #[inline]
    fn shr(self, other: &u64) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<&u64> for &__p {
    type Output = __p;
    #[inline]
    fn shr(self, other: &u64) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<u128> for __p {
    type Output = __p;
    #[inline]
    fn shr(self, other: u128) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<u128> for &__p {
    type Output = __p;
    #[inline]
    fn shr(self, other: u128) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<&u128> for __p {
    type Output = __p;
    #[inline]
    fn shr(self, other: &u128) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<&u128> for &__p {
    type Output = __p;
    #[inline]
    fn shr(self, other: &u128) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<usize> for __p {
    type Output = __p;
    #[inline]
    fn shr(self, other: usize) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<usize> for &__p {
    type Output = __p;
    #[inline]
    fn shr(self, other: usize) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<&usize> for __p {
    type Output = __p;
    #[inline]
    fn shr(self, other: &usize) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<&usize> for &__p {
    type Output = __p;
    #[inline]
    fn shr(self, other: &usize) -> __p {
        __p(self.0 >> other)
    }
}

impl ShrAssign<u8> for __p {
    #[inline]
    fn shr_assign(&mut self, other: u8) {
        *self = *self >> other;
    }
}

impl ShrAssign<&u8> for __p {
    #[inline]
    fn shr_assign(&mut self, other: &u8) {
        *self = *self >> other;
    }
}

impl ShrAssign<u16> for __p {
    #[inline]
    fn shr_assign(&mut self, other: u16) {
        *self = *self >> other;
    }
}

impl ShrAssign<&u16> for __p {
    #[inline]
    fn shr_assign(&mut self, other: &u16) {
        *self = *self >> other;
    }
}

impl ShrAssign<u32> for __p {
    #[inline]
    fn shr_assign(&mut self, other: u32) {
        *self = *self >> other;
    }
}

impl ShrAssign<&u32> for __p {
    #[inline]
    fn shr_assign(&mut self, other: &u32) {
        *self = *self >> other;
    }
}

impl ShrAssign<u64> for __p {
    #[inline]
    fn shr_assign(&mut self, other: u64) {
        *self = *self >> other;
    }
}

impl ShrAssign<&u64> for __p {
    #[inline]
    fn shr_assign(&mut self, other: &u64) {
        *self = *self >> other;
    }
}

impl ShrAssign<u128> for __p {
    #[inline]
    fn shr_assign(&mut self, other: u128) {
        *self = *self >> other;
    }
}

impl ShrAssign<&u128> for __p {
    #[inline]
    fn shr_assign(&mut self, other: &u128) {
        *self = *self >> other;
    }
}

impl ShrAssign<usize> for __p {
    #[inline]
    fn shr_assign(&mut self, other: usize) {
        *self = *self >> other;
    }
}

impl ShrAssign<&usize> for __p {
    #[inline]
    fn shr_assign(&mut self, other: &usize) {
        *self = *self >> other;
    }
}

impl Shl<i8> for __p {
    type Output = __p;
    #[inline]
    fn shl(self, other: i8) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<i8> for &__p {
    type Output = __p;
    #[inline]
    fn shl(self, other: i8) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<&i8> for __p {
    type Output = __p;
    #[inline]
    fn shl(self, other: &i8) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<&i8> for &__p {
    type Output = __p;
    #[inline]
    fn shl(self, other: &i8) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<i16> for __p {
    type Output = __p;
    #[inline]
    fn shl(self, other: i16) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<i16> for &__p {
    type Output = __p;
    #[inline]
    fn shl(self, other: i16) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<&i16> for __p {
    type Output = __p;
    #[inline]
    fn shl(self, other: &i16) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<&i16> for &__p {
    type Output = __p;
    #[inline]
    fn shl(self, other: &i16) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<i32> for __p {
    type Output = __p;
    #[inline]
    fn shl(self, other: i32) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<i32> for &__p {
    type Output = __p;
    #[inline]
    fn shl(self, other: i32) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<&i32> for __p {
    type Output = __p;
    #[inline]
    fn shl(self, other: &i32) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<&i32> for &__p {
    type Output = __p;
    #[inline]
    fn shl(self, other: &i32) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<i64> for __p {
    type Output = __p;
    #[inline]
    fn shl(self, other: i64) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<i64> for &__p {
    type Output = __p;
    #[inline]
    fn shl(self, other: i64) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<&i64> for __p {
    type Output = __p;
    #[inline]
    fn shl(self, other: &i64) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<&i64> for &__p {
    type Output = __p;
    #[inline]
    fn shl(self, other: &i64) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<i128> for __p {
    type Output = __p;
    #[inline]
    fn shl(self, other: i128) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<i128> for &__p {
    type Output = __p;
    #[inline]
    fn shl(self, other: i128) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<&i128> for __p {
    type Output = __p;
    #[inline]
    fn shl(self, other: &i128) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<&i128> for &__p {
    type Output = __p;
    #[inline]
    fn shl(self, other: &i128) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<isize> for __p {
    type Output = __p;
    #[inline]
    fn shl(self, other: isize) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<isize> for &__p {
    type Output = __p;
    #[inline]
    fn shl(self, other: isize) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<&isize> for __p {
    type Output = __p;
    #[inline]
    fn shl(self, other: &isize) -> __p {
        __p(self.0 << other)
    }
}

impl Shl<&isize> for &__p {
    type Output = __p;
    #[inline]
    fn shl(self, other: &isize) -> __p {
        __p(self.0 << other)
    }
}

impl ShlAssign<i8> for __p {
    #[inline]
    fn shl_assign(&mut self, other: i8) {
        *self = *self << other;
    }
}

impl ShlAssign<&i8> for __p {
    #[inline]
    fn shl_assign(&mut self, other: &i8) {
        *self = *self << other;
    }
}

impl ShlAssign<i16> for __p {
    #[inline]
    fn shl_assign(&mut self, other: i16) {
        *self = *self << other;
    }
}

impl ShlAssign<&i16> for __p {
    #[inline]
    fn shl_assign(&mut self, other: &i16) {
        *self = *self << other;
    }
}

impl ShlAssign<i32> for __p {
    #[inline]
    fn shl_assign(&mut self, other: i32) {
        *self = *self << other;
    }
}

impl ShlAssign<&i32> for __p {
    #[inline]
    fn shl_assign(&mut self, other: &i32) {
        *self = *self << other;
    }
}

impl ShlAssign<i64> for __p {
    #[inline]
    fn shl_assign(&mut self, other: i64) {
        *self = *self << other;
    }
}

impl ShlAssign<&i64> for __p {
    #[inline]
    fn shl_assign(&mut self, other: &i64) {
        *self = *self << other;
    }
}

impl ShlAssign<i128> for __p {
    #[inline]
    fn shl_assign(&mut self, other: i128) {
        *self = *self << other;
    }
}

impl ShlAssign<&i128> for __p {
    #[inline]
    fn shl_assign(&mut self, other: &i128) {
        *self = *self << other;
    }
}

impl ShlAssign<isize> for __p {
    #[inline]
    fn shl_assign(&mut self, other: isize) {
        *self = *self << other;
    }
}

impl ShlAssign<&isize> for __p {
    #[inline]
    fn shl_assign(&mut self, other: &isize) {
        *self = *self << other;
    }
}

impl Shr<i8> for __p {
    type Output = __p;
    #[inline]
    fn shr(self, other: i8) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<i8> for &__p {
    type Output = __p;
    #[inline]
    fn shr(self, other: i8) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<&i8> for __p {
    type Output = __p;
    #[inline]
    fn shr(self, other: &i8) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<&i8> for &__p {
    type Output = __p;
    #[inline]
    fn shr(self, other: &i8) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<i16> for __p {
    type Output = __p;
    #[inline]
    fn shr(self, other: i16) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<i16> for &__p {
    type Output = __p;
    #[inline]
    fn shr(self, other: i16) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<&i16> for __p {
    type Output = __p;
    #[inline]
    fn shr(self, other: &i16) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<&i16> for &__p {
    type Output = __p;
    #[inline]
    fn shr(self, other: &i16) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<i32> for __p {
    type Output = __p;
    #[inline]
    fn shr(self, other: i32) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<i32> for &__p {
    type Output = __p;
    #[inline]
    fn shr(self, other: i32) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<&i32> for __p {
    type Output = __p;
    #[inline]
    fn shr(self, other: &i32) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<&i32> for &__p {
    type Output = __p;
    #[inline]
    fn shr(self, other: &i32) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<i64> for __p {
    type Output = __p;
    #[inline]
    fn shr(self, other: i64) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<i64> for &__p {
    type Output = __p;
    #[inline]
    fn shr(self, other: i64) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<&i64> for __p {
    type Output = __p;
    #[inline]
    fn shr(self, other: &i64) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<&i64> for &__p {
    type Output = __p;
    #[inline]
    fn shr(self, other: &i64) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<i128> for __p {
    type Output = __p;
    #[inline]
    fn shr(self, other: i128) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<i128> for &__p {
    type Output = __p;
    #[inline]
    fn shr(self, other: i128) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<&i128> for __p {
    type Output = __p;
    #[inline]
    fn shr(self, other: &i128) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<&i128> for &__p {
    type Output = __p;
    #[inline]
    fn shr(self, other: &i128) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<isize> for __p {
    type Output = __p;
    #[inline]
    fn shr(self, other: isize) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<isize> for &__p {
    type Output = __p;
    #[inline]
    fn shr(self, other: isize) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<&isize> for __p {
    type Output = __p;
    #[inline]
    fn shr(self, other: &isize) -> __p {
        __p(self.0 >> other)
    }
}

impl Shr<&isize> for &__p {
    type Output = __p;
    #[inline]
    fn shr(self, other: &isize) -> __p {
        __p(self.0 >> other)
    }
}

impl ShrAssign<i8> for __p {
    #[inline]
    fn shr_assign(&mut self, other: i8) {
        *self = *self >> other;
    }
}

impl ShrAssign<&i8> for __p {
    #[inline]
    fn shr_assign(&mut self, other: &i8) {
        *self = *self >> other;
    }
}

impl ShrAssign<i16> for __p {
    #[inline]
    fn shr_assign(&mut self, other: i16) {
        *self = *self >> other;
    }
}

impl ShrAssign<&i16> for __p {
    #[inline]
    fn shr_assign(&mut self, other: &i16) {
        *self = *self >> other;
    }
}

impl ShrAssign<i32> for __p {
    #[inline]
    fn shr_assign(&mut self, other: i32) {
        *self = *self >> other;
    }
}

impl ShrAssign<&i32> for __p {
    #[inline]
    fn shr_assign(&mut self, other: &i32) {
        *self = *self >> other;
    }
}

impl ShrAssign<i64> for __p {
    #[inline]
    fn shr_assign(&mut self, other: i64) {
        *self = *self >> other;
    }
}

impl ShrAssign<&i64> for __p {
    #[inline]
    fn shr_assign(&mut self, other: &i64) {
        *self = *self >> other;
    }
}

impl ShrAssign<i128> for __p {
    #[inline]
    fn shr_assign(&mut self, other: i128) {
        *self = *self >> other;
    }
}

impl ShrAssign<&i128> for __p {
    #[inline]
    fn shr_assign(&mut self, other: &i128) {
        *self = *self >> other;
    }
}

impl ShrAssign<isize> for __p {
    #[inline]
    fn shr_assign(&mut self, other: isize) {
        *self = *self >> other;
    }
}

impl ShrAssign<&isize> for __p {
    #[inline]
    fn shr_assign(&mut self, other: &isize) {
        *self = *self >> other;
    }
}


//// To/from strings ////

impl fmt::Debug for __p {
    /// We use LowerHex for Debug, since this is a more useful representation
    /// of binary polynomials.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}(0x{:x})", stringify!(__p), self.0)
    }
}

impl fmt::Display for __p {
    /// We use LowerHex for Display since this is a more useful representation
    /// of binary polynomials.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "0x{:x}", self.0)
    }
}

impl fmt::Binary for __p {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        <__u as fmt::Binary>::fmt(&self.0, f)
    }
}

impl fmt::Octal for __p {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        <__u as fmt::Octal>::fmt(&self.0, f)
    }
}

impl fmt::LowerHex for __p {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        <__u as fmt::LowerHex>::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for __p {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        <__u as fmt::UpperHex>::fmt(&self.0, f)
    }
}

impl FromStr for __p {
    type Err = ParseIntError;

    /// In order to match Display, this `from_str` takes and only takes
    /// hexadecimal strings starting with `0x`. If you need a different radix
    /// there is [`from_str_radix`](#method.from_str_radix).
    fn from_str(s: &str) -> Result<__p, ParseIntError> {
        if s.starts_with("0x") {
            Ok(__p(__u::from_str_radix(&s[2..], 16)?))
        } else {
            "".parse::<__u>()?;
            unreachable!()
        }
    }
}

impl __p {
    pub fn from_str_radix(s: &str, radix: u32) -> Result<__p, ParseIntError> {
        Ok(__p(__u::from_str_radix(s, radix)?))
    }
}
