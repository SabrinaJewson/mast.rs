//! Utilities for combining statically-sized potentially-heterogenous collections of assets
//! into a single asset.

use {
    super::{Asset, Output, Source, SourceWalker, Types},
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
/// let asset = asset::zip([asset::constant(1), asset::constant(2)])
///     .map(|[a, b]: [&mut u32; 2]| {
///         assert_eq!(*a, 1);
///         assert_eq!(*b, 2);
///     });
/// # type_infer(asset).generate();
///
/// // Zip a tuple
/// let asset = asset::zip((asset::constant(1), asset::constant("foo")))
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
/// # fn type_infer<T: Asset<Time = Time, Source = ()>>(val: T) -> T { val }
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

impl<'a, A: Asset, const N: usize> Types<'a> for Array<A, N> {
    type Output = [Output<'a, A>; N];
    type Source = Source<'a, A>;
}

impl<A: Asset, const N: usize> Asset for Array<A, N> {
    fn generate(&mut self) -> Output<'_, Self> {
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

    fn sources<W: SourceWalker<Self>>(&mut self, walker: &mut W) -> Result<(), W::Error> {
        for asset in &mut self.0 {
            asset.sources(walker)?;
        }
        Ok(())
    }
}

macro_rules! impl_for_tuples {
    ($($ident:ident)*) => { impl_for_tuples!(($($ident)*) ($($ident)*)); };
    (($_:ident) ($__:ident)) => {};
    (($($ident:ident)*) ($first:ident $($rest:ident)*)) => {
        #[allow(non_snake_case)]
        const _: () = {
            #[derive(Debug, Clone, Copy)]
            pub struct Tuple<$($ident,)*>($($ident,)*);

            impl<'a, S, $($ident,)*> Types<'a> for Tuple<$($ident,)*>
            where
                $($ident: Types<'a, Source = S>,)*
            {
                type Output = ($(Output<'a, $ident>,)*);
                type Source = S;
            }

            impl<T, $($ident,)*> Asset for Tuple<$($ident,)*>
            where
                T: Time,
                $($rest: for<'a> Types<'a, Source = Source<'a, $first>>,)*
                Self: for<'a> Types<'a, Output = ($(Output<'a, $ident>,)*), Source = Source<'a, $first>>,
                $($ident: Asset<Time = T>,)*
            {
                fn generate(&mut self) -> Output<'_, Self> {
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

                fn sources<W: SourceWalker<Self>>(&mut self, walker: &mut W) -> Result<(), W::Error> {
                    let Self($($ident,)*) = self;
                    $($ident.sources(walker)?;)*
                    Ok(())
                }
            }

            impl<$($ident,)*> Zip for ($($ident,)*)
            where
                Tuple<$($ident,)*>: Asset,
            {
                type Zip = Tuple<$($ident,)*>;
                fn zip(self) -> Self::Zip {
                    let ($($ident,)*) = self;
                    Tuple($($ident,)*)
                }
            }
        };
        impl_for_tuples!(($($rest)*) ($($rest)*));
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
