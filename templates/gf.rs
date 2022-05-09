///! Template for polynomial types

use core::ops::*;
use core::iter::*;
use core::fmt;
use core::str::FromStr;
use core::num::TryFromIntError;
use core::num::ParseIntError;
use core::mem::size_of;
use core::slice;

use __crate::traits::TryFrom;
use __crate::traits::FromLossy;
use __crate::internal::cfg_if::cfg_if;


/// A binary-extension finite-field type.
///
/// ``` rust
/// use ::gf256::*;
///
/// let a = gf256(0xfd);
/// let b = gf256(0xfe);
/// let c = gf256(0xff);
/// assert_eq!(a*(b+c), a*b + a*c);
/// ```
///
/// See the [module-level documentation](../gf) for more info.
///
#[allow(non_camel_case_types)]
#[derive(Default, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct __gf(
    #[cfg(__if(__is_pw2ge8))] pub __u,
    #[cfg(__if(!__is_pw2ge8))] __u,
);

impl __gf {
    /// The irreducible polynomial that defines the field.
    ///
    /// In order to keep polynomial multiplication closed over a
    /// finite-field, all multiplications are performed modulo this
    /// polynomial.
    ///
    pub const POLYNOMIAL: __p2 = __p2(__polynomial);

    /// A generator, aka primitive element, in the field.
    ///
    /// Repeated multiplications of the generator will eventually
    /// iterate through ever non-zero element of the field.
    ///
    pub const GENERATOR: __gf = __gf(__generator);

    /// Number of non-zero elements in the field.
    pub const NONZEROS: __u = __nonzeros;

    // Generate log/antilog tables using our generator if we're in table mode
    #[cfg(__if(__table))]
    const LOG_TABLE: [__u; __nonzeros+1] = Self::LOG_EXP_TABLES.0;
    #[cfg(__if(__table))]
    const EXP_TABLE: [__u; __nonzeros+1] = Self::LOG_EXP_TABLES.1;
    #[cfg(__if(__table))]
    const LOG_EXP_TABLES: ([__u; __nonzeros+1], [__u; __nonzeros+1]) = {
        let mut log_table = [0; __nonzeros+1];
        let mut exp_table = [0; __nonzeros+1];

        let mut x = 1;
        let mut i = 0;
        while i < __nonzeros+1 {
            log_table[x as usize] = i as __u;
            exp_table[i as usize] = x as __u;

            x = __p2(x)
                .naive_mul(__p2(__generator))
                .naive_rem(__p2(__polynomial)).0;
            i += 1;
        }

        log_table[0] = __nonzeros; // log(0) is undefined
        log_table[1] = 0;          // log(1) is 0
        (log_table, exp_table)
    };

    // Generate remainder tables if we're in rem_table mode
    //
    #[cfg(__if(__rem_table))]
    const REM_TABLE: [__p; 256] = {
        let mut rem_table = [__p(0); 256];

        let mut i = 0;
        while i < rem_table.len() {
            rem_table[i] = __p(
                __p2((i as __u2) << 8*size_of::<__u>())
                    .naive_rem(__p2(__polynomial << (8*size_of::<__u>()-__width)))
                    .0 as __u
            );
            i += 1;
        }

        rem_table
    };

    // Generate small remainder tables if we're in small_rem_table mode
    //
    #[cfg(__if(__small_rem_table))]
    const REM_TABLE: [__p; 16] = {
        let mut rem_table = [__p(0); 16];

        let mut i = 0;
        while i < rem_table.len() {
            rem_table[i] = __p(
                __p2((i as __u2) << 8*size_of::<__u>())
                    .naive_rem(__p2(__polynomial << (8*size_of::<__u>()-__width)))
                    .0 as __u
            );
            i += 1;
        }

        rem_table
    };

    // Generate constant for Barret's reduction if we're
    // in Barret mode
    //
    #[cfg(__if(__barret))]
    const BARRET_CONSTANT: __p = {
        // Normally this would be 0x10000 / __polynomial, but we eagerly
        // do one step of division so we avoid needing a 4x wide type. We
        // can also drop the highest bit if we add the high bits manually
        // we use use this constant.
        //
        // = x % p
        // = 0xff & (x + p*(((x >> 8) * [0x10000/p]) >> 8))
        // = 0xff & (x + p*(((x >> 8) * [(p << 8)/p + 0x100]) >> 8))
        // = 0xff & (x + p*((((x >> 8) * [(p << 8)/p]) >> 8) + (x >> 8)))
        //                               \-----+----/
        //                                     '-- Barret constant
        //
        // Note that the shifts and masks can go away if we operate on u8s,
        // leaving 2 xmuls and 2 xors.
        //
        __p(
            __p2((__polynomial & __nonzeros) << ((8*size_of::<__u>()-__width) + 8*size_of::<__u>()))
                .naive_div(__p2(__polynomial << (8*size_of::<__u>()-__width)))
                .0 as __u
        )
    };

