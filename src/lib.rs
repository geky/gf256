

/// Common traits
pub mod traits;

/// Macros for generating customized types
pub mod macros;

/// Polynomial types
pub mod p;
pub use p::*;

/// re-exported for proc_macros
pub mod internal {
    pub use cfg_if;
}
