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
    /// The [`Source`](asset::Lifetime::Source) associated type of each asset in the sequence.
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
        + for<'b> asset::Lifetime<'b, Source = <Self as Lifetime2<'b>>::Source>;

    /// The type of each item in the sequence.
    type Item: asset::Once<Inner = Self::Asset>;

    /// The iterator over the items in the sequence
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

        impl<'a, S: ?Sized + Shared> SharedLifetime1<'a> for $($ref)* {
            type AssetShared = <Self::ItemShared as asset::Once>::Inner;
            type ItemShared = <Self::IterShared as Iterator>::Item;
            type IterShared = <S as SharedLifetime1<'a>>::IterShared;
        }
        impl<S: ?Sized + Shared> Shared for $($ref)* {
            fn iter_shared(&self) -> <S as SharedLifetime1<'_>>::IterShared {
                (**self).iter_shared()
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
            type Source = <A as asset::Lifetime<'b>>::Source;
        }
        impl<'a, A: Asset, $($generics)*> Lifetime1<'a> for $($ty)* {
            type Asset = &'a mut A;
            type Item = asset::TakeRef<'a, A>;
            type Iter = asset::TakeRefs<slice::IterMut<'a, A>>;
        }
        impl<A: Asset, $($generics)*> Sequence for $($ty)* {
            fn iter(&mut self) -> <Self as Lifetime1<'_>>::Iter {
                asset::TakeRefs::new(<[_]>::iter_mut(self))
            }
        }

        impl<'a, A: asset::Shared, $($generics)*> SharedLifetime1<'a> for $($ty)* {
            type AssetShared = &'a A;
            type ItemShared = &'a A;
            type IterShared = slice::Iter<'a, A>;
        }
        impl<A: asset::Shared, $($generics)*> Shared for $($ty)* {
            fn iter_shared(&self) -> <Self as SharedLifetime1<'_>>::IterShared {
                <[_]>::iter(self)
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
    type Source = <A as asset::Lifetime<'b>>::Source;
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
        asset::TakeRefs::new(VecDeque::iter_mut(self))
    }
}

#[cfg(feature = "alloc")]
impl<'a, A: asset::Shared> SharedLifetime1<'a> for VecDeque<A> {
    type AssetShared = &'a A;
    type ItemShared = &'a A;
    type IterShared = vec_deque::Iter<'a, A>;
}
#[cfg(feature = "alloc")]
impl<A: asset::Shared> Shared for VecDeque<A> {
    fn iter_shared(&self) -> <Self as SharedLifetime1<'_>>::IterShared {
        VecDeque::iter(self)
    }
}

/// Associated types of a [`Shared`] sequence with its first lifetime applied.
///
/// This first lifetime is the lifetime of the sequence itself.
pub trait SharedLifetime1<'a, ImplicitBounds: bounds::Sealed = bounds::Bounds<&'a Self>>:
    Sequence
{
    /// The inner asset of each `Once` in the sequence
    /// when iterated with a shared reference.
    type AssetShared: Asset<Time = <Self as Base>::Time>
        + for<'b> asset::Lifetime<'b, Source = <Self as Lifetime2<'b>>::Source>;

    /// The type of each item in the sequence
    /// when iterated with a shared reference.
    type ItemShared: asset::Once<Inner = Self::AssetShared>;

    /// The iterator over the items in the sequence
    /// when iterated with a shared reference.
    type IterShared: Iterator<Item = Self::ItemShared>;
}

/// A [`Sequence`] additionally supporting iteration with a shared reference.
///
/// Ideally,
/// we would provide a blanket implementation of `Sequence` for all types implementing this trait.
/// But that unfortunately interacts badly with generics and coherence,
/// so you'll often have to implement the two traits separately.
pub trait Shared: for<'a> SharedLifetime1<'a> {
    /// Iterate over the items in the sequence
    /// from a shared reference.
    fn iter_shared(&self) -> <Self as SharedLifetime1<'_>>::IterShared;
}

impl<S: ?Sized + Shared> Base for &S {
    type Time = S::Time;
}
impl<'b, S: ?Sized + Shared> Lifetime2<'b> for &S {
    type Source = <S as Lifetime2<'b>>::Source;
}
impl<'a, S: ?Sized + Shared> Lifetime1<'a> for &S {
    type Asset = <Self::Item as asset::Once>::Inner;
    type Item = <Self::Iter as Iterator>::Item;
    type Iter = <S as SharedLifetime1<'a>>::IterShared;
}
impl<S: ?Sized + Shared> Sequence for &S {
    fn iter(&mut self) -> <Self as Lifetime1<'_>>::Iter {
        (**self).iter_shared()
    }
}
impl<'a, S: ?Sized + Shared> SharedLifetime1<'a> for &S {
    type AssetShared = <Self::ItemShared as asset::Once>::Inner;
    type ItemShared = <Self::IterShared as Iterator>::Item;
    type IterShared = <Self as Lifetime1<'a>>::Iter;
}
impl<S: ?Sized + Shared> Shared for &S {
    fn iter_shared(&self) -> <Self as SharedLifetime1<'_>>::IterShared {
        (**self).iter_shared()
    }
}