    /// Create a finite-field element, panicking if the argument can't be
    /// represented in the field.
    #[inline]
    pub const fn new(x: __u) -> __gf {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                __gf(x)
            } else {
                if x < __nonzeros+1 {
                    __gf(x)
                } else {
                    panic!(concat!("value unrepresentable in ", stringify!(__gf)))
                }
            }
        }
    }

    /// Create a finite-field element.
    #[inline]
    pub const unsafe fn new_unchecked(x: __u) -> __gf {
        __gf(x)
    }

    /// Get the underlying primitive type.
    #[inline]
    pub const fn get(self) -> __u {
        self.0
    }

    /// Addition over the finite-field, aka xor.
    ///
    /// Note that since this is defined over a finite-field, it's not actually
    /// possible for this operation to overflow.
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// const X: gf256 = gf256(0x12).naive_add(gf256(0x34));
    /// assert_eq!(X, gf256(0x26));
    /// ```
    ///
    #[inline]
    pub const fn naive_add(self, other: __gf) -> __gf {
        __gf(self.0 ^ other.0)
    }

    /// Addition over the finite-field, aka xor.
    ///
    /// Note that since this is defined over a finite-field, it's not actually
    /// possible for this operation to overflow.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// assert_eq!(gf256(0x12) + gf256(0x34), gf256(0x26));
    /// ```
    ///
    #[inline]
    pub fn add(self, other: __gf) -> __gf {
        __gf(self.0 ^ other.0)
    }

    /// Subtraction over the finite-field, aka xor.
    ///
    /// Note that since this is defined over a finite-field, it's not actually
    /// possible for this operation to overflow.
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// const X: gf256 = gf256(0x12).naive_sub(gf256(0x34));
    /// assert_eq!(X, gf256(0x26));
    /// ```
    ///
    #[inline]
    pub const fn naive_sub(self, other: __gf) -> __gf {
        __gf(self.0 ^ other.0)
    }

    /// Subtraction over the finite-field, aka xor.
    ///
    /// Note that since this is defined over a finite-field, it's not actually
    /// possible for this operation to overflow.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// assert_eq!(gf256(0x12) - gf256(0x34), gf256(0x26));
    /// ```
    ///
    #[inline]
    pub fn sub(self, other: __gf) -> __gf {
        __gf(self.0 ^ other.0)
    }

    /// Naive multiplication over the finite-field.
    ///
    /// Note that since this is defined over a finite-field, it's not actually
    /// possible for this operation to overflow.
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// const X: gf256 = gf256(0x12).naive_mul(gf256(0x34));
    /// assert_eq!(X, gf256(0x0f));
    /// ```
    ///
    /// One important property of finite-fields, multiplication is distributive
    /// over addition:
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// const A: gf256 = gf256(0x12);
    /// const B: gf256 = gf256(0x34);
    /// const C: gf256 = gf256(0x56);
    /// const X: gf256 = A.naive_mul(B.naive_add(C));
    /// const Y: gf256 = A.naive_mul(B).naive_add(A.naive_mul(C));
    /// assert_eq!(X, Y);
    /// ```
    ///
    #[inline]
    pub const fn naive_mul(self, other: __gf) -> __gf {
        __gf(
            __p2(self.0 as _)
                .naive_mul(__p2(other.0 as _))
                .naive_rem(__p2(__polynomial))
                .0 as __u
        )
    }

    /// Naive exponentiation over the finite-field.
    ///
    /// Performs exponentiation by squaring, where exponentiation in a
    /// finite-field is defined as repeated multiplication. Note that this
    /// is not constant-time even when used in Barret mode!
    ///
    /// Note that since this is defined over a finite-field, it's not actually
    /// possible for this operation to overflow.
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// const X: gf256 = gf256(0x12).naive_pow(3);
    /// assert_eq!(X, gf256(0x12)*gf256(0x12)*gf256(0x12));
    /// assert_eq!(X, gf256(0xbf));
    /// ```
    ///
    #[inline]
    pub const fn naive_pow(self, exp: __u) -> __gf {
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

    /// Naive multiplicative inverse over the finite-field.
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts.
    ///
    /// Returns [`None`] if `other == 0`.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// const X: Option<gf256> = gf256(0x12).naive_checked_recip();
    /// const Y: Option<gf256> = gf256(0x00).naive_checked_recip();
    /// assert_eq!(X, Some(gf256(0xc0)));
    /// assert_eq!(X.unwrap()*gf256(0x12), gf256(0x01));
    /// assert_eq!(Y, None);
    /// ```
    ///
    #[inline]
    pub const fn naive_checked_recip(self) -> Option<__gf> {
        if self.0 == 0 {
            return None;
        }

        // x^-1 = x^255-1 = x^254
        Some(self.naive_pow(__nonzeros-1))
    }

    /// Naive multiplicative inverse over the finite-field.
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts.
    ///
    /// This will panic if `other == 0`.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// const X: gf256 = gf256(0x12).naive_recip();
    /// assert_eq!(X, gf256(0xc0));
    /// assert_eq!(X*gf256(0x12), gf256(0x01));
    /// ```
    ///
    #[inline]
    pub const fn naive_recip(self) -> __gf {
        match self.naive_checked_recip() {
            Some(x) => x,
            None => __gf(1 / 0),
        }
    }

    /// Naive division over the finite-field.
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts.
    ///
    /// Returns [`None`] if `other == 0`.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// const X: Option<gf256> = gf256(0x0f).naive_checked_div(gf256(0x34));
    /// const Y: Option<gf256> = gf256(0x0f).naive_checked_div(gf256(0x00));
    /// assert_eq!(X, Some(gf256(0x12)));
    /// assert_eq!(X.unwrap()*gf256(0x34), gf256(0x0f));
    /// assert_eq!(Y, None);
    /// ```
    ///
    #[inline]
    pub const fn naive_checked_div(self, other: __gf) -> Option<__gf> {
        match other.naive_checked_recip() {
            Some(other_recip) => Some(self.naive_mul(other_recip)),
            None => None,
        }
    }

    /// Naive division over the finite-field.
    ///
    /// Naive versions are built out of simple bitwise operations,
    /// these are more expensive, but also allowed in const contexts.
    ///
    /// This will panic if `other == 0`.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// const X: gf256 = gf256(0x0f).naive_div(gf256(0x34));
    /// assert_eq!(X, gf256(0x12));
    /// assert_eq!(X*gf256(0x34), gf256(0x0f));
    /// ```
    ///
    #[inline]
    pub const fn naive_div(self, other: __gf) -> __gf {
        match self.naive_checked_div(other) {
            Some(x) => x,
            None => __gf(self.0 / 0),
        }
    }

    /// Multiplication over the finite-field.
    ///
    /// Note that since this is defined over a finite-field, it's not actually
    /// possible for this operation to overflow.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// assert_eq!(gf256(0x12) * gf256(0x34), gf256(0x0f));
    /// ```
    ///
    /// One important property of finite-fields, multiplication is distributive
    /// over addition:
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// let a = gf256(0x12);
    /// let b = gf256(0x34);
    /// let c = gf256(0x56);
    /// assert_eq!(a*(b+c), a*b + a*c);
    /// ```
    ///
    #[inline]
    pub fn mul(self, other: __gf) -> __gf {
        cfg_if! {
            if #[cfg(__if(__table))] {
                // multiplication using log/antilog tables
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
                        unsafe { *Self::LOG_TABLE.get_unchecked(self.0 as usize) }
                            .overflowing_add(unsafe { *Self::LOG_TABLE.get_unchecked(other.0 as usize) })
                    {
                        (x, true)                    => x.wrapping_sub(__nonzeros),
                        (x, false) if x > __nonzeros => x.wrapping_sub(__nonzeros),
                        (x, false)                   => x,
                    };
                    __gf(unsafe { *Self::EXP_TABLE.get_unchecked(x as usize) })
                }
            } else if #[cfg(__if(__rem_table))] {
                // multiplication with a per-byte remainder table
                let (mut lo, mut hi) = __p(self.0 << (8*size_of::<__u>()-__width))
                    .widening_mul(__p(other.0));

                let mut x = __p(0);
                for b in hi.to_be_bytes() {
                    cfg_if! {
                        if #[cfg(__if(__width <= 8))] {
                            x = unsafe { *Self::REM_TABLE.get_unchecked(usize::from(
                                x.0 ^ b)) };
                        } else {
                            x = (x << 8) ^ unsafe { *Self::REM_TABLE.get_unchecked(usize::from(
                                ((x >> (8*size_of::<__u>()-8)).0 as u8) ^ b)) };
                        }
                    }
                }

                __gf((x + lo).0 >> (8*size_of::<__u>()-__width))
            } else if #[cfg(__if(__small_rem_table))] {
                // multiplication with a per-nibble remainder table
                let (mut lo, mut hi) = __p(self.0 << (8*size_of::<__u>()-__width)).widening_mul(__p(other.0));

                let mut x = __p(0);
                for b in hi.to_be_bytes() {
                    x = (x << 4) ^ unsafe { *Self::REM_TABLE.get_unchecked(usize::from(
                        (((x >> (8*size_of::<__u>()-4)).0 as u8) ^ (b >> 4)) & 0xf)) };
                    x = (x << 4) ^ unsafe { *Self::REM_TABLE.get_unchecked(usize::from(
                        (((x >> (8*size_of::<__u>()-4)).0 as u8) ^ (b >> 0)) & 0xf)) };
                }

                __gf((x + lo).0 >> (8*size_of::<__u>()-__width))
            } else if #[cfg(__if(__barret))] {
                // multiplication using Barret reduction
                //
                // Barret reduction is a method for turning division/remainder
                // by a constant into multiplication by a couple constants. It's
                // useful here if we have hardware xmul instructions, though
                // it may be more expensive if xmul is naive.
                //
                let (lo, hi) = __p(self.0 << (8*size_of::<__u>()-__width))
                    .widening_mul(__p(other.0));
                let x = lo + (hi.widening_mul(Self::BARRET_CONSTANT).1 + hi)
                    .wrapping_mul(__p((__polynomial & __nonzeros) << (8*size_of::<__u>()-__width)));
                __gf(x.0 >> (8*size_of::<__u>()-__width))
            } else {
                // fallback to naive multiplication
                //
                // Note this is still a bit better than naive_mul, since we
                // use the p-type's non-naive mul, which may be hardware
                // accelerated
                //
                let (lo, hi) = __p(self.0).widening_mul(__p(other.0));
                let x = __p2(((hi.0 as __u2) << (8*size_of::<__u>())) | (lo.0 as __u2))
                    % __p2(__polynomial);
                __gf(x.0 as __u)
            }
        }
    }

    /// Exponentiation over the finite-field.
    ///
    /// Performs exponentiation by squaring, where exponentiation in a
    /// finite-field is defined as repeated multiplication. Note that this
    /// is not constant-time even when used in Barret mode!
    ///
    /// Note that since this is defined over a finite-field, it's not actually
    /// possible for this operation to overflow.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// assert_eq!(gf256(0x12).pow(3), gf256(0x12)*gf256(0x12)*gf256(0x12));
    /// assert_eq!(gf256(0x12).pow(3), gf256(0xbf));
    /// ```
    ///
    #[inline]
    pub fn pow(self, exp: __u) -> __gf {
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
                    let x = (__u2::from(unsafe { *Self::LOG_TABLE.get_unchecked(self.0 as usize) })
                        * __u2::from(exp)) % __nonzeros;
                    __gf(unsafe { *Self::EXP_TABLE.get_unchecked(x as usize) })
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

    /// Multiplicative inverse over the finite-field.
    ///
    /// Returns [`None`] if `other == 0`.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// assert_eq!(gf256(0x12).checked_recip(), Some(gf256(0xc0)));
    /// assert_eq!(gf256(0x12).checked_recip().unwrap()*gf256(0x12), gf256(0x01));
    /// assert_eq!(gf256(0x00).checked_recip(), None);
    /// ```
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
                let x = __nonzeros - unsafe { *Self::LOG_TABLE.get_unchecked(self.0 as usize) };
                Some(__gf(unsafe { *Self::EXP_TABLE.get_unchecked(x as usize) }))
            } else {
                // x^-1 = x^255-1 = x^254
                //
                Some(self.pow(__nonzeros-1))
            }
        }
    }

    /// Naive multiplicative inverse over the finite-field.
    ///
    /// This will panic if `other == 0`.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// assert_eq!(gf256(0x12).recip(), gf256(0xc0));
    /// assert_eq!(gf256(0x12).recip()*gf256(0x12), gf256(0x01));
    /// ```
    ///
    #[inline]
    pub fn recip(self) -> __gf {
        self.checked_recip()
            .expect("gf division by zero")
    }

    /// Division over the finite-field.
    ///
    /// Returns [`None`] if `other == 0`.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// assert_eq!(gf256(0x0f).checked_div(gf256(0x34)), Some(gf256(0x12)));
    /// assert_eq!(gf256(0x0f).checked_div(gf256(0x34)).unwrap()*gf256(0x34), gf256(0x0f));
    /// assert_eq!(gf256(0x0f).checked_div(gf256(0x00)), None);
    /// ```
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
                        unsafe { *Self::LOG_TABLE.get_unchecked(self.0 as usize) }
                            .overflowing_add(__nonzeros - unsafe { *Self::LOG_TABLE.get_unchecked(other.0 as usize) })
                    {
                        (x, true)                    => x.wrapping_sub(__nonzeros),
                        (x, false) if x > __nonzeros => x.wrapping_sub(__nonzeros),
                        (x, false)                   => x,
                    };
                    Some(__gf(unsafe { *Self::EXP_TABLE.get_unchecked(x as usize) }))
                }
            } else {
                // a/b = a*b^1
                //
                Some(self * other.recip())
            }
        }
    }

    /// Division over the finite-field.
    ///
    /// This will panic if `other == 0`.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// assert_eq!(gf256(0x0f) / gf256(0x34), gf256(0x12));
    /// assert_eq!((gf256(0x0f) / gf256(0x34))*gf256(0x34), gf256(0x0f));
    /// ```
    ///
    #[inline]
    pub fn div(self, other: __gf) -> __gf {
        self.checked_div(other)
            .expect("gf division by zero")
    }

    /// Cast slice of unsigned-types to slice of finite-field types.
    ///
    /// This is useful for when you want to view an array of bytes
    /// as an array of finite-field elements without an additional memory
    /// allocation or unsafe code.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// let x: &[u8] = &[0x01, 0x02, 0x03, 0x04, 0x05];
    /// let y: &[gf256] = gf256::slice_from_slice(x);
    /// assert_eq!(y, &[gf256(0x01), gf256(0x02), gf256(0x03), gf256(0x04), gf256(0x05)]);
    /// ```
    ///
    #[cfg(__if(__is_pw2ge8))]
    #[inline]
    pub fn slice_from_slice(slice: &[__u]) -> &[__gf] {
        unsafe {
            slice::from_raw_parts(
                slice.as_ptr() as *const __gf,
                slice.len()
            )
        }
    }

    /// Cast mut slice of unsigned-types to mut slice of finite-field types.
    ///
    /// This is useful for when you want to view an array of bytes
    /// as an array of finite-field elements without an additional memory
    /// allocation or unsafe code.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// let x: &mut [u8] = &mut [0x01, 0x02, 0x03, 0x04, 0x05];
    /// let y: &mut [gf256] = gf256::slice_from_slice_mut(x);
    /// for i in 0..y.len() {
    ///     y[i] *= gf256(0x05);
    /// }
    /// assert_eq!(x, &[0x05, 0x0a, 0x0f, 0x14, 0x11]);
    /// ```
    ///
    #[cfg(__if(__is_pw2ge8))]
    #[inline]
    pub fn slice_from_slice_mut(slice: &mut [__u]) -> &mut [__gf] {
        unsafe {
            slice::from_raw_parts_mut(
                slice.as_mut_ptr() as *mut __gf,
                slice.len()
            )
        }
    }

    /// Cast slice of unsigned-types to slice of finite-field types unsafely.
    ///
    /// This is useful for when you want to view an array of bytes
    /// as an array of finite-field elements without an additional memory
    /// allocation or unsafe code.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// #[gf(polynomial=0x13, generator=0x2)]
    /// type gf16;
    ///
    /// # fn main() {
    /// let x: &[u8] = &[0x01, 0x02, 0x03, 0x04, 0x05];
    /// let y: &[gf16] = unsafe { gf16::slice_from_slice_unchecked(x) };
    /// assert_eq!(y, &[gf16::new(0x1), gf16::new(0x2), gf16::new(0x3), gf16::new(0x4), gf16::new(0x5)]);
    /// # }
    /// ```
    ///
    #[inline]
    pub unsafe fn slice_from_slice_unchecked(slice: &[__u]) -> &[__gf] {
        unsafe {
            slice::from_raw_parts(
                slice.as_ptr() as *const __gf,
                slice.len()
            )
        }
    }

    /// Cast mut slice of unsigned-types to mut slice of finite-field types unsafely.
    ///
    /// This is useful for when you want to view an array of bytes
    /// as an array of finite-field elements without an additional memory
    /// allocation or unsafe code.
    ///
    /// ``` rust
    /// # use ::gf256::*;
    /// # use ::gf256::gf::gf;
    /// #[gf(polynomial=0x13, generator=0x2)]
    /// type gf16;
    ///
    /// # fn main() {
    /// let x: &mut [u8] = &mut [0x01, 0x02, 0x03, 0x04, 0x05];
    /// let y: &mut [gf16] = unsafe { gf16::slice_from_slice_mut_unchecked(x) };
    /// for i in 0..y.len() {
    ///     y[i] *= gf16::new(0x5);
    /// }
    /// assert_eq!(x, &[0x05, 0x0a, 0x0f, 0x07, 0x02]);
    /// # }
    /// ```
    ///
    #[inline]
    pub unsafe fn slice_from_slice_mut_unchecked(slice: &mut [__u]) -> &mut [__gf] {
        unsafe {
            slice::from_raw_parts_mut(
                slice.as_mut_ptr() as *mut __gf,
                slice.len()
            )
        }
    }
}


