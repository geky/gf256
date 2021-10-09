

/// Common traits
pub mod traits;

/// Polynomial types
pub mod p;
pub use p::*;

/// re-exported for proc_macros
pub mod internal {
    pub use cfg_if;
}
