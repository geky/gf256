//! Hardware xmul implementations if available
//!
//! These are declared here in order to be able to leverage unstable
//! features on nightly (if the feature nightly-features is provided).
//! Most of gf256 is provided as proc_macros, and those can't use unstable
//! features unless the feature is enabled with #[feature!] at the crate
//! level.
//!
//! These functions may or may not exist depending on what target_features
//! are available, so they shouldn't be used directly.
//!


/// x86_64 provides 64-bit xmul via the pclmulqdq instruction
#[cfg(all(
    target_arch="x86_64",
    target_feature="pclmulqdq"
))]
#[inline]
pub fn __pclmulqdq(a: u64, b: u64) -> u128 {
    use core::arch::x86_64::*;
    unsafe {
        let a = _mm_set_epi64x(0, a as i64);
        let b = _mm_set_epi64x(0, b as i64);
        let x = _mm_clmulepi64_si128::<0>(a, b);
        let x0 = _mm_extract_epi64::<0>(x) as u64;
        let x1 = _mm_extract_epi64::<1>(x) as u64;
        ((x1 as u128) << 64) | (x0 as u128)
    }
}

/// x86_64 provides 64-bit xmul via the pclmulqdq instruction
#[cfg(all(
    target_arch="x86_64",
    target_feature="pclmulqdq"
))]
#[inline]
pub fn __pclmulqdq_u64(a: u64, b: u64) -> u64 {
    use core::arch::x86_64::*;
    unsafe {
        let a = _mm_set_epi64x(0, a as i64);
        let b = _mm_set_epi64x(0, b as i64);
        let x = _mm_clmulepi64_si128::<0>(a, b);
        _mm_extract_epi64::<0>(x) as u64
    }
}

/// x86_64 provides 64-bit xmul via the pclmulqdq instruction
#[cfg(all(
    target_arch="x86_64",
    target_feature="pclmulqdq"
))]
#[inline]
pub fn __pclmulqdq_u128(a: u128, b: u128) -> u128 {
    use core::arch::x86_64::*;
    unsafe {
        let a = _mm_set_epi64x((a >> 64) as i64, a as i64);
        let b = _mm_set_epi64x((b >> 64) as i64, b as i64);
        let x = _mm_clmulepi64_si128::<0x0>(a, b);
        let y = _mm_clmulepi64_si128::<0x1>(a, b);
        let z = _mm_clmulepi64_si128::<0x4>(a, b);
        let x0 = _mm_extract_epi64::<0>(x) as u64;
        let x1 = (_mm_extract_epi64::<1>(x) as u64)
            ^ (_mm_extract_epi64::<1>(y) as u64)
            ^ (_mm_extract_epi64::<1>(z) as u64);
        ((x1 as u128) << 64) | (x0 as u128)
    }
}

/// aarch64 provides 64-bit xmul via the pmull instruction
#[cfg(all(
    feature="use-nightly-features",
    target_arch="aarch64",
    target_feature="neon"
))]
#[inline]
pub fn __pmull(a: u64, b: u64) -> u128 {
    use core::arch::aarch64::*;
    unsafe {
        vmull_p64(a, b)
    }
}

/// aarch64 provides 64-bit xmul via the pmull instruction
#[cfg(all(
    feature="use-nightly-features",
    target_arch="aarch64",
    target_feature="neon"
))]
#[inline]
pub fn __pmull_u64(a: u64, b: u64) -> u64 {
    use core::arch::aarch64::*;
    unsafe {
        vmull_p64(a, b) as u64
    }
}

/// aarch64 provides 64-bit xmul via the pmull instruction
#[cfg(all(
    feature="use-nightly-features",
    target_arch="aarch64",
    target_feature="neon"
))]
#[inline]
pub fn __pmull_u128(a: u128, b: u128) -> u128 {
    use core::arch::aarch64::*;
    unsafe {
        let x = vmull_p64(a as u64, b as u64);
        let y = vmull_p64((a >> 64) as u64, (b >>  0) as u64) << 64;
        let z = vmull_p64((a >>  0) as u64, (b >> 64) as u64) << 64;
        x ^ y ^ z
    }
}