//// Conversions into __gf ////

#[cfg(__if(__is_pw2ge8))]
impl From<__p> for __gf {
    #[inline]
    fn from(x: __p) -> __gf {
        __gf(x.0)
    }
}

#[cfg(__if(__is_pw2ge8))]
impl From<__u> for __gf {
    #[inline]
    fn from(x: __u) -> __gf {
        __gf(x)
    }
}

impl From<bool> for __gf {
    #[inline]
    fn from(x: bool) -> __gf {
        __gf(__u::from(x))
    }
}

#[cfg(__if(__width >= 32 && !__is_usize))]
impl From<char> for __gf {
    #[inline]
    fn from(x: char) -> __gf {
        __gf(__u::from(x))
    }
}

#[cfg(__if(__width > 8))]
impl From<u8> for __gf {
    #[inline]
    fn from(x: u8) -> __gf {
        __gf(__u::from(x))
    }
}

#[cfg(__if(__width > 16))]
impl From<u16> for __gf {
    #[inline]
    fn from(x: u16) -> __gf {
        __gf(__u::from(x))
    }
}

#[cfg(__if(__width > 32 && !__is_usize))]
impl From<u32> for __gf {
    #[inline]
    fn from(x: u32) -> __gf {
        __gf(__u::from(x))
    }
}

#[cfg(__if(__width > 64 && !__is_usize))]
impl From<u64> for __gf {
    #[inline]
    fn from(x: u64) -> __gf {
        __gf(__u::from(x))
    }
}

#[cfg(__if(__width > 8))]
impl From<__crate::p::p8> for __gf {
    #[inline]
    fn from(x: __crate::p::p8) -> __gf {
        __gf(__u::from(x.0))
    }
}

#[cfg(__if(__width > 16))]
impl From<__crate::p::p16> for __gf {
    #[inline]
    fn from(x: __crate::p::p16) -> __gf {
        __gf(__u::from(x.0))
    }
}

#[cfg(__if(__width > 32 && !__is_usize))]
impl From<__crate::p::p32> for __gf {
    #[inline]
    fn from(x: __crate::p::p32) -> __gf {
        __gf(__u::from(x.0))
    }
}

#[cfg(__if(__width > 64 && !__is_usize))]
impl From<__crate::p::p64> for __gf {
    #[inline]
    fn from(x: __crate::p::p64) -> __gf {
        __gf(__u::from(x.0))
    }
}

#[cfg(__if(__width < 8))]
impl TryFrom<u8> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: u8) -> Result<__gf, Self::Error> {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                Ok(__gf(__u::try_from(x)?))
            } else {
                if x < __nonzeros+1 {
                    Ok(__gf(__u::try_from(x)?))
                } else {
                    // force an error
                    Err(__u::try_from(u128::MAX).unwrap_err())
                }
            }
        }
    }
}

#[cfg(__if(__width < 16))]
impl TryFrom<u16> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: u16) -> Result<__gf, Self::Error> {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                Ok(__gf(__u::try_from(x)?))
            } else {
                if x < __nonzeros+1 {
                    Ok(__gf(__u::try_from(x)?))
                } else {
                    // force an error
                    Err(__u::try_from(u128::MAX).unwrap_err())
                }
            }
        }
    }
}

#[cfg(__if(__width < 32 || __is_usize))]
impl TryFrom<u32> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: u32) -> Result<__gf, Self::Error> {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                Ok(__gf(__u::try_from(x)?))
            } else {
                if x < __nonzeros+1 {
                    Ok(__gf(__u::try_from(x)?))
                } else {
                    // force an error
                    Err(__u::try_from(u128::MAX).unwrap_err())
                }
            }
        }
    }
}

