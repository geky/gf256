#![doc=include_str!("../README.md")]


// Enable stdsimd for pmull on aarch64
#![cfg_attr(
    all(not(feature="no-xmul"), feature="nightly", target_arch="aarch64"),
    feature(stdsimd)
)]

// We don't really need std
#![no_std]

// Other assertions
#![deny(missing_debug_implementations)]


/// Extra traits
pub mod traits;

/// Macros for generating customized types
pub mod macros;

/// Polynomial types
pub mod p;
pub use p::*;

/// Galois-field types
pub mod gf;
pub use gf::*;

/// LFSR structs
#[cfg(feature="lfsr")]
pub mod lfsr;

/// CRC functions
#[cfg(feature="crc")]
pub mod crc;

/// Shamir secret-sharing
#[cfg(feature="shamir")]
pub mod shamir;

/// RAID-parity structs
#[cfg(feature="raid")]
pub mod raid;

/// Reed-Solomon error-correction
#[cfg(feature="rs")]
pub mod rs;


/// Re-exports for proc_macros
#[path="."]
pub mod internal {
    pub mod xmul;
    pub use cfg_if;
    #[cfg(any(feature="lfsr", feature="shamir"))]
    pub use rand;
}

/// A flag indicating if hardware carry-less multiplication instructions
/// are available
pub use internal::xmul::HAS_XMUL;

