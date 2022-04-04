//! Traits for homogenous dynamically-sized collections of assets
//! that can be iterated over.
// Clippy is often not able to see that <Self::Iter as SequenceIter<'_>>::Type will be an iterator.
#![allow(clippy::iter_not_returning_iterator)]

#[cfg(feature = "alloc")]
use ::alloc::collections::vec_deque::{self, VecDeque};
use {
    crate::{
        asset::{self, Asset},
        bounds,
        time::Time,
    },
    ::core::slice,
};

/// Base associated types of a sequence,
/// before any lifetimes have been applied.
pub trait Base {
    /// The associated [`Time`](Asset::Time) type of every asset in the sequence.
    type Time: Time;
}

/// Associated types of a sequence with its second lifetime applied.
///
/// This second lifetime is the lifetime given to the asset in the sequence
/// if `.into_inner()` is called on the [`asset::Once`].
///
/// Note that the types in this trait
/// are independent of the first lifetime given to the sequence
/// (the lifetime of the sequence itself).
pub trait Lifetime2<'b, ImplicitBounds: bounds::Sealed = bounds::Bounds<&'b Self>>: Base {
    /// The [`Source`](asset::Types::Source) associated type of each asset in the sequence.
    type Source;
}

/// Associated types of a sequence with its first lifetime applied.
///
/// This first lifetime is the lifetime of the sequence itself.
pub trait Lifetime1<'a, ImplicitBounds: bounds::Sealed = bounds::Bounds<&'a Self>>:
    for<'b> Lifetime2<'b>
{
    /// The inner asset type of each `Once` in the sequence.
    type Asset: Asset<Time = <Self as Base>::Time>
        + for<'b> asset::Types<'b, Source = <Self as Lifetime2<'b>>::Source>;

    /// The type of each item in the sequence.
    type Item: asset::Once<Inner = Self::Asset>;

    /// An iterator over the items in the sequence.
    type Iter: Iterator<Item = Self::Item>;
}

/// A homogenous collection of assets that can be iterated over.
pub trait Sequence: for<'a> Lifetime1<'a> {
    /// Iterate over the items in the sequence.
    fn iter(&mut self) -> <Self as Lifetime1<'_>>::Iter;
}

macro_rules! impl_for_mut_ref {
    ($($ref:tt)*) => {
        impl<S: ?Sized + Sequence> Base for $($ref)* {
            type Time = S::Time;
        }
        impl<'b, S: ?Sized + Sequence> Lifetime2<'b> for $($ref)* {
            type Source = <S as Lifetime2<'b>>::Source;
        }
        impl<'a, S: ?Sized + Sequence> Lifetime1<'a> for $($ref)* {
            type Asset = <Self::Item as asset::Once>::Inner;
            type Item = <Self::Iter as Iterator>::Item;
            type Iter = <S as Lifetime1<'a>>::Iter;
        }
        impl<S: ?Sized + Sequence> Sequence for $($ref)* {
            fn iter(&mut self) -> <S as Lifetime1<'_>>::Iter {
                (**self).iter()
            }
        }
    };
}

impl_for_mut_ref!(&mut S);

#[cfg(feature = "alloc")]
impl_for_mut_ref!(alloc::boxed::Box<S>);

macro_rules! impl_for_slicelike {
    ({$($generics:tt)*} $($ty:tt)*) => {
        impl<A: Asset, $($generics)*> Base for $($ty)* {
            type Time = A::Time;
        }
        impl<'b, A: Asset, $($generics)*> Lifetime2<'b> for $($ty)* {
            type Source = <A as asset::Types<'b>>::Source;
        }
        impl<'a, A: Asset, $($generics)*> Lifetime1<'a> for $($ty)* {
            type Asset = &'a mut A;
            type Item = asset::TakeRef<'a, A>;
            type Iter = asset::TakeRefs<slice::IterMut<'a, A>>;
        }
        impl<A: Asset, $($generics)*> Sequence for $($ty)* {
            fn iter(&mut self) -> <Self as Lifetime1<'_>>::Iter {
                asset::TakeRefs::new(self.iter_mut())
            }
        }
    };
}

impl_for_slicelike!({}[A]);
impl_for_slicelike!({ const N: usize } [A; N]);
#[cfg(feature = "alloc")]
impl_for_slicelike!({} alloc::vec::Vec<A>);

#[cfg(feature = "alloc")]
impl<A: Asset> Base for VecDeque<A> {
    type Time = A::Time;
}
#[cfg(feature = "alloc")]
impl<'b, A: Asset> Lifetime2<'b> for VecDeque<A> {
    type Source = <A as asset::Types<'b>>::Source;
}
#[cfg(feature = "alloc")]
impl<'a, A: Asset> Lifetime1<'a> for VecDeque<A> {
    type Asset = &'a mut A;
    type Item = asset::TakeRef<'a, A>;
    type Iter = asset::TakeRefs<vec_deque::IterMut<'a, A>>;
}
#[cfg(feature = "alloc")]
impl<A: Asset> Sequence for VecDeque<A> {
    fn iter(&mut self) -> <Self as Lifetime1<'_>>::Iter {
        asset::TakeRefs::new(self.iter_mut())
    }
}
