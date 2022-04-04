//! The [`Asset`] trait, defining a mutable resource with a known modification time.

use {
    crate::{bounds, time::Time},
    ::macro_vis::macro_vis,
};

mod once;
pub use once::{Once, TakeRef, TakeRefs};

mod constant;
pub use constant::{constant, Constant};

mod immutable;
pub use immutable::Immutable;

mod map;
pub use map::Map;

mod map_source;
pub use map_source::MapSource;

mod flatten;
pub use flatten::Flatten;

mod cache;
pub use cache::Cache;

mod share_local;
pub use share_local::ShareLocal;

pub mod share_mutex;

pub mod zip;
#[doc(no_inline)]
pub use zip::zip;

pub mod zip_all;
#[doc(no_inline)]
pub use zip_all::zip_all;

pub mod sequence;
#[doc(no_inline)]
pub use sequence::Sequence;

mod erased;
pub use erased::{Erased, ErasedExt, ErasedTypes};

/// Helper to get the [output type](Lifetime::Output) of an [`Asset`] for a specific lifetime.
pub type Output<'a, A> = <A as Lifetime<'a>>::Output;

/// Helper to get the [source type](Lifetime::Source) of an [`Asset`] for a specific lifetime.
pub type Source<'a, A> = <A as Lifetime<'a>>::Source;

/// The set of types associated with an [`Asset`] when a specific lifetime is applied.
///
/// This is used to work around the lack of GATs.
// ImplicitBounds is cursed magic to get the equivalent of GATs' `where Self: 'a`
pub trait Lifetime<'a, ImplicitBounds: bounds::Sealed = bounds::Bounds<&'a Self>> {
    /// The value the asset gives once it is generated.
    type Output;

    /// The lowest-level source of where the asset obtains its value from.
    ///
    /// An asset that sources from the filesystem
    /// will probably use a `&Path` here.
    type Source;
}

/// A mutable resource with a known modification time.
///
/// All the methods in here take `&mut self`;
/// if you don't require that,
/// make sure to implement the less restrictive [`Shared`] as well.
#[must_use]
pub trait Asset: for<'a> Lifetime<'a> {
    /// Generate the asset's value.
    /// This may perform computationally expensive work.
    ///
    /// The value returned by this function should generally be immutable,
    /// since all mutation should happen between calls to `generate` instead.
    /// If this function returns a unique reference,
    /// it is probably only for buffer reuse reasons
    /// and it should be safe to read and write to the value however you like
    /// without worrying about it affecting the next call to `generate` in any way.
    fn generate(&mut self) -> Output<'_, Self>;

    /// The type this asset uses to keep track of time.
    type Time: Time;

    /// Obtain the time at which this asset was last modified,
    /// or in other words
    /// the time at which [`Asset::generate`] started returning the value it does.
    ///
    /// This can be used to avoid calling `generate` again, since that may be expensive.
    fn modified(&mut self) -> Self::Time;

    /// Walk over each of the [source](Lifetime::Source)s of the asset.
    ///
    /// This can be useful
    /// to determine which files to watch when implementing features like a watch mode.
    ///
    /// If you are having trouble calling this function due to unsatisfied trait bounds,
    /// try wrapping the closure in [`funnel_source_walker`]:
    ///
    /// ```ignore
    /// asset.sources(&mut funnel_source_walker(|source| handle(source)));
    /// ```
    ///
    /// # Errors
    ///
    /// This function can only error if the underlying [`SourceWalker`] wishes to exit early.
    fn sources<W: SourceWalker<Self>>(&mut self, walker: &mut W) -> Result<(), W::Error>;

    /// Map the resulting value of an asset with a function.
    ///
    /// When using this function,
    /// you will often encounter problems
    /// if you try to return a value that has a lifetime depending on the input's lifetime.
    /// For example, the following does not compile:
    ///
    /// ```compile_fail
    /// use ::mast::asset::{self, Asset};
    ///
    /// let asset = asset::constant(5)
    ///     .map(|val: &u32| -> &u32 { val });
    /// # type_infer(asset).generate();
    /// # fn type_infer<A: Asset<Time = std::time::SystemTime, Source = ()>>(a: A) -> A { a }
    /// ```
    ///
    /// To resolve this,
    /// you can use a helper funnelling function:
    ///
    /// ```
    /// use ::mast::asset::{self, Asset};
    ///
    /// fn funnel<F: FnMut(&u32) -> &u32>(f: F) -> F { f }
    ///
    /// let asset = asset::constant(5)
    ///     .map(funnel(|val| val));
    /// # type_infer(asset).generate();
    /// # fn type_infer<A: Asset<Time = std::time::SystemTime, Source = ()>>(a: A) -> A { a }
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
        F: for<'a> map::MapperMut<'a, Self>,
    {
        Map::new(self, mapper)
    }

