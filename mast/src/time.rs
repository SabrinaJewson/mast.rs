//! Generic time handling.

/// An abstract representation of a specific point in time.
///
/// Implementors should have a cheap [`Clone`] implementation.
pub trait Time: Ord + Clone + Sized {
    /// Obtain the earliest relevant time, used when no other time is applicable.
    /// This is generally the time of the creation of the executable.
    ///
    /// Calling this function should be cheap.
    fn earliest() -> Self;
}

/// Times that additionally support retrieving the current time.
pub trait Now: Time {
    /// Obtain the current time.
    fn now() -> Self;
}
