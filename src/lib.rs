
// Enable stdsimd for pmull on aarch64
#![cfg_attr(
    all(target_arch="aarch64", feature="use-nightly-features"),
    feature(stdsimd)
)]


/// Common traits
pub mod traits;

/// Macros for generating customized types
pub mod macros;

/// Polynomial types
pub mod p;
pub use p::*;

/// Galois-field types
pub mod gf;
pub use gf::*;

/// re-exported for proc_macros
pub mod internal {
    pub use cfg_if;
}