    /// Map the [source](Lifetime::Source) type of this asset.
    ///
    /// This is useful when combining multiple asset kinds,
    /// each with different source types.
    // TODO: Write docs about the funnel function here
    ///
    /// No sanity or stability guarantees are provided if you override this function.
    fn map_source<F>(self, mapper: F) -> MapSource<Self, F>
    where
        Self: Sized,
        F: for<'a> map_source::SourceMapperMut<'a, Self>,
    {
        MapSource::new(self, mapper)
    }

    /// Flatten the layering of an asset that outputs another asset.
    ///
    /// This is useful when dealing with dynamically-generated assets,
    /// e.g. a file which contains the path of another file.
    ///
    /// No sanity or stability guarantees are provided if you override this function.
    fn flatten(self) -> Flatten<Self>
    where
        Self: Sized,
        for<'a> Output<'a, Self>: Asset + Once,
        for<'a, 'c> <Output<'a, Self> as Once>::Inner:
            Asset<Time = Self::Time> + Lifetime<'c, Source = Source<'c, Self>>,
    {
        Flatten::new(self)
    }

    /// Cache the result of this asset in memory based on its modification time.
    ///
    /// With this combinator,
    /// the inner asset won't be regenerated
    /// unless it is newer than the cached version.
    ///
    /// No sanity or stability guarantees are provided if you override this function.
    fn cache(self) -> Cache<Self>
    where
        Self: Sized + FixedOutput,
    {
        Cache::new(self)
    }

    /// Apply this on an asset that writes its result to a path on the filesystem
    /// to avoid regenerating the asset if the output path is newer than the asset's value.
    ///
    /// No sanity or stability guarantees are provided if you override this function.
    #[cfg(feature = "fs")]
    #[cfg_attr(doc_nightly, doc(cfg(feature = "fs")))]
    fn fs_cached<P>(self, path: P) -> crate::fs::Cached<Self, P>
    where
        Self: Sized + Asset<Time = std::time::SystemTime>,
        P: AsRef<std::path::Path>,
    {
        crate::fs::Cached::new(self, path)
    }

    /// Take a unique reference to this asset and implement [`Once`] for it.
    ///
    /// No sanity or stability guarantees are provided if you override this function.
    fn take_ref(&mut self) -> TakeRef<'_, Self> {
        TakeRef::new(self)
    }

    /// Implement [`Shared`] for an asset using single-threaded shared mutability.
    ///
    /// If you want the resulting asset to be `Sync`,
    /// see the [`share_mutex`] module.
    ///
    /// No sanity or stability guarantees are provided if you override this function.
    fn share_local(self) -> ShareLocal<Self>
    where
        Self: Sized + FixedOutput,
    {
        ShareLocal::new(self)
    }
}

