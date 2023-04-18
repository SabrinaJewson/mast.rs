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

    /// Type used to generate the final result of an asset. Returned by [`Self::update`].
    type Generator<'cx, 'e>: Generator<Output = Self::Output>;

    /// Check whether the etag is still accurate and generate the asset’s result.
    fn update<'cx, 'e>(
        self,
        cx: Context<'cx>,
        etag: &'e mut Self::Etag,
    ) -> Tracked<Self::Generator<'cx, 'e>>;

    /// Chain another asset after this one.
    ///
    /// The callback accepts a [`Tracked`]`<`[`Self::Generator`]`>`
    /// (i.e. the return value of [`Self::update`])
    /// and returns another asset.
    fn then<InnerEtag, InnerOutput, F>(self, f: F) -> Then<Self, F>
    where
        F: for<'cx, 'e> then::FnOnce1<Tracked<Self::Generator<'cx, 'e>>>,
        for<'cx, 'e> <F as then::FnOnce1<Tracked<Self::Generator<'cx, 'e>>>>::Output:
            Asset<Etag = InnerEtag, Output = InnerOutput>,
        InnerEtag: Etag,
    {
        ensure_asset(Then::new(self, f))
    }

    /// Like [`Self::then`], but has better type inference
    /// for the case when the function’s return type doesn’t borrow from its input type.
    fn then_fixed<A, F>(self, f: F) -> Then<Self, F>
    where
        F: for<'cx, 'e> FnOnce(Tracked<Self::Generator<'cx, 'e>>) -> A,
        A: Asset,
    {
        ensure_asset(self.then::<A::Etag, A::Output, F>(f))
    }
}

mod then;
pub use then::Then;

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

fn ensure_asset<T: Asset>(value: T) -> T {
    value
}

use crate::Etag;
use crate::Tracked;
