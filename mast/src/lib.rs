//! Mast is a flexible build and caching system configured by Rust code.
//!
//! # Non-features
//!
//! - Automatic cleaning
#![warn(
    noop_method_call,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    missing_docs,
    missing_debug_implementations,
    clippy::pedantic
)]
#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod asset;
pub use asset::Asset;

pub mod etag;
pub use etag::Etag;

mod tracked {
    /// A value as well whether it is the same or modified.
    #[derive(Debug, Clone, Copy)]
    pub struct Tracked<T> {
        /// The value itself.
        pub value: T,
        /// Whether the value is the same or modified.
        pub delta: Delta,
    }

    impl<T> Tracked<T> {
        /// Construct a value that is always the same.
        ///
        /// # Example
        ///
        /// ```
        /// # use mast::Delta;
        /// # use mast::Tracked;
        /// assert_eq!(Tracked::constant(37).delta, Delta::Same);
        /// ```
        #[must_use]
        pub const fn constant(value: T) -> Self {
            Delta::Same.track(value)
        }
        /// Check whether `self.delta == Delta::Same`.
        ///
        /// # Example
        ///
        /// ```
        /// # use mast::Tracked;
        /// assert!(Tracked::constant(37).is_same());
        /// ```
        #[must_use]
        pub const fn is_same(&self) -> bool {
            matches!(self.delta, Delta::Same)
        }
        /// Check whether `self.delta == Delta::Modified`.
        ///
        /// # Example
        ///
        /// ```
        /// # use mast::Tracked;
        /// assert!(!Tracked::constant(37).is_modified());
        /// ```
        #[must_use]
        pub const fn is_modified(&self) -> bool {
            matches!(self.delta, Delta::Modified)
        }
        /// Convert a `&Tracked<T>` to a `Tracked<&T>`.
        #[must_use]
        pub const fn as_ref(&self) -> Tracked<&T> {
            Tracked {
                value: &self.value,
                delta: self.delta,
            }
        }
        /// Convert a `&mut Tracked<T>` to a `Tracked<&mut T>`.
        #[must_use]
        pub fn as_mut(&mut self) -> Tracked<&mut T> {
            Tracked {
                value: &mut self.value,
                delta: self.delta,
            }
        }
    }

    use crate::Delta;
}
pub use tracked::Tracked;

mod delta {
    /// Whether a value is the same or different.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Delta {
        /// The value is definitely the same as before.
        Same,
        /// The value may have changed from before.
        Modified,
    }

    impl Delta {
        /// Compare a previous and current value and return the appropriate `Delta`.
        ///
        /// # Examples
        ///
        /// ```
        /// # use mast::Delta;
        /// assert_eq!(Delta::cmp(&37, &37), Delta::Same);
        /// assert_eq!(Delta::cmp(&17, &37), Delta::Modified);
        /// ```
        #[must_use]
        pub fn cmp<T: PartialEq<U>, U>(prev: &T, current: &U) -> Self {
            if prev == current {
                Self::Same
            } else {
                Self::Modified
            }
        }

        /// Combine two deltas such that the overall result is considered modified
        /// if either delta is modified.
        ///
        /// # Examples
        ///
        /// ```
        /// # use mast::Delta;
        /// assert_eq!(Delta::Same.or(Delta::Same), Delta::Same);
        /// assert_eq!(Delta::Same.or(Delta::Modified), Delta::Modified);
        /// assert_eq!(Delta::Modified.or(Delta::Same), Delta::Modified);
        /// assert_eq!(Delta::Modified.or(Delta::Modified), Delta::Modified);
        /// ```
        #[must_use]
        pub const fn or(self, other: Self) -> Self {
            match self {
                Self::Same => other,
                Self::Modified => Self::Modified,
            }
        }

        /// Lazily combine two deltas such that the overall result is considered modified
        /// if either delta is modified.
        ///
        /// # Examples
        ///
        /// ```
        /// # use mast::Delta;
        /// fn compute_delta() -> Delta { Delta::Modified }
        ///
        /// // `compute_delta` is not called
        /// assert_eq!(Delta::Modified.or_else(|| compute_delta()), Delta::Modified);
        ///
        /// // `compute_delta` *is* called
        /// assert_eq!(Delta::Same.or_else(|| compute_delta()), Delta::Modified);
        /// ```
        #[must_use]
        pub fn or_else<F: FnOnce() -> Delta>(self, f: F) -> Self {
            match self {
                Self::Same => f(),
                Self::Modified => Self::Modified,
            }
        }

        /// Construct a [`Tracked`] using this value as its delta.
        ///
        /// # Examples
        ///
        /// ```
        /// # use mast::Delta;
        /// # use mast::Tracked;
        /// fn number() -> Tracked<u32> {
        ///     Delta::Same.track(37)
        /// }
        /// assert!(number().is_same());
        /// ```
        #[must_use]
        pub const fn track<T>(self, value: T) -> Tracked<T> {
            Tracked { value, delta: self }
        }
    }

    use crate::Tracked;
}
pub use delta::Delta;

macro_rules! for_tuples {
    ($macro:ident) => {
        $macro!(Tuple0:                         );
        $macro!(Tuple1:             A           );
        $macro!(Tuple2:            A B          );
        $macro!(Tuple3:           A B C         );
        $macro!(Tuple4:          A B C D        );
        $macro!(Tuple5:         A B C D E       );
        $macro!(Tuple6:        A B C D E F      );
        $macro!(Tuple7:       A B C D E F G     );
        $macro!(Tuple8:      A B C D E F G H    );
        $macro!(Tuple9:     A B C D E F G H I   );
        $macro!(Tuple10:   A B C D E F G H I J  );
        $macro!(Tuple11:  A B C D E F G H I J K );
        $macro!(Tuple12: A B C D E F G H I J K L);
    };
}
use for_tuples;
