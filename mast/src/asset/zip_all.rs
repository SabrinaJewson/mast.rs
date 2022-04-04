//! Utilties for combining dynamic-length homogenous collections of assets
//! into a single asset.

use {
    crate::{
        asset::{self, Asset, Once as _},
        time::Time,
    },
    ::core::iter::FusedIterator,
};

/// Combine a dynamic-length homogenous collection of assets
/// into a single asset
/// that yields an iterator over the inner assets' values.
///
/// Because assets are required to only have one output type
/// whether they are shared or not,
/// [`ZipAll`] itself can never implement [`asset::Shared`].
/// If you wish to share it use [`shared`] instead.
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
pub fn zip_all<S: asset::Sequence>(sequence: S) -> ZipAll<S> {
    ZipAll { sequence }
}

/// The asset returned by [`zip_all`].
#[derive(Debug, Clone)]
#[must_use]
pub struct ZipAll<S> {
    sequence: S,
}

impl<'a, S: asset::Sequence> asset::Lifetime<'a> for ZipAll<S> {
    type Output = Outputs<<S as asset::sequence::Lifetime1<'a>>::Iter>;
    type Source = <S as asset::sequence::Lifetime2<'a>>::Source;
}

impl<S: asset::Sequence> Asset for ZipAll<S> {
    fn generate(&mut self) -> asset::Output<'_, Self> {
        Outputs(self.sequence.iter())
    }

    type Time = S::Time;
    fn modified(&mut self) -> Self::Time {
        modified(self.sequence.iter())
    }

    fn sources<W: asset::SourceWalker<Self>>(&mut self, walker: &mut W) -> Result<(), W::Error> {
        for asset in self.sequence.iter() {
            asset.into_inner().sources(walker)?;
        }
        Ok(())
    }
}

/// A variant of [`zip_all`]
/// that produces an asset that implements [`Shared`].
pub fn shared<S: asset::sequence::Shared>(sequence: S) -> Shared<S> {
    Shared { sequence }
}

/// The asset returned by [`shared`].
#[derive(Debug, Clone)]
#[must_use]
pub struct Shared<S> {
    sequence: S,
}

impl<'a, S: asset::sequence::Shared> asset::Lifetime<'a> for Shared<S> {
    type Output = Outputs<<S as asset::sequence::SharedLifetime1<'a>>::IterShared>;
    type Source = <S as asset::sequence::Lifetime2<'a>>::Source;
}

impl<S: asset::sequence::Shared> Asset for Shared<S> {
    type Time = S::Time;

    asset::forward_to_shared!();
}

impl<S: asset::sequence::Shared> asset::Shared for Shared<S> {
    fn generate_shared(&self) -> asset::Output<'_, Self> {
        Outputs(self.sequence.iter_shared())
    }

    fn modified_shared(&self) -> Self::Time {
        modified(self.sequence.iter_shared())
    }

    fn sources_shared<W: asset::SourceWalker<Self>>(&self, walker: &mut W) -> Result<(), W::Error> {
        for asset in self.sequence.iter_shared() {
            asset.into_inner().sources(walker)?;
        }
        Ok(())
    }
}

fn modified<A: asset::Once>(iter: impl Iterator<Item = A>) -> <A::Inner as Asset>::Time {
    iter.map(|asset| asset.into_inner().modified())
        .max()
        .unwrap_or_else(Time::earliest)
}

/// An iterator over the outputs of a zipped asset sequence.
/// This is the output type of [`ZipAll`] and [`Shared`].
#[derive(Debug, Clone)]
#[must_use]
pub struct Outputs<I>(I);

impl<I> Iterator for Outputs<I>
where
    I: Iterator,
    I::Item: asset::Once,
{
    type Item = <I::Item as asset::Once>::OutputOnce;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(asset::Once::generate_once)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n).map(asset::Once::generate_once)
    }
}

impl<I> DoubleEndedIterator for Outputs<I>
where
    I: DoubleEndedIterator,
    I::Item: asset::Once,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(asset::Once::generate_once)
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth_back(n).map(asset::Once::generate_once)
    }
}

impl<I> ExactSizeIterator for Outputs<I>
where
    I: ExactSizeIterator,
    I::Item: asset::Once,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<I> FusedIterator for Outputs<I>
where
    I: FusedIterator,
    I::Item: asset::Once,
{
}

/// Type alias for [`Outputs`]`<`[`TakeRefs`](asset::TakeRefs)`<I>>`.
///
/// This is a common output type
/// when dealing with zipped owned sequences of assets.
pub type RefOutputs<I> = Outputs<asset::TakeRefs<I>>;