/// The callback passed into [`Asset::sources`].
///
/// This is a trait alias for `FnMut(Source<'_, A>) -> Result<(), E>`
/// used to make implementing [`Asset`] more succinct.
pub trait SourceWalker<A: ?Sized + Asset>:
    FnMut(Source<'_, A>) -> Result<(), <Self as SourceWalker<A>>::Error>
{
    /// The error given when the `SourceWalker` wishes to exit early.
    type Error;
}

impl<F, E, A> SourceWalker<A> for F
where
    A: ?Sized + Asset,
    F: ?Sized + FnMut(Source<'_, A>) -> Result<(), E>,
{
    type Error = E;
}

/// An identity function that accepts and returns a [`SourceWalker`].
///
/// This is very useful when passing a [`SourceWalker`] into a function,
/// because it makes rustc better at its type inference.
#[must_use]
pub fn funnel_source_walker<A, W, E>(walker: W) -> W
where
    A: ?Sized + Asset,
    W: FnMut(Source<'_, A>) -> Result<(), E>,
{
    walker
}

macro_rules! impl_for_unique_refs {
    ($($ty:ty),*) => { $(
        impl<'a, A: ?Sized + Asset> Lifetime<'a> for $ty {
            type Output = Output<'a, A>;
            type Source = Source<'a, A>;
        }

        impl<A: ?Sized + Asset> Asset for $ty {
            fn generate(&mut self) -> Output<'_, Self> {
                (**self).generate()
            }

            type Time = A::Time;
            fn modified(&mut self) -> Self::Time {
                (**self).modified()
            }

            fn sources<W: SourceWalker<Self>>(&mut self, walker: &mut W) -> Result<(), W::Error> {
                (**self).sources(walker)
            }
        }

        impl<A: ?Sized + Shared> Shared for $ty {
            fn generate_shared(&self) -> Output<'_, Self> {
                (**self).generate_shared()
            }

            fn modified_shared(&self) -> Self::Time {
                (**self).modified_shared()
            }

            fn sources_shared<W: SourceWalker<Self>>(&self, walker: &mut W) -> Result<(), W::Error> {
                (**self).sources_shared(walker)
            }
        }
    )* };
}

impl_for_unique_refs!(&mut A);
#[cfg(feature = "alloc")]
impl_for_unique_refs!(alloc::boxed::Box<A>);

/// An [`Asset`] that doesn't require unique references.
///
/// Ideally,
/// we would provide a blanket implementation of `Asset` for all types implementing this trait.
/// But that unfortunately interacts badly with generics and coherence,
/// so you'll often have to implement the two traits separately.
/// Most of the code duplication can be avoided however with the [`forward_to_shared!`] macro.
#[must_use]
pub trait Shared: Asset {
    /// Like [`Asset::generate`], but takes a shared reference to `self` instead.
    fn generate_shared(&self) -> Output<'_, Self>;

    /// Like [`Asset::modified`], but takes a shared reference to `self` instead.
    fn modified_shared(&self) -> Self::Time;

    /// Like [`Asset::sources`], but takes a shared reference to `self` instead.
    #[allow(clippy::missing_errors_doc)] // Already documented at `Asset::sources`
    fn sources_shared<W: SourceWalker<Self>>(&self, walker: &mut W) -> Result<(), W::Error>;
}

/// Implement the methods of [`Asset`]
/// by forwarding to an existing implementation of [`Shared`].
///
/// You can invoke this in an `impl Asset` block
/// to have `generate`, `modified` and `sources`
/// implemented for you automatically.
/// Note that it doesn't define a `Time` associated type,
/// so you'll have to do that yourself.
///
/// # Examples
///
/// ```
/// use ::mast::{asset::{self, Asset}, Time as _};
///
/// struct MyAsset;
///
/// impl<'a> asset::Lifetime<'a> for MyAsset {
///     type Output = ();
///     type Source = &'a str;
/// }
///
/// impl Asset for MyAsset {
///     type Time = std::time::SystemTime;
///     asset::forward_to_shared!();
/// }
///
/// impl asset::Shared for MyAsset {
///     fn generate_shared(&self) -> asset::Output<'_, Self> {
///         ()
///     }
///     fn modified_shared(&self) -> Self::Time {
///         std::time::SystemTime::earliest()
///     }
///     fn sources_shared<W: asset::SourceWalker<Self>>(&self, walker: &mut W) -> Result<(), W::Error> {
///         walker("my asset")
///     }
/// }
/// ```
#[macro_vis(pub)]
macro_rules! forward_to_shared {
    () => {
        fn generate(&mut self) -> $crate::asset::Output<'_, Self> {
            <Self as $crate::asset::Shared>::generate_shared(self)
        }
        fn modified(&mut self) -> <Self as $crate::Asset>::Time {
            <Self as $crate::asset::Shared>::modified_shared(self)
        }
        fn sources<W: $crate::asset::SourceWalker<Self>>(
            &mut self,
            walker: &mut W,
        ) -> $crate::asset::__private::Result<(), <W as $crate::asset::SourceWalker<Self>>::Error> {
            <Self as $crate::asset::Shared>::sources_shared::<W>(self, walker)
        }
    };
}

// Not public API.
#[doc(hidden)]
pub mod __private {
    pub use Result;
}

impl<'a, 'b, A: ?Sized + Shared> Lifetime<'a> for &'b A {
    type Output = Output<'b, A>;
    type Source = Source<'a, A>;
}

impl<A: ?Sized + Shared> Asset for &A {
    type Time = A::Time;
    forward_to_shared!();
}

impl<A: ?Sized + Shared> Shared for &A {
    fn generate_shared(&self) -> Output<'_, Self> {
        (**self).generate_shared()
    }
    fn modified_shared(&self) -> Self::Time {
        (**self).modified_shared()
    }
    fn sources_shared<W: SourceWalker<Self>>(&self, walker: &mut W) -> Result<(), W::Error> {
        (**self).sources_shared(walker)
    }
}

#[cfg(feature = "alloc")]
macro_rules! impl_for_shared_refs {
    ($($ty:ty),*) => { $(
        impl<'a, A: ?Sized + Shared> Lifetime<'a> for $ty {
            type Output = Output<'a, A>;
            type Source = Source<'a, A>;
        }

        impl<A: ?Sized + Shared> Asset for $ty {
            type Time = A::Time;

            forward_to_shared!();
        }

        impl<A: ?Sized + Shared> Shared for $ty {
            fn generate_shared(&self) -> Output<'_, Self> {
                (**self).generate_shared()
            }
            fn modified_shared(&self) -> Self::Time {
                (**self).modified_shared()
            }
            fn sources_shared<W: SourceWalker<Self>>(
                &self,
                walker: &mut W,
            ) -> Result<(), W::Error> {
                (**self).sources_shared(walker)
            }
        }
    )* };
}

#[cfg(feature = "alloc")]
impl_for_shared_refs!(::alloc::rc::Rc<A>, ::alloc::sync::Arc<A>);

/// An asset whose `Output` does not depend on the lifetime of the asset passed to [`generate`].
///
/// This trait is automatically blanket-implemented
/// for any appropriate asset.
///
/// [`generate`]: Asset::generate
pub trait FixedOutput:
    Asset + for<'a> Lifetime<'a, Output = <Self as FixedOutput>::FixedOutput>
{
    /// The asset's output type, independent of any input lifetimes.
    type FixedOutput;
}
impl<A, O> FixedOutput for A
where
    A: ?Sized + Asset + for<'a> Lifetime<'a, Output = O>,
{
    type FixedOutput = O;
}
