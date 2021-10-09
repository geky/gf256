//! Common traits
//!
//! Currently just a workaround for FromLossy/IntoLossy traits
//!

// TryFrom/TryInto forwarded for convenience
pub use std::convert::TryFrom;
pub use std::convert::TryInto;

/// A From trait for conversions which may lose precision
///
/// Note this is just a temporary workaround. Once [RFC2484] is implemented
/// these traits should go away.
///
/// [RFC2484]: https://github.com/rust-lang/rfcs/pull/2484
///
pub trait FromLossy<T> {
    fn from_lossy(t: T) -> Self;
}

/// An Into trait for conversions which may lose precision
///
/// This is similar to Into, but for FromLossy
///
pub trait IntoLossy<T> {
    fn into_lossy(self) -> T;
}

/// IntoLossy is the inverse of FromLossy
impl<T, U> IntoLossy<T> for U
where
    T: FromLossy<U>
{
    #[inline]
    fn into_lossy(self) -> T {
        T::from_lossy(self)
    }
}

/// All types that provide From provide FromLossy
impl<T, U> FromLossy<T> for U
where
    U: From<T>
{
    #[inline]
    fn from_lossy(t: T) -> Self {
        Self::from(t)
    }
}

