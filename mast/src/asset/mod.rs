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
        #[must_use]
        pub const fn constant(value: T) -> Self {
            Delta::Same.track(value)
        }
        #[must_use]
        pub const fn is_same(&self) -> bool {
            matches!(self.delta, Delta::Same)
        }
        #[must_use]
        pub const fn is_modified(&self) -> bool {
            matches!(self.delta, Delta::Modified)
        }
        #[must_use]
        pub const fn as_ref(&self) -> Tracked<&T> {
            Tracked {
                value: &self.value,
                delta: self.delta,
            }
        }
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
    #[must_use]
    pub fn cmp<T: PartialEq<U>, U>(prev: &T, current: &U) -> Self {
        if prev == current {
            Self::Same
        } else {
            Self::Modified
        }
    }

    #[must_use]
    pub const fn or(self, other: Self) -> Self {
        match self {
            Self::Same => other,
            Self::Modified => Self::Modified,
        }
    }

    #[must_use]
    pub fn or_else<F: FnOnce() -> Delta>(self, f: F) -> Self {
        match self {
            Self::Same => f(),
            Self::Modified => Self::Modified,
        }
    }

    #[must_use]
    pub const fn track<T>(self, value: T) -> Tracked<T> {
        Tracked { value, delta: self }
    }
}

/// Trait for the subset of [`Asset`]s whose regeneration logic just
/// calculates the new etag and compares it with the old one.
/// Used to implement [`Asset::hash_etag`].
///
/// Examples of assets that do _not_ implement this trait
/// include assets representing content
/// downloaded from the Internet,
/// because obtaining the etag is done at the same time as the generation step.
pub trait EtagCmp: Asset {
    /// Calculate the current etag.
    fn etag(&self) -> Self::Etag;
}

use crate::etag::Etag;
