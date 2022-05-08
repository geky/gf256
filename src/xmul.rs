//! Hardware xmul implementations if available
//!
//! These are declared here in order to be able to leverage unstable
//! features on nightly (if the feature nightly-features is provided).
//! Most of gf256 is provided as proc_macros, and those can't use unstable
//! features unless the feature is enabled with `#[feature!]` at the crate
//! level.
//!
//! These functions are intended to only be used by gf256's proc_macros,
//! these funcitons may or may not be available depending on target_features,
//! and may change behavior, so they shouldn't be used directly.
//!

use cfg_if::cfg_if;


/// A flag indicating if hardware carry-less multiplication
/// instructions are available.
///
/// If this is false, any carry-less multiplication operations
/// will use a more expensive bitwise implementation.
///
/// Some algorithms trade expensive division/remainder operations for
/// multiple multiplication operations, but this can backfire if
/// multiplication is also expensive. This flag allows algorithms
/// to choose the best strategy based on what's available.
///
pub const HAS_XMUL: bool = {
    cfg_if! {
        if #[cfg(any(
            all(
                not(feature="no-xmul"),
                target_arch="x86_64",
                target_feature="pclmulqdq"
            ),
            all(
                not(feature="no-xmul"),
                target_arch="aarch64",
                target_feature="neon"
            )
        ))] {
            true
        } else {
            false
        }
    }
};


/// Widening carry-less multiplication, if hardware instructions are available
///
/// Result is a tuple (lo, hi)
///
#[cfg(any(
    all(
        not(feature="no-xmul"),
        target_arch="x86_64",
        target_feature="pclmulqdq"
    ),
    all(
        not(feature="no-xmul"),
        target_arch="aarch64",
        target_feature="neon"
    )
))]
#[inline]
pub fn xmul8(a: u8, b: u8) -> (u8, u8) {
    cfg_if! {
        if #[cfg(all(
            not(feature="no-xmul"),
            target_arch="x86_64",
            target_feature="pclmulqdq"
        ))] {
            // x86_64 provides 64-bit xmul via the pclmulqdq instruction
            use core::arch::x86_64::*;
            unsafe {
                let a = _mm_set_epi64x(0, a as i64);
                let b = _mm_set_epi64x(0, b as i64);
                let x = _mm_clmulepi64_si128::<0>(a, b);
                let lo = _mm_extract_epi64::<0>(x) as u64;
                (lo as u8, (lo >> 8) as u8)
            }
        } else if #[cfg(all(
            not(feature="no-xmul"),
            target_arch="aarch64",
            target_feature="neon"
        ))] {
            // aarch64 provides 64-bit xmul via the pmull instruction
            use core::arch::aarch64::*;
            unsafe {
                let x = vmull_p64(a as u64, b as u64);
                (x as u8, (x >> 8) as u8)
            }
        }
    }
}

/// Widening carry-less multiplication, if hardware instructions are available
///
/// Result is a tuple (lo, hi)
///
#[cfg(any(
    all(
        not(feature="no-xmul"),
        target_arch="x86_64",
        target_feature="pclmulqdq"
    ),
    all(
        not(feature="no-xmul"),
        target_arch="aarch64",
        target_feature="neon"
    )
))]
#[inline]
pub fn xmul16(a: u16, b: u16) -> (u16, u16) {
    cfg_if! {
        if #[cfg(all(
            not(feature="no-xmul"),
            target_arch="x86_64",
            target_feature="pclmulqdq"
        ))] {
            // x86_64 provides 64-bit xmul via the pclmulqdq instruction
            use core::arch::x86_64::*;
            unsafe {
                let a = _mm_set_epi64x(0, a as i64);
                let b = _mm_set_epi64x(0, b as i64);
                let x = _mm_clmulepi64_si128::<0>(a, b);
                let lo = _mm_extract_epi64::<0>(x) as u64;
                (lo as u16, (lo >> 16) as u16)
            }
        } else if #[cfg(all(
            not(feature="no-xmul"),
            target_arch="aarch64",
            target_feature="neon"
        ))] {
            // aarch64 provides 64-bit xmul via the pmull instruction
            use core::arch::aarch64::*;
            unsafe {
                let x = vmull_p64(a as u64, b as u64);
                (x as u16, (x >> 16) as u16)
            }
        }
    }
}

/// Widening carry-less multiplication, if hardware instructions are available
///
/// Result is a tuple (lo, hi)
///
#[cfg(any(
    all(
        not(feature="no-xmul"),
        target_arch="x86_64",
        target_feature="pclmulqdq"
    ),
    all(
        not(feature="no-xmul"),
        target_arch="aarch64",
        target_feature="neon"
    )
))]
#[inline]
pub fn xmul32(a: u32, b: u32) -> (u32, u32) {
    cfg_if! {
        if #[cfg(all(
            not(feature="no-xmul"),
            target_arch="x86_64",
            target_feature="pclmulqdq"
        ))] {
            // x86_64 provides 64-bit xmul via the pclmulqdq instruction
            use core::arch::x86_64::*;
            unsafe {
                let a = _mm_set_epi64x(0, a as i64);
                let b = _mm_set_epi64x(0, b as i64);
                let x = _mm_clmulepi64_si128::<0>(a, b);
                let lo = _mm_extract_epi64::<0>(x) as u64;
                (lo as u32, (lo >> 32) as u32)
            }
        } else if #[cfg(all(
            not(feature="no-xmul"),
            target_arch="aarch64",
            target_feature="neon"
        ))] {
            // aarch64 provides 64-bit xmul via the pmull instruction
            use core::arch::aarch64::*;
            unsafe {
                let x = vmull_p64(a as u64, b as u64);
                (x as u32, (x >> 32) as u32)
            }
        }
    }
}

