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

mod map;
pub use map::Map;

mod map_source;
pub use map_source::MapSource;

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

    /// Map the resulting value of an asset with a function.
    ///
    /// When using this function,
    /// you will often encounter problems
    /// if you try to return a value that has a lifetime depending on the input's lifetime.
    /// For example, the following does not compile:
    ///
    /// ```compile_fail
    /// use ::mast::{self, Asset};
    ///
    /// let asset = mast::constant(5)
    ///     .map(|val: &mut u32| -> &mut u32 { val });
    /// # #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Time;
    /// # impl mast::Time for Time {
    /// #     fn earliest() -> Self { Self }
    /// # }
    /// # fn type_infer(_: impl Asset<Time = Time, Source = ()>) {}
    /// # type_infer(asset);
    /// ```
    ///
    /// To resolve this,
    /// you can use a helper funnelling function:
    ///
    /// ```
    /// use ::mast::{self, Asset};
    ///
    /// fn funnel<F: FnMut(&mut u32) -> &mut u32>(f: F) -> F { f }
    ///
    /// let asset = mast::constant(5)
    ///     .map(funnel(|val| val));
    /// # #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    /// # struct Time;
    /// # impl mast::Time for Time {
    /// #     fn earliest() -> Self { Self }
    /// # }
    /// # fn type_infer(_: impl Asset<Time = Time, Source = ()>) {}
    /// # type_infer(asset);
    /// ```
    ///
    /// This will force `rustc` to make type inference on your closure work differently,
    /// allowing it compile.
    /// The [`::higher-order-closure`] library
    /// can help to reduce the boilerplate of this pattern.
    ///
    /// No sanity or stability guarantees are provided if you override this function.
    ///
    /// [`::higher-order-closure`]: https://docs.rs/higher-order-closure
    fn map<F>(self, mapper: F) -> Map<Self, F>
    where
        Self: Sized,
        F: for<'a> map::Mapper<'a, Self>,
    {
        Map::new(self, mapper)
    }

    /// Map the [source](AssetLifetime::Source) type of this asset.
    ///
    /// This is useful when combining multiple asset kinds,
    /// each with different source types.
    fn map_source<F>(self, mapper: F) -> MapSource<Self, F>
    where
        Self: Sized,
        F: for<'a> map_source::SourceMapper<'a, Self>,
    {
        MapSource::new(self, mapper)
    }
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
