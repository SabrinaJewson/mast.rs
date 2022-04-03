use {
    super::{Asset, FixedOutput, Output},
    ::core::iter::FusedIterator,
};

/// An asset that can be generated once.
///
/// The difference between this trait and [`Asset`]
/// is that its `generate` function
/// takes `self` instead of `&mut self`.
///
/// This is implemented for
/// any type implementing [`FixedOutput`]
/// as well as [`TakeRef`].
#[must_use]
pub trait Once: Sized {
    /// The inner asset that can be generated multiple times.
    type Inner: Asset;

    /// Consume `self`, giving its inner asset.
    fn into_inner(self) -> Self::Inner;

    /// The output of the asset after generating it once.
    type OutputOnce;

    /// Generate the value of the asset once.
    fn generate_once(self) -> Self::OutputOnce;
}

impl<A: FixedOutput> Once for A {
    type Inner = Self;
    fn into_inner(self) -> Self::Inner {
        self
    }
    type OutputOnce = A::FixedOutput;
    fn generate_once(mut self) -> Self::OutputOnce {
        self.generate()
    }
}

/// A wrapper around an `&mut A` that implements [`Once`],
/// created by [`Asset::take_ref`].
#[derive(Debug)]
#[must_use]
pub struct TakeRef<'a, A: ?Sized> {
    inner: &'a mut A,
}

impl<'a, A: ?Sized> TakeRef<'a, A> {
    pub(crate) fn new(inner: &'a mut A) -> Self {
        Self { inner }
    }
}

impl<'a, A: ?Sized + Asset> Once for TakeRef<'a, A> {
    type Inner = &'a mut A;
    fn into_inner(self) -> Self::Inner {
        self.inner
    }

    type OutputOnce = Output<'a, A>;
    fn generate_once(self) -> Self::OutputOnce {
        self.inner.generate()
    }
}

/// An iterator that maps `&mut A`s to [`TakeRef<'_, A>`](TakeRef)s.
#[derive(Debug, Clone)]
#[must_use]
pub struct TakeRefs<I>(I);

impl<I> TakeRefs<I> {
    /// Map an iterator's contents with [`Asset::take_ref`].
    pub fn new(inner: I) -> Self {
        Self(inner)
    }
}

impl<'a, I, A> Iterator for TakeRefs<I>
where
    I: Iterator<Item = &'a mut A>,
    A: 'a + ?Sized + Asset,
{
    type Item = TakeRef<'a, A>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Asset::take_ref)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n).map(Asset::take_ref)
    }
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.0.fold(init, |acc, x| f(acc, x.take_ref()))
    }
}

impl<'a, I, A> DoubleEndedIterator for TakeRefs<I>
where
    I: DoubleEndedIterator<Item = &'a mut A>,
    A: 'a + ?Sized + Asset,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(Asset::take_ref)
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth_back(n).map(Asset::take_ref)
    }
    fn rfold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.0.rfold(init, |acc, x| f(acc, x.take_ref()))
    }
}

impl<'a, I, A> ExactSizeIterator for TakeRefs<I>
where
    I: ExactSizeIterator<Item = &'a mut A>,
    A: 'a + ?Sized + Asset,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, I, A> FusedIterator for TakeRefs<I>
where
    I: FusedIterator<Item = &'a mut A>,
    A: 'a + ?Sized + Asset,
{
}
