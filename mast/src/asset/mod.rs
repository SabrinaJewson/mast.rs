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

use crate::Etag;
use crate::Tracked;
