//! Generic time handling.

#[cfg(feature = "fs")]
use ::{once_cell::sync::Lazy, std::time::SystemTime};

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

#[cfg(feature = "fs")]
static EXE_MODIFIED: Lazy<SystemTime> = Lazy::new(|| {
    std::env::current_exe()
        .and_then(std::fs::symlink_metadata)
        .and_then(|meta| meta.modified())
        .ok()
        .unwrap_or_else(SystemTime::now)
});

#[cfg(feature = "fs")]
#[cfg_attr(doc_nightly, doc(cfg(feature = "fs")))]
impl Time for SystemTime {
    fn earliest() -> Self {
        *EXE_MODIFIED
    }
}

#[cfg(feature = "fs")]
#[cfg_attr(doc_nightly, doc(cfg(feature = "fs")))]
impl Now for SystemTime {
    fn now() -> Self {
        Self::now()
    }
}
