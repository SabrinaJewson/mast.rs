use super::{Asset, FixedOutput, Output};

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
