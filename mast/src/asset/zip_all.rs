//! Utilties for combining dynamic-length homogenous collections of assets
//! into a single asset.

use {
    super::{
        sequence::{self, Sequence},
        Asset, Once, Output, SourceWalker, TakeRefs, Types,
    },
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
/// use ::{core::slice, mast::asset::{self, Asset}};
///
/// let asset = asset::zip_all(vec![asset::constant(0), asset::constant(1)])
///     .map(|iter: asset::zip_all::RefOutputs<slice::IterMut<'_, _>>| {
///         for (i, item @ &u32) in iter.enumerate() {
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
    type Output = Outputs<<S as sequence::Lifetime1<'a>>::Iter>;
    type Source = <S as sequence::Lifetime2<'a>>::Source;
}

impl<S: Sequence> Asset for ZipAll<S> {
    fn generate(&mut self) -> Output<'_, Self> {
        Outputs(self.sequence.iter())
    }

    type Time = S::Time;
    fn modified(&mut self) -> Self::Time {
        self.sequence
            .iter()
            .map(|asset| asset.into_inner().modified())
            .max()
            .unwrap_or_else(Time::earliest)
    }

    fn sources<W: SourceWalker<Self>>(&mut self, walker: &mut W) -> Result<(), W::Error> {
        for asset in self.sequence.iter() {
            asset.into_inner().sources(walker)?;
        }
        Ok(())
    }
}

/// An iterator over the outputs of a zipped asset sequence.
/// This is the output type of [`ZipAll`].
#[derive(Debug, Clone)]
#[must_use]
pub struct Outputs<I>(I);

impl<I> Iterator for Outputs<I>
where
    I: Iterator,
    I::Item: Once,
{
    type Item = <I::Item as Once>::OutputOnce;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Once::generate_once)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n).map(Once::generate_once)
    }
}

impl<I> DoubleEndedIterator for Outputs<I>
where
    I: DoubleEndedIterator,
    I::Item: Once,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(Once::generate_once)
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth_back(n).map(Once::generate_once)
    }
}

impl<I> ExactSizeIterator for Outputs<I>
where
    I: ExactSizeIterator,
    I::Item: Once,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<I> FusedIterator for Outputs<I>
where
    I: FusedIterator,
    I::Item: Once,
{
}

/// Type alias for [`Outputs`]`<`[`TakeRefs`]`<I>>`.
///
/// This is a common output type
/// when dealing with zipped owned sequences of assets.
pub type RefOutputs<I> = Outputs<TakeRefs<I>>;
