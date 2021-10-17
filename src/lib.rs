
// Enable stdsimd for pmull on aarch64
#![cfg_attr(
    all(feature="use-nightly-features", target_arch="aarch64"),
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
#[path="."]
pub mod internal {
    pub use cfg_if;
    pub mod xmul;
}
