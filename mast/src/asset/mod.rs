//! The [`Asset`] trait, defining a mutable resource with a known modification time.

use crate::time::Time;

mod map;
pub use map::Map;

mod map_source;
pub use map_source::MapSource;

/// A mutable resource with a known modification time.
#[must_use]
pub trait Asset {
    /// The value the asset gives once it is generated.
    ///
    /// As a workaround for the lack of GATs,
    /// this associated type must be a function pointer taking `&()` and giving the output type.
    /// The lifetime in the `&()` type is the same lifetime as the one given to `generate`.
    type Output: for<'a> Output<'a>;

    /// Generate the asset's value.
    ///
    /// This may perform computationally expensive work.
    fn generate(&mut self) -> <Self::Output as Output<'_>>::Type;

    /// The type this asset uses to keep track of time.
    type Time: Time;

    /// Obtain the time at which this asset was last modified,
    /// or in other words
    /// the time at which [`Asset::generate`] started returning the value it does.
    ///
    /// This can be used to avoid calling `generate` again, since that may be expensive.
    fn last_modified(&mut self) -> Self::Time;

    /// The lowest-level source of where the asset obtains its value from.
    ///
    /// As a workaround for the lack of GATs,
    /// this associated type must be a function pointer taking `&()` and giving the output type.
    /// The lifetime in the `&()` type is the same lifetime as the one given to `generate`.
    ///
    /// An asset that sources from the filesystem
    /// will probably use a `fn(&()) -> &Path` here.
    type Source: for<'a> Source<'a>;

    /// Walk over each of the [source](Asset::Source)s of the asset.
    ///
    /// This can be useful to determine which files to watch when implementing a watch mode.
    fn sources(&mut self, walker: SourceWalker<'_, Self>);

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
    /// # fn type_infer(_: impl Asset<Time = Time, Source = fn(&()) -> ()>) {}
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
    /// # fn type_infer(_: impl Asset<Time = Time, Source = fn(&()) -> ()>) {}
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

    /// Map the [source](Asset::Source) type of this asset.
    ///
    /// This is useful when combining multiple asset kinds,
    /// each with different source types.
    // TODO: Write docs about the funnel function here
    ///
    /// No sanity or stability guarantees are provided if you override this function.
    fn map_source<F>(self, mapper: F) -> MapSource<Self, F>
    where
        Self: Sized,
        F: map_source::SourceMapper<Self>,
    {
        MapSource::new(self, mapper)
    }
}

/// The type constructor of an asset's [output](Asset::Output),
/// represented as a function pointer of the form `fn(&()) -> O`
/// where `O` is the actual output type.
pub trait Output<'a>: output::Sealed<'a> {
    /// The output type of the asset.
    type Type;
}

impl<'a, F: FnOnce(&'a ()) -> O, O> Output<'a> for F {
    type Type = O;
}

mod output {
    pub trait Sealed<'a> {}
    impl<'a, F: FnOnce(&'a ()) -> O, O> Sealed<'a> for F {}
}

/// The type constructor of an asset's [source](Asset::Source),
/// represented as a function pointer of the form `fn(&()) -> S`
/// where `S` is the actual source type.
pub trait Source<'a>: source::Sealed<'a> {
    /// The source type of the asset.
    type Type;
}

impl<'a, F: FnOnce(&'a ()) -> O, O> Source<'a> for F {
    type Type = O;
}

mod source {
    pub trait Sealed<'a> {}
    impl<'a, F: FnOnce(&'a ()) -> O, O> Sealed<'a> for F {}
}

/// The callback passed into [`Asset::sources`].
///
/// This is effectively an `&mut dyn for<'a> FnMut(<A::Source as Source<'a>>::Type)`
/// but with some extra trickery applied to get rustc to accept it.
pub type SourceWalker<'source_walker, A> =
    &'source_walker mut dyn source_walker::SourceWalker<<A as Asset>::Source>;

mod source_walker {
    use super::Source;

    // Workaround for https://github.com/rust-lang/rust/issues/95331
    pub trait SourceWalker<S: for<'a> Source<'a>>: for<'a> FnMut(<S as Source<'a>>::Type) {}

    impl<F, S: for<'a> Source<'a>> SourceWalker<S> for F where
        F: ?Sized + for<'a> FnMut(<S as Source<'a>>::Type)
    {
    }
}

#[test]
fn is_object_safe() {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct Time;
    impl crate::Time for Time {
        fn earliest() -> Self {
            Self
        }
    }
    let mut asset = crate::constant(5).map(|x: &mut _| *x);
    let _: &mut dyn Asset<Output = fn(&()) -> u32, Time = Time, Source = fn(&()) -> ()> =
        &mut asset;
}

macro_rules! impl_for_refs {
    ($($ty:ty),*) => { $(
        impl<A: Asset + ?Sized> Asset for $ty {
            type Output = A::Output;
            fn generate(&mut self) -> <Self::Output as Output<'_>>::Type {
                (**self).generate()
            }

            type Time = A::Time;
            fn last_modified(&mut self) -> Self::Time {
                (**self).last_modified()
            }

            type Source = A::Source;
            fn sources(&mut self, walker: SourceWalker<'_, Self>) {
                (**self).sources(walker)
            }
        }
    )* };
}

impl_for_refs!(&mut A);
#[cfg(feature = "alloc")]
impl_for_refs!(alloc::boxed::Box<A>);
