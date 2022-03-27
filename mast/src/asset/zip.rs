//! Utilities for combining statically-sized potentially-heterogenous collections of assets
//! into a single asset.

use {
    super::{Asset, Output, Source, SourceWalker},
    crate::time::Time,
};

/// Combine a set of several assets into a single one
/// that depends on all of the inner assets' values.
///
/// This function works with arrays and tuples.
///
/// # Examples
///
/// ```
/// use ::mast::asset::{self, Asset};
///
/// // Zip an array
/// let asset = asset::zip([mast::constant(1), mast::constant(2)])
///     .map(|[a, b]: [&mut u32; 2]| {
///         assert_eq!(*a, 1);
///         assert_eq!(*b, 2);
///     });
/// # type_infer(asset).generate();
///
/// // Zip a tuple
/// let asset = asset::zip((mast::constant(1), mast::constant("foo")))
///     .map(|(a, b): (&mut u32, &mut &'static str)| {
///         assert_eq!(*a, 1);
///         assert_eq!(*b, "foo");
///     });
/// # type_infer(asset).generate();
/// #
/// # #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// # struct Time;
/// # impl mast::Time for Time {
/// #     fn earliest() -> Self { Self }
/// # }
/// # fn type_infer<T: Asset<Time = Time, Source = fn(&()) -> ()>>(val: T) -> T { val }
/// ```
pub fn zip<T: Zip>(zip: T) -> T::Zip {
    zip.zip()
}

/// A statically-sized potentially-heterogenous collection of assets
/// that can be combined into a single asset.
pub trait Zip {
    /// The asset produced by [`Self::zip`].
    type Zip: Asset;
    /// Combine all the assets in this collection.
    fn zip(self) -> Self::Zip;
}

impl<A: Asset, const N: usize> Zip for [A; N] {
    type Zip = Array<A, N>;

    fn zip(self) -> Self::Zip {
        Array(self)
    }
}

/// An array of assets combined into a single asset.
/// This is created by calling [`zip`] on an array.
///
/// This asset outputs an array of all the generated results.
#[derive(Debug, Clone, Copy)]
#[must_use]
pub struct Array<A, const N: usize>([A; N]);

impl<A: Asset, const N: usize> Asset for Array<A, N> {
    type Output = fn(&()) -> [<A::Output as Output<'_>>::Type; N];
    fn generate(&mut self) -> <Self::Output as Output<'_>>::Type {
        array_each_mut(&mut self.0).map(A::generate)
    }

    type Time = A::Time;
    fn last_modified(&mut self) -> Self::Time {
        self.0
            .iter_mut()
            .map(A::last_modified)
            .max()
            .unwrap_or_else(Time::earliest)
    }

    type Source = A::Source;
    fn sources(&mut self, walker: SourceWalker<'_, Self>) {
        for asset in &mut self.0 {
            asset.sources(walker);
        }
    }
}

macro_rules! impl_for_tuples {
    (@$_:ident) => {};
    (@$first:ident $($rest:ident)*) => {
        impl_for_tuples!($($rest)*);
    };
    ($($ident:ident)*) => {
        #[allow(non_snake_case)]
        const _: () = {
            #[derive(Debug, Clone, Copy)]
            pub struct Tuple<$($ident,)*>($($ident,)*);
            impl<T, S, $($ident: Asset<Time = T, Source = S>,)*> Asset for Tuple<$($ident,)*>
            where
                T: Time,
                S: for<'a> Source<'a>,
            {
                type Output = fn(&()) -> ($(<$ident::Output as Output<'_>>::Type,)*);
                fn generate(&mut self) -> <Self::Output as Output<'_>>::Type {
                    let Self($($ident,)*) = self;
                    ($($ident.generate(),)*)
                }

                type Time = T;
                fn last_modified(&mut self) -> Self::Time {
                    let Self($($ident,)*) = self;
                    let mut latest = T::earliest();
                    $(latest = Ord::max(latest, $ident.last_modified());)*
                    latest
                }

                type Source = S;
                fn sources(&mut self, walker: SourceWalker<'_, Self>) {
                    let Self($($ident,)*) = self;
                    $($ident.sources(walker);)*
                }
            }

            impl<T, S, $($ident: Asset<Time = T, Source = S>,)*> Zip for ($($ident,)*)
            where
                T: Time,
                S: for<'a> Source<'a>,
            {
                type Zip = Tuple<$($ident,)*>;
                fn zip(self) -> Self::Zip {
                    let ($($ident,)*) = self;
                    Tuple($($ident,)*)
                }
            }
        };
        impl_for_tuples!(@$($ident)*);
    };
}
impl_for_tuples!(A B C D E F G H I J K L);

fn array_each_mut<T, const N: usize>(values: &mut [T; N]) -> [&mut T; N] {
    use ::core::mem::MaybeUninit;

    struct Helper<T>(T);
    impl<T> Helper<T> {
        const UNINIT: MaybeUninit<T> = MaybeUninit::uninit();
    }
    let mut array = [<Helper<&mut T>>::UNINIT; N];
    for (i, reference) in values.iter_mut().enumerate() {
        array[i] = MaybeUninit::new(reference);
    }
    // SAFETY:
    // - These two types have the same layout
    // - MaybeUninit on the original array guarantees we won't double-drop
    // - We have just fully initialized the array
    unsafe { core::mem::transmute_copy::<[MaybeUninit<&mut T>; N], [&mut T; N]>(&array) }
}

#[test]
fn array_each_mut_works() {
    let mut array = [1, 2, 3];
    let references = array_each_mut(&mut array);
    assert_eq!(*references[0], 1);
    assert_eq!(*references[1], 2);
    assert_eq!(*references[2], 3);
}