#[cfg(__if(__width < 64 || __is_usize))]
impl TryFrom<u64> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: u64) -> Result<__gf, Self::Error> {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                Ok(__gf(__u::try_from(x)?))
            } else {
                if x < __nonzeros+1 {
                    Ok(__gf(__u::try_from(x)?))
                } else {
                    // force an error
                    Err(__u::try_from(u128::MAX).unwrap_err())
                }
            }
        }
    }
}

#[cfg(__if(__width < 128 || __is_usize))]
impl TryFrom<u128> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: u128) -> Result<__gf, Self::Error> {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                Ok(__gf(__u::try_from(x)?))
            } else {
                if x < __nonzeros+1 {
                    Ok(__gf(__u::try_from(x)?))
                } else {
                    // force an error
                    Err(__u::try_from(u128::MAX).unwrap_err())
                }
            }
        }
    }
}

#[cfg(__if(!__is_usize))]
impl TryFrom<usize> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: usize) -> Result<__gf, Self::Error> {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                Ok(__gf(__u::try_from(x)?))
            } else {
                if x < __nonzeros+1 {
                    Ok(__gf(__u::try_from(x)?))
                } else {
                    // force an error
                    Err(__u::try_from(u128::MAX).unwrap_err())
                }
            }
        }
    }
}

#[cfg(__if(__width < 8))]
impl TryFrom<__crate::p::p8> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __crate::p::p8) -> Result<__gf, Self::Error> {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                Ok(__gf(__u::try_from(x.0)?))
            } else {
                if x.0 < __nonzeros+1 {
                    Ok(__gf(__u::try_from(x.0)?))
                } else {
                    // force an error
                    Err(__u::try_from(u128::MAX).unwrap_err())
                }
            }
        }
    }
}

#[cfg(__if(__width < 16))]
impl TryFrom<__crate::p::p16> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __crate::p::p16) -> Result<__gf, Self::Error> {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                Ok(__gf(__u::try_from(x.0)?))
            } else {
                if x.0 < __nonzeros+1 {
                    Ok(__gf(__u::try_from(x.0)?))
                } else {
                    // force an error
                    Err(__u::try_from(u128::MAX).unwrap_err())
                }
            }
        }
    }
}

#[cfg(__if(__width < 32 || __is_usize))]
impl TryFrom<__crate::p::p32> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __crate::p::p32) -> Result<__gf, Self::Error> {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                Ok(__gf(__u::try_from(x.0)?))
            } else {
                if x.0 < __nonzeros+1 {
                    Ok(__gf(__u::try_from(x.0)?))
                } else {
                    // force an error
                    Err(__u::try_from(u128::MAX).unwrap_err())
                }
            }
        }
    }
}

#[cfg(__if(__width < 64 || __is_usize))]
impl TryFrom<__crate::p::p64> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __crate::p::p64) -> Result<__gf, Self::Error> {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                Ok(__gf(__u::try_from(x.0)?))
            } else {
                if x.0 < __nonzeros+1 {
                    Ok(__gf(__u::try_from(x.0)?))
                } else {
                    // force an error
                    Err(__u::try_from(u128::MAX).unwrap_err())
                }
            }
        }
    }
}

#[cfg(__if(__width < 128 || __is_usize))]
impl TryFrom<__crate::p::p128> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __crate::p::p128) -> Result<__gf, Self::Error> {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                Ok(__gf(__u::try_from(x.0)?))
            } else {
                if x.0 < __nonzeros+1 {
                    Ok(__gf(__u::try_from(x.0)?))
                } else {
                    // force an error
                    Err(__u::try_from(u128::MAX).unwrap_err())
                }
            }
        }
    }
}

#[cfg(__if(!__is_usize))]
impl TryFrom<__crate::p::psize> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __crate::p::psize) -> Result<__gf, Self::Error> {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                Ok(__gf(__u::try_from(x.0)?))
            } else {
                if x.0 < __nonzeros+1 {
                    Ok(__gf(__u::try_from(x.0)?))
                } else {
                    // force an error
                    Err(__u::try_from(u128::MAX).unwrap_err())
                }
            }
        }
    }
}

#[cfg(__if(__width < 8))]
impl FromLossy<u8> for __gf {
    #[inline]
    fn from_lossy(x: u8) -> __gf {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                __gf(x as __u)
            } else {
                __gf((x as __u) & __nonzeros)
            }
        }
    }
}

#[cfg(__if(__width < 16))]
impl FromLossy<u16> for __gf {
    #[inline]
    fn from_lossy(x: u16) -> __gf {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                __gf(x as __u)
            } else {
                __gf((x as __u) & __nonzeros)
            }
        }
    }
}

#[cfg(__if(__width < 32 || __is_usize))]
impl FromLossy<u32> for __gf {
    #[inline]
    fn from_lossy(x: u32) -> __gf {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                __gf(x as __u)
            } else {
                __gf((x as __u) & __nonzeros)
            }
        }
    }
}

#[cfg(__if(__width < 64 || __is_usize))]
impl FromLossy<u64> for __gf {
    #[inline]
    fn from_lossy(x: u64) -> __gf {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                __gf(x as __u)
            } else {
                __gf((x as __u) & __nonzeros)
            }
        }
    }
}

#[cfg(__if(__width < 128 || __is_usize))]
impl FromLossy<u128> for __gf {
    #[inline]
    fn from_lossy(x: u128) -> __gf {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                __gf(x as __u)
            } else {
                __gf((x as __u) & __nonzeros)
            }
        }
    }
}

#[cfg(__if(!__is_usize))]
impl FromLossy<usize> for __gf {
    #[inline]
    fn from_lossy(x: usize) -> __gf {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                __gf(x as __u)
            } else {
                __gf((x as __u) & __nonzeros)
            }
        }
    }
}

#[cfg(__if(__width < 8))]
impl FromLossy<__crate::p::p8> for __gf {
    #[inline]
    fn from_lossy(x: __crate::p::p8) -> __gf {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                __gf(x.0 as __u)
            } else {
                __gf((x.0 as __u) & __nonzeros)
            }
        }
    }
}

#[cfg(__if(__width < 16))]
impl FromLossy<__crate::p::p16> for __gf {
    #[inline]
    fn from_lossy(x: __crate::p::p16) -> __gf {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                __gf(x.0 as __u)
            } else {
                __gf((x.0 as __u) & __nonzeros)
            }
        }
    }
}

#[cfg(__if(__width < 32 || __is_usize))]
impl FromLossy<__crate::p::p32> for __gf {
    #[inline]
    fn from_lossy(x: __crate::p::p32) -> __gf {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                __gf(x.0 as __u)
            } else {
                __gf((x.0 as __u) & __nonzeros)
            }
        }
    }
}

#[cfg(__if(__width < 64 || __is_usize))]
impl FromLossy<__crate::p::p64> for __gf {
    #[inline]
    fn from_lossy(x: __crate::p::p64) -> __gf {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                __gf(x.0 as __u)
            } else {
                __gf((x.0 as __u) & __nonzeros)
            }
        }
    }
}

#[cfg(__if(__width < 128 || __is_usize))]
impl FromLossy<__crate::p::p128> for __gf {
    #[inline]
    fn from_lossy(x: __crate::p::p128) -> __gf {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                __gf(x.0 as __u)
            } else {
                __gf((x.0 as __u) & __nonzeros)
            }
        }
    }
}

#[cfg(__if(!__is_usize))]
impl FromLossy<__crate::p::psize> for __gf {
    #[inline]
    fn from_lossy(x: __crate::p::psize) -> __gf {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                __gf(x.0 as __u)
            } else {
                __gf((x.0 as __u) & __nonzeros)
            }
        }
    }
}

impl TryFrom<i8> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: i8) -> Result<__gf, Self::Error> {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                Ok(__gf(__u::try_from(x)?))
            } else {
                if x < __nonzeros+1 {
                    Ok(__gf(__u::try_from(x)?))
                } else {
                    // force an error
                    Err(__u::try_from(i128::MAX).unwrap_err())
                }
            }
        }
    }
}

impl TryFrom<i16> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: i16) -> Result<__gf, Self::Error> {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                Ok(__gf(__u::try_from(x)?))
            } else {
                if x < __nonzeros+1 {
                    Ok(__gf(__u::try_from(x)?))
                } else {
                    // force an error
                    Err(__u::try_from(i128::MAX).unwrap_err())
                }
            }
        }
    }
}

impl TryFrom<i32> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: i32) -> Result<__gf, Self::Error> {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                Ok(__gf(__u::try_from(x)?))
            } else {
                if x < __nonzeros+1 {
                    Ok(__gf(__u::try_from(x)?))
                } else {
                    // force an error
                    Err(__u::try_from(i128::MAX).unwrap_err())
                }
            }
        }
    }
}

