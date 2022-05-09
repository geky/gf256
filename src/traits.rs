//! Common traits
//!
//! Currently just a workaround for FromLossy/IntoLossy traits
//!

// TryFrom/TryInto forwarded for convenience
pub use core::convert::TryFrom;
pub use core::convert::TryInto;

/// A From trait for conversions which may lose precision
///
/// Note this is just a temporary solution. Once [RFC2484] is implemented
/// these traits should go away.
///
/// [RFC2484]: https://github.com/rust-lang/rfcs/pull/2484
///
pub trait FromLossy<T> {
    /// Convert to this type lossily
    fn from_lossy(t: T) -> Self;
}

/// An Into trait for conversions which may lose precision
///
/// This is similar to Into, but for FromLossy
///
pub trait IntoLossy<T> {
    /// Convert this type lossily
    fn into_lossy(self) -> T;
}

// IntoLossy is the inverse of FromLossy
impl<T, U> IntoLossy<T> for U
where
    T: FromLossy<U>
{
    #[inline]
    fn into_lossy(self) -> T {
        T::from_lossy(self)
    }
}

// All types that provide From provide FromLossy
impl<T, U> FromLossy<T> for U
where
    U: From<T>
{
    #[inline]
    fn from_lossy(t: T) -> Self {
        Self::from(t)
    }
}

