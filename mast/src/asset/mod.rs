//! The [`Asset`] trait, defining a mutable resource with a known modification time.
//!
//! # GAT workaround
//!
//! To work around the lack of Generic Associated Types in stable Rust,
//! `Asset` is split into two traits: [`AssetLifetime`] and [`Asset`].
//! `AssetLifetime` contains all the parts of [`Asset`]
//! relevant to a specific lifetime that an asset may have,
//! while [`Asset`] contains all the lifetime-independent associated items.
//!
//! Implementors must implement both traits,
//! but only [`Asset`] needs to be used in trait bounds.

use crate::time::Time;

/// The base trait of [`Asset`], tied to a specific lifetime.
///
/// This trait should be implemented for any lifetime `'a`.
#[allow(clippy::module_name_repetitions)]
// ImplicitBounds is cursed magic to get the non-GAT equivalent of `where Self: 'a`.
pub trait AssetLifetime<'a, ImplicitBounds: sealed::Sealed = sealed::Bounds<&'a Self>> {
    /// The value the asset gives once it is generated.
    type Output;

    /// Generate the asset's value.
    ///
    /// This may perform computationally expensive work.
    fn generate(&'a mut self) -> Self::Output;

    /// The lowest-level source of where the asset obtains its value from.
    ///
    /// An asset that sources from the filesystem will probably use an `&'a Path` here.
    type Source;
}

mod sealed {
    pub trait Sealed {}
    pub struct Bounds<T>(T);
    impl<T> Sealed for Bounds<T> {}
}

/// A mutable resource with a known modification time.
#[must_use]
pub trait Asset: for<'a> AssetLifetime<'a> {
    /// The type this asset uses to keep track of time.
    type Time: Time;

    /// Obtain the time at which this asset was last modified,
    /// or in other words
    /// the time at which [`AssetLifetime::generate`] started returning the value it does.
    ///
    /// This can be used to avoid calling `generate` again, since that may be expensive.
    fn last_modified(&mut self) -> Self::Time;

    /// Walk over each of the [source](AssetLifetime::Source)s of the asset.
    ///
    /// This can be useful to determine which files to watch when implementing a watch mode.
    fn sources(&mut self, walker: &mut dyn SourceWalker<Self>);
}

/// A helper "trait alias" for `for<'a> FnMut(<A as AssetLifetime<'a>>::Source)`.
///
/// `&mut dyn FnMut(Self::Source)` can't be used directly in the signature of [`Asset::sources`]
/// because rustc gives an error about outlives requirements not being met.
/// As a workaround we can define a helper trait and use that instead.
pub trait SourceWalker<A: ?Sized + Asset>: for<'a> FnMut(<A as AssetLifetime<'a>>::Source) {}

impl<F, A: ?Sized + Asset> SourceWalker<A> for F where
    F: ?Sized + for<'a> FnMut(<A as AssetLifetime<'a>>::Source)
{
}

macro_rules! impl_for_refs {
    ($($ty:ty),*) => { $(
        impl<'a, A: Asset + ?Sized> AssetLifetime<'a> for $ty {
            type Output = <A as AssetLifetime<'a>>::Output;
            fn generate(&'a mut self) -> Self::Output {
                (**self).generate()
            }

            type Source = <A as AssetLifetime<'a>>::Source;
        }

        impl<A: ?Sized + Asset> Asset for $ty {
            type Time = A::Time;

            fn last_modified(&mut self) -> Self::Time {
                (**self).last_modified()
            }

            fn sources(&mut self, walker: &mut dyn SourceWalker<Self>) {
                (**self).sources(&mut |source| walker(source));
            }
        }
    )* };
}

impl_for_refs!(&mut A);
#[cfg(feature = "alloc")]
impl_for_refs!(alloc::boxed::Box<A>);