impl TryFrom<i64> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: i64) -> Result<__gf, Self::Error> {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                Ok(__gf(__u::try_from(x)?))
            } else {
                if x < __nonzeros+1 {
                    Ok(__gf(__u::try_from(x)?))
                } else {
                    // force an error
                    Err(__u::try_from(i128::MAX).unwrap_err())
                }
            }
        }
    }
}

impl TryFrom<i128> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: i128) -> Result<__gf, Self::Error> {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                Ok(__gf(__u::try_from(x)?))
            } else {
                if x < __nonzeros+1 {
                    Ok(__gf(__u::try_from(x)?))
                } else {
                    // force an error
                    Err(__u::try_from(i128::MAX).unwrap_err())
                }
            }
        }
    }
}

impl TryFrom<isize> for __gf {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: isize) -> Result<__gf, Self::Error> {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                Ok(__gf(__u::try_from(x)?))
            } else {
                if x < __nonzeros+1 {
                    Ok(__gf(__u::try_from(x)?))
                } else {
                    // force an error
                    Err(__u::try_from(i128::MAX).unwrap_err())
                }
            }
        }
    }
}

impl FromLossy<i8> for __gf {
    #[inline]
    fn from_lossy(x: i8) -> __gf {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                __gf(x as __u)
            } else {
                __gf((x as __u) & __nonzeros)
            }
        }
    }
}

impl FromLossy<i16> for __gf {
    #[inline]
    fn from_lossy(x: i16) -> __gf {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                __gf(x as __u)
            } else {
                __gf((x as __u) & __nonzeros)
            }
        }
    }
}

impl FromLossy<i32> for __gf {
    #[inline]
    fn from_lossy(x: i32) -> __gf {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                __gf(x as __u)
            } else {
                __gf((x as __u) & __nonzeros)
            }
        }
    }
}

impl FromLossy<i64> for __gf {
    #[inline]
    fn from_lossy(x: i64) -> __gf {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                __gf(x as __u)
            } else {
                __gf((x as __u) & __nonzeros)
            }
        }
    }
}

impl FromLossy<i128> for __gf {
    #[inline]
    fn from_lossy(x: i128) -> __gf {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                __gf(x as __u)
            } else {
                __gf((x as __u) & __nonzeros)
            }
        }
    }
}

impl FromLossy<isize> for __gf {
    #[inline]
    fn from_lossy(x: isize) -> __gf {
        cfg_if! {
            if #[cfg(__if(__is_pw2ge8))] {
                __gf(x as __u)
            } else {
                __gf((x as __u) & __nonzeros)
            }
        }
    }
}


//// Conversions from __gf ////

#[cfg(__if(__is_pw2ge8))]
impl From<__gf> for __p {
    #[inline]
    fn from(x: __gf) -> __p {
        __p(x.0)
    }
}

#[cfg(__if(__is_pw2ge8))]
impl From<__gf> for __u {
    #[inline]
    fn from(x: __gf) -> __u {
        x.0
    }
}

#[cfg(__if(__width < 8))]
impl From<__gf> for u8 {
    #[inline]
    fn from(x: __gf) -> u8 {
        u8::from(x.0)
    }
}

#[cfg(__if(__width < 16))]
impl From<__gf> for u16 {
    #[inline]
    fn from(x: __gf) -> u16 {
        u16::from(x.0)
    }
}

#[cfg(__if(__width < 32 && !__is_usize))]
impl From<__gf> for u32 {
    #[inline]
    fn from(x: __gf) -> u32 {
        u32::from(x.0)
    }
}

#[cfg(__if(__width < 64 && !__is_usize))]
impl From<__gf> for u64 {
    #[inline]
    fn from(x: __gf) -> u64 {
        u64::from(x.0)
    }
}

#[cfg(__if(__width < 128 && !__is_usize))]
impl From<__gf> for u128 {
    #[inline]
    fn from(x: __gf) -> u128 {
        u128::from(x.0)
    }
}

#[cfg(__if(__width <= 16 && !__is_usize))]
impl From<__gf> for usize {
    #[inline]
    fn from(x: __gf) -> usize {
        usize::from(x.0)
    }
}

#[cfg(__if(__width > 8))]
impl TryFrom<__gf> for u8 {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __gf) -> Result<u8, Self::Error> {
        u8::try_from(x.0)
    }
}

#[cfg(__if(__width > 16))]
impl TryFrom<__gf> for u16 {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __gf) -> Result<u16, Self::Error> {
        u16::try_from(x.0)
    }
}

#[cfg(__if(__width > 32 || __is_usize))]
impl TryFrom<__gf> for u32 {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __gf) -> Result<u32, Self::Error> {
        u32::try_from(x.0)
    }
}

#[cfg(__if(__width > 64 || __is_usize))]
impl TryFrom<__gf> for u64 {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __gf) -> Result<u64, Self::Error> {
        u64::try_from(x.0)
    }
}

#[cfg(__if(__width > 16 && !__is_usize))]
impl TryFrom<__gf> for usize {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __gf) -> Result<usize, Self::Error> {
        usize::try_from(x.0)
    }
}

#[cfg(__if(__width > 8))]
impl FromLossy<__gf> for u8 {
    #[inline]
    fn from_lossy(x: __gf) -> u8 {
        x.0 as u8
    }
}

#[cfg(__if(__width > 16))]
impl FromLossy<__gf> for u16 {
    #[inline]
    fn from_lossy(x: __gf) -> u16 {
        x.0 as u16
    }
}

#[cfg(__if(__width > 32 || __is_usize))]
impl FromLossy<__gf> for u32 {
    #[inline]
    fn from_lossy(x: __gf) -> u32 {
        x.0 as u32
    }
}

#[cfg(__if(__width > 64 || __is_usize))]
impl FromLossy<__gf> for u64 {
    #[inline]
    fn from_lossy(x: __gf) -> u64 {
        x.0 as u64
    }
}

#[cfg(__if(__width > 16 && !__is_usize))]
impl FromLossy<__gf> for usize {
    #[inline]
    fn from_lossy(x: __gf) -> usize {
        x.0 as usize
    }
}

#[cfg(__if(__width < 8))]
impl From<__gf> for __crate::p::p8 {
    #[inline]
    fn from(x: __gf) -> __crate::p::p8 {
        __crate::p::p8(u8::from(x.0))
    }
}

#[cfg(__if(__width < 16))]
impl From<__gf> for __crate::p::p16 {
    #[inline]
    fn from(x: __gf) -> __crate::p::p16 {
        __crate::p::p16(u16::from(x.0))
    }
}

#[cfg(__if(__width < 32 && !__is_usize))]
impl From<__gf> for __crate::p::p32 {
    #[inline]
    fn from(x: __gf) -> __crate::p::p32 {
        __crate::p::p32(u32::from(x.0))
    }
}

#[cfg(__if(__width < 64 && !__is_usize))]
impl From<__gf> for __crate::p::p64 {
    #[inline]
    fn from(x: __gf) -> __crate::p::p64 {
        __crate::p::p64(u64::from(x.0))
    }
}

#[cfg(__if(__width < 128 && !__is_usize))]
impl From<__gf> for __crate::p::p128 {
    #[inline]
    fn from(x: __gf) -> __crate::p::p128 {
        __crate::p::p128(u128::from(x.0))
    }
}

#[cfg(__if(__width <= 16 && !__is_usize))]
impl From<__gf> for __crate::p::psize {
    #[inline]
    fn from(x: __gf) -> __crate::p::psize {
        __crate::p::psize(usize::from(x.0))
    }
}

#[cfg(__if(__width > 8))]
impl TryFrom<__gf> for __crate::p::p8 {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __gf) -> Result<__crate::p::p8, Self::Error> {
        Ok(__crate::p::p8(u8::try_from(x.0)?))
    }
}

#[cfg(__if(__width > 16))]
impl TryFrom<__gf> for __crate::p::p16 {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __gf) -> Result<__crate::p::p16, Self::Error> {
        Ok(__crate::p::p16(u16::try_from(x.0)?))
    }
}

#[cfg(__if(__width > 32 || __is_usize))]
impl TryFrom<__gf> for __crate::p::p32 {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __gf) -> Result<__crate::p::p32, Self::Error> {
        Ok(__crate::p::p32(u32::try_from(x.0)?))
    }
}

#[cfg(__if(__width > 64 || __is_usize))]
impl TryFrom<__gf> for __crate::p::p64 {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __gf) -> Result<__crate::p::p64, Self::Error> {
        Ok(__crate::p::p64(u64::try_from(x.0)?))
    }
}

#[cfg(__if(__width > 16 && !__is_usize))]
impl TryFrom<__gf> for __crate::p::psize {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __gf) -> Result<__crate::p::psize, Self::Error> {
        Ok(__crate::p::psize(usize::try_from(x.0)?))
    }
}

