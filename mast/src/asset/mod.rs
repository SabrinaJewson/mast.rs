//! The [`Asset`] trait.

/// A step in a build process.
pub trait Asset: Sized {
    /// An asset’s etag, analagous to the `ETag` header found in HTTP,
    /// is a fingerprint of the `Output` and sideeffects —
    /// a short piece of data such that identical etags imply identical full data
    /// (but identical full data does not necessarily imply identical etags).
    type Etag: Etag;

    /// The result of the build process step.
    type Output;

    /// Check whether the etag is still accurate and generate the asset’s result.
    fn update<'cx, 'e>(
        self,
        cx: Context<'cx>,
        etag: &'e mut Self::Etag,
    ) -> Tracked<Self::Generator<'cx, 'e>>;

    /// Type used to generate the final result of an asset. Returned by [`Self::update`].
    type Generator<'cx, 'e>: Generator<Output = Self::Output>;
}

/// Helper trait for generating the final result of an [`Asset`].
/// Returned by [`Asset::update`].
///
/// This trait is implemented for all functions that do not take arguments.
pub trait Generator: Sized {
    /// The output of the asset; the same as [`Asset::Output`].
    type Output;

    /// Perform the work necessary to generate the output.
    fn generate(self) -> Self::Output;
}

impl<O, F: FnOnce() -> O> Generator for F {
    type Output = O;
    fn generate(self) -> Self::Output {
        self()
    }
}

pub mod context;
pub use context::Context;

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
        /// # use mast::asset::Delta;
        /// # use mast::asset::Tracked;
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
        /// # use mast::asset::Tracked;
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
        /// # use mast::asset::Tracked;
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

    use super::Delta;
}
pub use tracked::Tracked;

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
    /// # use mast::asset::Delta;
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
    /// # use mast::asset::Delta;
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
    /// # use mast::asset::Delta;
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
    /// # use mast::asset::Delta;
    /// # use mast::asset::Tracked;
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

use crate::etag::Etag;