/// Widening carry-less multiplication, if hardware instructions are available
///
/// Result is a tuple (lo, hi)
///
#[cfg(any(
    all(
        not(feature="no-xmul"),
        target_arch="x86_64",
        target_feature="pclmulqdq"
    ),
    all(
        not(feature="no-xmul"),
        target_arch="aarch64",
        target_feature="neon"
    )
))]
#[inline]
pub fn xmul64(a: u64, b: u64) -> (u64, u64) {
    cfg_if! {
        if #[cfg(all(
            not(feature="no-xmul"),
            target_arch="x86_64",
            target_feature="pclmulqdq"
        ))] {
            // x86_64 provides 64-bit xmul via the pclmulqdq instruction
            use core::arch::x86_64::*;
            unsafe {
                let a = _mm_set_epi64x(0, a as i64);
                let b = _mm_set_epi64x(0, b as i64);
                let x = _mm_clmulepi64_si128::<0>(a, b);
                let lo = _mm_extract_epi64::<0>(x) as u64;
                let hi = _mm_extract_epi64::<1>(x) as u64;
                (lo, hi)
            }
        } else if #[cfg(all(
            not(feature="no-xmul"),
            target_arch="aarch64",
            target_feature="neon"
        ))] {
            // aarch64 provides 64-bit xmul via the pmull instruction
            use core::arch::aarch64::*;
            unsafe {
                let x = vmull_p64(a as u64, b as u64);
                (x as u64, (x >> 64) as u64)
            }
        }
    }
}

/// Widening carry-less multiplication, if hardware instructions are available
///
/// Result is a tuple (lo, hi)
///
#[cfg(any(
    all(
        not(feature="no-xmul"),
        target_arch="x86_64",
        target_feature="pclmulqdq"
    ),
    all(
        not(feature="no-xmul"),
        target_arch="aarch64",
        target_feature="neon"
    )
))]
#[inline]
pub fn xmul128(a: u128, b: u128) -> (u128, u128) {
    cfg_if! {
        if #[cfg(all(
            not(feature="no-xmul"),
            target_arch="x86_64",
            target_feature="pclmulqdq"
        ))] {
            // x86_64 provides 64-bit xmul via the pclmulqdq instruction
            use core::arch::x86_64::*;
            unsafe {
                let a = _mm_set_epi64x((a >> 64) as i64, a as i64);
                let b = _mm_set_epi64x((b >> 64) as i64, b as i64);
                let x = _mm_clmulepi64_si128::<0x00>(a, b);
                let y = _mm_clmulepi64_si128::<0x01>(a, b);
                let z = _mm_clmulepi64_si128::<0x10>(a, b);
                let w = _mm_clmulepi64_si128::<0x11>(a, b);
                let lolo = _mm_extract_epi64::<0>(x) as u64;
                let lohi = (_mm_extract_epi64::<1>(x) as u64)
                    ^ (_mm_extract_epi64::<0>(y) as u64)
                    ^ (_mm_extract_epi64::<0>(z) as u64);
                let hilo = (_mm_extract_epi64::<0>(w) as u64)
                    ^ (_mm_extract_epi64::<1>(y) as u64)
                    ^ (_mm_extract_epi64::<1>(z) as u64);
                let hihi = _mm_extract_epi64::<1>(w) as u64;
                let lo = ((lohi as u128) << 64) | (lolo as u128);
                let hi = ((hihi as u128) << 64) | (hilo as u128);
                (lo, hi)
            }
        } else if #[cfg(all(
            not(feature="no-xmul"),
            target_arch="aarch64",
            target_feature="neon"
        ))] {
            // aarch64 provides 64-bit xmul via the pmull instruction
            use core::arch::aarch64::*;
            unsafe {
                let x = vmull_p64(a as u64, b as u64);
                let y = vmull_p64((a >> 64) as u64, (b >>  0) as u64);
                let z = vmull_p64((a >>  0) as u64, (b >> 64) as u64);
                let w = vmull_p64((a >> 64) as u64, (b >> 64) as u64);
                (x ^ (y << 64) ^ (z << 64), w ^ (y >> 64) ^ (z >> 64))
            }
        }
    }
}


#[cfg(test)]
mod test {
    #[allow(unused)]
    use super::*;

    #[cfg(any(
        all(
            not(feature="no-xmul"),
            target_arch="x86_64",
            target_feature="pclmulqdq"
        ),
        all(
            not(feature="no-xmul"),
            target_arch="aarch64",
            target_feature="neon"
        )
    ))]
    #[test]
    fn xmul() {
        assert_eq!(xmul8(0x12, 0x12), (0x04, 0x01));
        assert_eq!(xmul16(0x1234, 0x1234), (0x0510, 0x0104));
        assert_eq!(xmul32(0x12345678, 0x12345678), (0x11141540, 0x01040510));
        assert_eq!(xmul64(0x123456789abcdef1, 0x123456789abcdef1), (0x4144455051545501, 0x0104051011141540));
        assert_eq!(xmul128(0x123456789abcdef123456789abcdef12, 0x123456789abcdef123456789abcdef12), (0x04051011141540414445505154550104, 0x01040510111415404144455051545501));
    }
}