#[cfg(__if(__width > 8))]
impl FromLossy<__gf> for __crate::p::p8 {
    #[inline]
    fn from_lossy(x: __gf) -> __crate::p::p8 {
        __crate::p::p8(x.0 as u8)
    }
}

#[cfg(__if(__width > 16))]
impl FromLossy<__gf> for __crate::p::p16 {
    #[inline]
    fn from_lossy(x: __gf) -> __crate::p::p16 {
        __crate::p::p16(x.0 as u16)
    }
}

#[cfg(__if(__width > 32 || __is_usize))]
impl FromLossy<__gf> for __crate::p::p32 {
    #[inline]
    fn from_lossy(x: __gf) -> __crate::p::p32 {
        __crate::p::p32(x.0 as u32)
    }
}

#[cfg(__if(__width > 64 || __is_usize))]
impl FromLossy<__gf> for __crate::p::p64 {
    #[inline]
    fn from_lossy(x: __gf) -> __crate::p::p64 {
        __crate::p::p64(x.0 as u64)
    }
}

#[cfg(__if(__width > 16 && !__is_usize))]
impl FromLossy<__gf> for __crate::p::psize {
    #[inline]
    fn from_lossy(x: __gf) -> __crate::p::psize {
        __crate::p::psize(x.0 as usize)
    }
}

#[cfg(__if(__width < 8))]
impl From<__gf> for i8 {
    #[inline]
    fn from(x: __gf) -> i8 {
        x.0 as i8
    }
}

#[cfg(__if(__width < 16))]
impl From<__gf> for i16 {
    #[inline]
    fn from(x: __gf) -> i16 {
        x.0 as i16
    }
}

#[cfg(__if(__width < 32 && !__is_usize))]
impl From<__gf> for i32 {
    #[inline]
    fn from(x: __gf) -> i32 {
        x.0 as i32
    }
}

#[cfg(__if(__width < 64 && !__is_usize))]
impl From<__gf> for i64 {
    #[inline]
    fn from(x: __gf) -> i64 {
        x.0 as i64
    }
}

#[cfg(__if(__width < 128 && !__is_usize))]
impl From<__gf> for i128 {
    #[inline]
    fn from(x: __gf) -> i128 {
        x.0 as i128
    }
}

#[cfg(__if(__width < 16 && !__is_usize))]
impl From<__gf> for isize {
    #[inline]
    fn from(x: __gf) -> isize {
        x.0 as isize
    }
}

#[cfg(__if(__width >= 8))]
impl TryFrom<__gf> for i8 {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __gf) -> Result<i8, Self::Error> {
        i8::try_from(x.0)
    }
}

#[cfg(__if(__width >= 16))]
impl TryFrom<__gf> for i16 {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __gf) -> Result<i16, Self::Error> {
        i16::try_from(x.0)
    }
}

#[cfg(__if(__width >= 32 || __is_usize))]
impl TryFrom<__gf> for i32 {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __gf) -> Result<i32, Self::Error> {
        i32::try_from(x.0)
    }
}

#[cfg(__if(__width >= 64 || __is_usize))]
impl TryFrom<__gf> for i64 {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __gf) -> Result<i64, Self::Error> {
        i64::try_from(x.0)
    }
}

#[cfg(__if(__width >= 128 || __is_usize))]
impl TryFrom<__gf> for i128 {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __gf) -> Result<i128, Self::Error> {
        i128::try_from(x.0)
    }
}

#[cfg(__if(__width >= 16))]
impl TryFrom<__gf> for isize {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(x: __gf) -> Result<isize, Self::Error> {
        isize::try_from(x.0)
    }
}

#[cfg(__if(__width >= 8))]
impl FromLossy<__gf> for i8 {
    #[inline]
    fn from_lossy(x: __gf) -> i8 {
        x.0 as i8
    }
}

#[cfg(__if(__width >= 16))]
impl FromLossy<__gf> for i16 {
    #[inline]
    fn from_lossy(x: __gf) -> i16 {
        x.0 as i16
    }
}

#[cfg(__if(__width >= 32 || __is_usize))]
impl FromLossy<__gf> for i32 {
    #[inline]
    fn from_lossy(x: __gf) -> i32 {
        x.0 as i32
    }
}

#[cfg(__if(__width >= 64 || __is_usize))]
impl FromLossy<__gf> for i64 {
    #[inline]
    fn from_lossy(x: __gf) -> i64 {
        x.0 as i64
    }
}

#[cfg(__if(__width >= 128 || __is_usize))]
impl FromLossy<__gf> for i128 {
    #[inline]
    fn from_lossy(x: __gf) -> i128 {
        x.0 as i128
    }
}

#[cfg(__if(__width >= 16))]
impl FromLossy<__gf> for isize {
    #[inline]
    fn from_lossy(x: __gf) -> isize {
        x.0 as isize
    }
}


//// Negate ////

impl Neg for __gf {
    type Output = __gf;
    // Negate is a noop for polynomials
    #[inline]
    fn neg(self) -> __gf {
        self
    }
}

impl Neg for &__gf {
    type Output = __gf;
    // Negate is a noop for polynomials
    #[inline]
    fn neg(self) -> __gf {
        *self
    }
}


//// Addition ////

impl Add<__gf> for __gf {
    type Output = __gf;
    #[inline]
    fn add(self, other: __gf) -> __gf {
        __gf::add(self, other)
    }
}

impl Add<__gf> for &__gf {
    type Output = __gf;
    #[inline]
    fn add(self, other: __gf) -> __gf {
        __gf::add(*self, other)
    }
}

impl Add<&__gf> for __gf {
    type Output = __gf;
    #[inline]
    fn add(self, other: &__gf) -> __gf {
        __gf::add(self, *other)
    }
}

impl Add<&__gf> for &__gf {
    type Output = __gf;
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
    #[inline]
    fn sub(self, other: __gf) -> __gf {
        __gf::sub(self, other)
    }
}

impl Sub<__gf> for &__gf {
    type Output = __gf;
    #[inline]
    fn sub(self, other: __gf) -> __gf {
        __gf::sub(*self, other)
    }
}

impl Sub<&__gf> for __gf {
    type Output = __gf;
    #[inline]
    fn sub(self, other: &__gf) -> __gf {
        __gf::sub(self, *other)
    }
}

impl Sub<&__gf> for &__gf {
    type Output = __gf;
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
    #[inline]
    fn mul(self, other: __gf) -> __gf {
        __gf::mul(self, other)
    }
}

impl Mul<__gf> for &__gf {
    type Output = __gf;
    #[inline]
    fn mul(self, other: __gf) -> __gf {
        __gf::mul(*self, other)
    }
}

impl Mul<&__gf> for __gf {
    type Output = __gf;
    #[inline]
    fn mul(self, other: &__gf) -> __gf {
        __gf::mul(self, *other)
    }
}

