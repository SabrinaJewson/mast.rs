//! Utilties for combining dynamic-length homogenous collections of assets
//! into a single asset.
// Clippy is often not able to see that <Self::Iter as SequenceIter<'_>>::Type will be an iterator.
#![allow(clippy::iter_not_returning_iterator)]

use {
    super::{Asset, Output, SourceWalker, Types},
    crate::time::Time,
    ::core::iter::FusedIterator,
};

/// Combine a dynamic-length homogenous collection of assets
/// into a single asset
/// that yields an iterator over the inner assets' values.
///
/// # Examples
///
/// ```
/// use ::mast::asset::{self, Asset};
///
/// let asset = asset::zip_all(vec![asset::constant(0), asset::constant(1)])
///     .map(|iter: asset::zip_all::Outputs<core::slice::IterMut<'_, _>>| {
///         for (i, item @ &mut u32) in iter.enumerate() {
///             assert_eq!(*item, i);
///         }
///     });
/// # type_infer(asset).generate();
/// # fn type_infer<T: Asset<Time = std::time::SystemTime, Source = ()>>(v: T) -> T { v }
/// ```
pub fn zip_all<S: Sequence>(sequence: S) -> ZipAll<S> {
    ZipAll { sequence }
}

/// The asset returned by [`zip_all`].
#[derive(Debug, Clone)]
#[must_use]
pub struct ZipAll<S> {
    sequence: S,
}

impl<'a, S: Sequence> Types<'a> for ZipAll<S> {
    type Output = Outputs<<S::Iter as SequenceIter<'a>>::Type>;
    type Source = <S::Asset as Types<'a>>::Source;
}

impl<S: Sequence> Asset for ZipAll<S> {
    fn generate(&mut self) -> Output<'_, Self> {
        Outputs(self.sequence.iter())
    }

    type Time = <S::Asset as Asset>::Time;
    fn modified(&mut self) -> Self::Time {
        self.sequence
            .iter()
            .map(Asset::modified)
            .max()
            .unwrap_or_else(Time::earliest)
    }

    fn sources<W: SourceWalker<Self>>(&mut self, walker: &mut W) -> Result<(), W::Error> {
        for asset in self.sequence.iter() {
            asset.sources(walker)?;
        }
        Ok(())
    }
}

/// A homogenous sequence of items for use with [`zip_all`].
pub trait Sequence {
    /// The type of each asset in the sequence.
    type Asset: ?Sized + Asset;

    /// A pseudo-GAT representing an iterator over the sequence.
    type Iter: for<'a> SequenceIter<'a, Item = &'a mut Self::Asset>;

    /// Iterate over the assets in the sequence.
    fn iter(&mut self) -> <Self::Iter as SequenceIter<'_>>::Type;
}

/// The type constructor of a sequence's [iterator](Sequence::Iter),
/// represented as a function pointer of the form `fn(&()) -> I`
/// where `I` is the actual iterator type.
pub trait SequenceIter<'a>: sequence_iter::Sealed<'a> {
    /// The type of each asset in the collection.
    type Item: Asset;

    /// The iterator type that iterates over the collection.
    type Type: Iterator<Item = Self::Item>;
}

impl<'a, F: FnOnce(&'a ()) -> O, O> SequenceIter<'a> for F
where
    O: Iterator,
    O::Item: Asset,
{
    type Item = O::Item;
    type Type = O;
}

mod sequence_iter {
    use super::super::Asset;

    pub trait Sealed<'a> {}
    impl<'a, F: FnOnce(&'a ()) -> O, O> Sealed<'a> for F
    where
        O: Iterator,
        O::Item: Asset,
    {
    }
}

impl<S: ?Sized + Sequence> Sequence for &mut S {
    type Asset = S::Asset;
    type Iter = S::Iter;
    fn iter(&mut self) -> <Self::Iter as SequenceIter<'_>>::Type {
        (**self).iter()
    }
}

#[cfg(feature = "alloc")]
impl<S: ?Sized + Sequence> Sequence for alloc::boxed::Box<S> {
    type Asset = S::Asset;
    type Iter = S::Iter;
    fn iter(&mut self) -> <Self::Iter as SequenceIter<'_>>::Type {
        (**self).iter()
    }
}

impl<A: Asset> Sequence for [A] {
    type Asset = A;
    type Iter = fn(&()) -> core::slice::IterMut<'_, A>;
    fn iter(&mut self) -> <Self::Iter as SequenceIter<'_>>::Type {
        self.iter_mut()
    }
}

#[cfg(feature = "alloc")]
impl<A: Asset> Sequence for alloc::vec::Vec<A> {
    type Asset = A;
    type Iter = fn(&()) -> core::slice::IterMut<'_, A>;
    fn iter(&mut self) -> <Self::Iter as SequenceIter<'_>>::Type {
        self.iter_mut()
    }
}

#[cfg(feature = "alloc")]
impl<A: Asset> Sequence for alloc::collections::VecDeque<A> {
    type Asset = A;
    type Iter = fn(&()) -> alloc::collections::vec_deque::IterMut<'_, A>;
    fn iter(&mut self) -> <Self::Iter as SequenceIter<'_>>::Type {
        self.iter_mut()
    }
}

/// An iterator over the outputs of a zipped asset sequence.
/// This is the output type of [`ZipAll`].
#[derive(Debug, Clone)]
#[must_use]
pub struct Outputs<I>(I);

impl<'a, I, A> Iterator for Outputs<I>
where
    I: Iterator<Item = &'a mut A>,
    A: 'a + ?Sized + Asset,
{
    type Item = Output<'a, A>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Asset::generate)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, I, A> DoubleEndedIterator for Outputs<I>
where
    I: DoubleEndedIterator<Item = &'a mut A>,
    A: 'a + ?Sized + Asset,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(Asset::generate)
    }
}

impl<'a, I, A> ExactSizeIterator for Outputs<I>
where
    I: ExactSizeIterator<Item = &'a mut A>,
    A: 'a + ?Sized + Asset,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, I, A> FusedIterator for Outputs<I>
where
    I: FusedIterator<Item = &'a mut A>,
    A: 'a + ?Sized + Asset,
{
}