impl Mul<&__gf> for &__gf {
    type Output = __gf;
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

impl BitAnd<__gf> for __u {
    type Output = __gf;
    #[inline]
    fn bitand(self, other: __gf) -> __gf {
        __gf(self & other.0)
    }
}

impl BitAnd<__gf> for &__u {
    type Output = __gf;
    #[inline]
    fn bitand(self, other: __gf) -> __gf {
        __gf(self & other.0)
    }
}

impl BitAnd<&__gf> for __u {
    type Output = __gf;
    #[inline]
    fn bitand(self, other: &__gf) -> __gf {
        __gf(self & other.0)
    }
}

impl BitAnd<&__gf> for &__u {
    type Output = __gf;
    #[inline]
    fn bitand(self, other: &__gf) -> __gf {
        __gf(self & other.0)
    }
}

impl BitAnd<__u> for __gf {
    type Output = __gf;
    #[inline]
    fn bitand(self, other: __u) -> __gf {
        __gf(self.0 & other)
    }
}

impl BitAnd<__u> for &__gf {
    type Output = __gf;
    #[inline]
    fn bitand(self, other: __u) -> __gf {
        __gf(self.0 & other)
    }
}

impl BitAnd<&__u> for __gf {
    type Output = __gf;
    #[inline]
    fn bitand(self, other: &__u) -> __gf {
        __gf(self.0 & other)
    }
}

impl BitAnd<&__u> for &__gf {
    type Output = __gf;
    #[inline]
    fn bitand(self, other: &__u) -> __gf {
        __gf(self.0 & other)
    }
}

impl BitAndAssign<__u> for __gf {
    #[inline]
    fn bitand_assign(&mut self, other: __u) {
        *self = *self & other;
    }
}

impl BitAndAssign<&__u> for __gf {
    #[inline]
    fn bitand_assign(&mut self, other: &__u) {
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

impl BitOr<__gf> for __u {
    type Output = __gf;
    #[inline]
    fn bitor(self, other: __gf) -> __gf {
        __gf(self | other.0)
    }
}

impl BitOr<__gf> for &__u {
    type Output = __gf;
    #[inline]
    fn bitor(self, other: __gf) -> __gf {
        __gf(self | other.0)
    }
}

impl BitOr<&__gf> for __u {
    type Output = __gf;
    #[inline]
    fn bitor(self, other: &__gf) -> __gf {
        __gf(self | other.0)
    }
}

impl BitOr<&__gf> for &__u {
    type Output = __gf;
    #[inline]
    fn bitor(self, other: &__gf) -> __gf {
        __gf(self | other.0)
    }
}

impl BitOr<__u> for __gf {
    type Output = __gf;
    #[inline]
    fn bitor(self, other: __u) -> __gf {
        __gf(self.0 | other)
    }
}

impl BitOr<__u> for &__gf {
    type Output = __gf;
    #[inline]
    fn bitor(self, other: __u) -> __gf {
        __gf(self.0 | other)
    }
}

impl BitOr<&__u> for __gf {
    type Output = __gf;
    #[inline]
    fn bitor(self, other: &__u) -> __gf {
        __gf(self.0 | other)
    }
}

impl BitOr<&__u> for &__gf {
    type Output = __gf;
    #[inline]
    fn bitor(self, other: &__u) -> __gf {
        __gf(self.0 | other)
    }
}

impl BitOrAssign<__u> for __gf {
    #[inline]
    fn bitor_assign(&mut self, other: __u) {
        *self = *self | other;
    }
}

impl BitOrAssign<&__u> for __gf {
    #[inline]
    fn bitor_assign(&mut self, other: &__u) {
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

impl BitXor<__gf> for __u {
    type Output = __gf;
    #[inline]
    fn bitxor(self, other: __gf) -> __gf {
        __gf(self ^ other.0)
    }
}

impl BitXor<__gf> for &__u {
    type Output = __gf;
    #[inline]
    fn bitxor(self, other: __gf) -> __gf {
        __gf(self ^ other.0)
    }
}

impl BitXor<&__gf> for __u {
    type Output = __gf;
    #[inline]
    fn bitxor(self, other: &__gf) -> __gf {
        __gf(self ^ other.0)
    }
}

impl BitXor<&__gf> for &__u {
    type Output = __gf;
    #[inline]
    fn bitxor(self, other: &__gf) -> __gf {
        __gf(self ^ other.0)
    }
}

impl BitXor<__u> for __gf {
    type Output = __gf;
    #[inline]
    fn bitxor(self, other: __u) -> __gf {
        __gf(self.0 ^ other)
    }
}

impl BitXor<__u> for &__gf {
    type Output = __gf;
    #[inline]
    fn bitxor(self, other: __u) -> __gf {
        __gf(self.0 ^ other)
    }
}

impl BitXor<&__u> for __gf {
    type Output = __gf;
    #[inline]
    fn bitxor(self, other: &__u) -> __gf {
        __gf(self.0 ^ other)
    }
}

impl BitXor<&__u> for &__gf {
    type Output = __gf;
    #[inline]
    fn bitxor(self, other: &__u) -> __gf {
        __gf(self.0 ^ other)
    }
}

impl BitXorAssign<__u> for __gf {
    #[inline]
    fn bitxor_assign(&mut self, other: __u) {
        *self = *self ^ other;
    }
}

impl BitXorAssign<&__u> for __gf {
    #[inline]
    fn bitxor_assign(&mut self, other: &__u) {
        *self = *self ^ *other;
    }
}


//// Byte order ////

impl __gf {
    #[inline]
    pub const fn swap_bytes(self) -> __gf {
        __gf(self.0.swap_bytes())
    }

    #[inline]
    pub const fn to_le(self) -> __gf {
        __gf(self.0.to_le())
    }

    #[inline]
    pub const fn from_le(self_: __gf) -> __gf {
        __gf(__u::from_le(self_.0))
    }

    #[inline]
    pub const fn to_le_bytes(self) -> [u8; size_of::<__u>()] {
        self.0.to_le_bytes()
    }

    #[inline]
    pub const fn from_le_bytes(bytes: [u8; size_of::<__u>()]) -> __gf {
        __gf(__u::from_le_bytes(bytes))
    }

    #[inline]
    pub const fn to_be(self) -> __gf {
        __gf(self.0.to_be())
    }

    #[inline]
    pub const fn from_be(self_: __gf) -> __gf {
        __gf(__u::from_be(self_.0))
    }

    #[inline]
    pub const fn to_be_bytes(self) -> [u8; size_of::<__u>()] {
        self.0.to_be_bytes()
    }

    #[inline]
    pub const fn from_be_bytes(bytes: [u8; size_of::<__u>()]) -> __gf {
        __gf(__u::from_be_bytes(bytes))
    }

    #[inline]
    pub const fn to_ne_bytes(self) -> [u8; size_of::<__u>()] {
        self.0.to_ne_bytes()
    }

    #[inline]
    pub const fn from_ne_bytes(bytes: [u8; size_of::<__u>()]) -> __gf {
        __gf(__u::from_ne_bytes(bytes))
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
    #[inline]
    fn shl(self, other: u8) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<u8> for &__gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: u8) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&u8> for __gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: &u8) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&u8> for &__gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: &u8) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<u16> for __gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: u16) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<u16> for &__gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: u16) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&u16> for __gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: &u16) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&u16> for &__gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: &u16) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<u32> for __gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: u32) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<u32> for &__gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: u32) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&u32> for __gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: &u32) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&u32> for &__gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: &u32) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<u64> for __gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: u64) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<u64> for &__gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: u64) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&u64> for __gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: &u64) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&u64> for &__gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: &u64) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<u128> for __gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: u128) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<u128> for &__gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: u128) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&u128> for __gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: &u128) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&u128> for &__gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: &u128) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<usize> for __gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: usize) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<usize> for &__gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: usize) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&usize> for __gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: &usize) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&usize> for &__gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: &usize) -> __gf {
        __gf(self.0 << other)
    }
}

impl ShlAssign<u8> for __gf {
    #[inline]
    fn shl_assign(&mut self, other: u8) {
        *self = *self << other;
    }
}

impl ShlAssign<&u8> for __gf {
    #[inline]
    fn shl_assign(&mut self, other: &u8) {
        *self = *self << other;
    }
}

impl ShlAssign<u16> for __gf {
    #[inline]
    fn shl_assign(&mut self, other: u16) {
        *self = *self << other;
    }
}

impl ShlAssign<&u16> for __gf {
    #[inline]
    fn shl_assign(&mut self, other: &u16) {
        *self = *self << other;
    }
}

impl ShlAssign<u32> for __gf {
    #[inline]
    fn shl_assign(&mut self, other: u32) {
        *self = *self << other;
    }
}

impl ShlAssign<&u32> for __gf {
    #[inline]
    fn shl_assign(&mut self, other: &u32) {
        *self = *self << other;
    }
}

impl ShlAssign<u64> for __gf {
    #[inline]
    fn shl_assign(&mut self, other: u64) {
        *self = *self << other;
    }
}

impl ShlAssign<&u64> for __gf {
    #[inline]
    fn shl_assign(&mut self, other: &u64) {
        *self = *self << other;
    }
}

impl ShlAssign<u128> for __gf {
    #[inline]
    fn shl_assign(&mut self, other: u128) {
        *self = *self << other;
    }
}

impl ShlAssign<&u128> for __gf {
    #[inline]
    fn shl_assign(&mut self, other: &u128) {
        *self = *self << other;
    }
}

impl ShlAssign<usize> for __gf {
    #[inline]
    fn shl_assign(&mut self, other: usize) {
        *self = *self << other;
    }
}

impl ShlAssign<&usize> for __gf {
    #[inline]
    fn shl_assign(&mut self, other: &usize) {
        *self = *self << other;
    }
}

impl Shr<u8> for __gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: u8) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<u8> for &__gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: u8) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&u8> for __gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: &u8) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&u8> for &__gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: &u8) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<u16> for __gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: u16) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<u16> for &__gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: u16) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&u16> for __gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: &u16) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&u16> for &__gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: &u16) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<u32> for __gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: u32) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<u32> for &__gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: u32) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&u32> for __gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: &u32) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&u32> for &__gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: &u32) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<u64> for __gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: u64) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<u64> for &__gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: u64) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&u64> for __gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: &u64) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&u64> for &__gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: &u64) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<u128> for __gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: u128) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<u128> for &__gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: u128) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&u128> for __gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: &u128) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&u128> for &__gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: &u128) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<usize> for __gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: usize) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<usize> for &__gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: usize) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&usize> for __gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: &usize) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&usize> for &__gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: &usize) -> __gf {
        __gf(self.0 >> other)
    }
}

impl ShrAssign<u8> for __gf {
    #[inline]
    fn shr_assign(&mut self, other: u8) {
        *self = *self >> other;
    }
}

impl ShrAssign<&u8> for __gf {
    #[inline]
    fn shr_assign(&mut self, other: &u8) {
        *self = *self >> other;
    }
}

impl ShrAssign<u16> for __gf {
    #[inline]
    fn shr_assign(&mut self, other: u16) {
        *self = *self >> other;
    }
}

impl ShrAssign<&u16> for __gf {
    #[inline]
    fn shr_assign(&mut self, other: &u16) {
        *self = *self >> other;
    }
}

impl ShrAssign<u32> for __gf {
    #[inline]
    fn shr_assign(&mut self, other: u32) {
        *self = *self >> other;
    }
}

impl ShrAssign<&u32> for __gf {
    #[inline]
    fn shr_assign(&mut self, other: &u32) {
        *self = *self >> other;
    }
}

impl ShrAssign<u64> for __gf {
    #[inline]
    fn shr_assign(&mut self, other: u64) {
        *self = *self >> other;
    }
}

impl ShrAssign<&u64> for __gf {
    #[inline]
    fn shr_assign(&mut self, other: &u64) {
        *self = *self >> other;
    }
}

impl ShrAssign<u128> for __gf {
    #[inline]
    fn shr_assign(&mut self, other: u128) {
        *self = *self >> other;
    }
}

impl ShrAssign<&u128> for __gf {
    #[inline]
    fn shr_assign(&mut self, other: &u128) {
        *self = *self >> other;
    }
}

impl ShrAssign<usize> for __gf {
    #[inline]
    fn shr_assign(&mut self, other: usize) {
        *self = *self >> other;
    }
}

impl ShrAssign<&usize> for __gf {
    #[inline]
    fn shr_assign(&mut self, other: &usize) {
        *self = *self >> other;
    }
}

impl Shl<i8> for __gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: i8) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<i8> for &__gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: i8) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&i8> for __gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: &i8) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&i8> for &__gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: &i8) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<i16> for __gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: i16) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<i16> for &__gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: i16) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&i16> for __gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: &i16) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&i16> for &__gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: &i16) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<i32> for __gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: i32) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<i32> for &__gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: i32) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&i32> for __gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: &i32) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&i32> for &__gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: &i32) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<i64> for __gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: i64) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<i64> for &__gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: i64) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&i64> for __gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: &i64) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&i64> for &__gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: &i64) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<i128> for __gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: i128) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<i128> for &__gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: i128) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&i128> for __gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: &i128) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&i128> for &__gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: &i128) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<isize> for __gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: isize) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<isize> for &__gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: isize) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&isize> for __gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: &isize) -> __gf {
        __gf(self.0 << other)
    }
}

impl Shl<&isize> for &__gf {
    type Output = __gf;
    #[inline]
    fn shl(self, other: &isize) -> __gf {
        __gf(self.0 << other)
    }
}

impl ShlAssign<i8> for __gf {
    #[inline]
    fn shl_assign(&mut self, other: i8) {
        *self = *self << other;
    }
}

impl ShlAssign<&i8> for __gf {
    #[inline]
    fn shl_assign(&mut self, other: &i8) {
        *self = *self << other;
    }
}

impl ShlAssign<i16> for __gf {
    #[inline]
    fn shl_assign(&mut self, other: i16) {
        *self = *self << other;
    }
}

impl ShlAssign<&i16> for __gf {
    #[inline]
    fn shl_assign(&mut self, other: &i16) {
        *self = *self << other;
    }
}

impl ShlAssign<i32> for __gf {
    #[inline]
    fn shl_assign(&mut self, other: i32) {
        *self = *self << other;
    }
}

impl ShlAssign<&i32> for __gf {
    #[inline]
    fn shl_assign(&mut self, other: &i32) {
        *self = *self << other;
    }
}

impl ShlAssign<i64> for __gf {
    #[inline]
    fn shl_assign(&mut self, other: i64) {
        *self = *self << other;
    }
}

impl ShlAssign<&i64> for __gf {
    #[inline]
    fn shl_assign(&mut self, other: &i64) {
        *self = *self << other;
    }
}

impl ShlAssign<i128> for __gf {
    #[inline]
    fn shl_assign(&mut self, other: i128) {
        *self = *self << other;
    }
}

impl ShlAssign<&i128> for __gf {
    #[inline]
    fn shl_assign(&mut self, other: &i128) {
        *self = *self << other;
    }
}

impl ShlAssign<isize> for __gf {
    #[inline]
    fn shl_assign(&mut self, other: isize) {
        *self = *self << other;
    }
}

impl ShlAssign<&isize> for __gf {
    #[inline]
    fn shl_assign(&mut self, other: &isize) {
        *self = *self << other;
    }
}

impl Shr<i8> for __gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: i8) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<i8> for &__gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: i8) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&i8> for __gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: &i8) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&i8> for &__gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: &i8) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<i16> for __gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: i16) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<i16> for &__gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: i16) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&i16> for __gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: &i16) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&i16> for &__gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: &i16) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<i32> for __gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: i32) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<i32> for &__gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: i32) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&i32> for __gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: &i32) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&i32> for &__gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: &i32) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<i64> for __gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: i64) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<i64> for &__gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: i64) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&i64> for __gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: &i64) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&i64> for &__gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: &i64) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<i128> for __gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: i128) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<i128> for &__gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: i128) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&i128> for __gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: &i128) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&i128> for &__gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: &i128) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<isize> for __gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: isize) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<isize> for &__gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: isize) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&isize> for __gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: &isize) -> __gf {
        __gf(self.0 >> other)
    }
}

impl Shr<&isize> for &__gf {
    type Output = __gf;
    #[inline]
    fn shr(self, other: &isize) -> __gf {
        __gf(self.0 >> other)
    }
}

impl ShrAssign<i8> for __gf {
    #[inline]
    fn shr_assign(&mut self, other: i8) {
        *self = *self >> other;
    }
}

impl ShrAssign<&i8> for __gf {
    #[inline]
    fn shr_assign(&mut self, other: &i8) {
        *self = *self >> other;
    }
}

impl ShrAssign<i16> for __gf {
    #[inline]
    fn shr_assign(&mut self, other: i16) {
        *self = *self >> other;
    }
}

impl ShrAssign<&i16> for __gf {
    #[inline]
    fn shr_assign(&mut self, other: &i16) {
        *self = *self >> other;
    }
}

impl ShrAssign<i32> for __gf {
    #[inline]
    fn shr_assign(&mut self, other: i32) {
        *self = *self >> other;
    }
}

impl ShrAssign<&i32> for __gf {
    #[inline]
    fn shr_assign(&mut self, other: &i32) {
        *self = *self >> other;
    }
}

impl ShrAssign<i64> for __gf {
    #[inline]
    fn shr_assign(&mut self, other: i64) {
        *self = *self >> other;
    }
}

impl ShrAssign<&i64> for __gf {
    #[inline]
    fn shr_assign(&mut self, other: &i64) {
        *self = *self >> other;
    }
}

impl ShrAssign<i128> for __gf {
    #[inline]
    fn shr_assign(&mut self, other: i128) {
        *self = *self >> other;
    }
}

impl ShrAssign<&i128> for __gf {
    #[inline]
    fn shr_assign(&mut self, other: &i128) {
        *self = *self >> other;
    }
}

impl ShrAssign<isize> for __gf {
    #[inline]
    fn shr_assign(&mut self, other: isize) {
        *self = *self >> other;
    }
}

impl ShrAssign<&isize> for __gf {
    #[inline]
    fn shr_assign(&mut self, other: &isize) {
        *self = *self >> other;
    }
}


//// To/from strings ////

impl fmt::Debug for __gf {
    /// We use LowerHex for Debug, since this is a more useful representation
    /// of binary polynomials.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}(0x{:0w$x})", stringify!(__gf), self.0, w=__width/4)
    }
}

impl fmt::Display for __gf {
    /// We use LowerHex for Display since this is a more useful representation
    /// of binary polynomials.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "0x{:0w$x}", self.0, w=__width/4)
    }
}

impl fmt::Binary for __gf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        <__u as fmt::Binary>::fmt(&self.0, f)
    }
}

impl fmt::Octal for __gf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        <__u as fmt::Octal>::fmt(&self.0, f)
    }
}

impl fmt::LowerHex for __gf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        <__u as fmt::LowerHex>::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for __gf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        <__u as fmt::UpperHex>::fmt(&self.0, f)
    }
}

impl FromStr for __gf {
    type Err = ParseIntError;

    /// In order to match Display, this `from_str` takes and only takes
    /// hexadecimal strings starting with `0x`. If you need a different radix
    /// there is [`from_str_radix`](#method.from_str_radix).
    fn from_str(s: &str) -> Result<__gf, ParseIntError> {
        if s.starts_with("0x") {
            Ok(__gf(__u::from_str_radix(&s[2..], 16)?))
        } else {
            "".parse::<__u>()?;
            unreachable!()
        }
    }
}

impl __gf {
    pub fn from_str_radix(s: &str, radix: u32) -> Result<__gf, ParseIntError> {
        Ok(__gf(__u::from_str_radix(s, radix)?))
    }
}
