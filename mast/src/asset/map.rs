use crate::asset::{self, Asset};

/// An asset whose output is mapped to another type by a closure,
/// created by [`Asset::map`].
#[derive(Debug, Clone, Copy)]
pub struct Map<A, F> {
    asset: A,
    mapper: F,
}

impl<A, F> Map<A, F> {
    pub(crate) fn new(asset: A, mapper: F) -> Self {
        Self { asset, mapper }
    }
}

impl<'a, A, F> asset::Lifetime<'a> for Map<A, F>
where
    A: Asset,
    F: for<'b> MapperMut<'b, A>,
{
    type Output = <F as MapperMut<'a, A>>::Output;
    type Source = asset::Source<'a, A>;
}

impl<A, F> Asset for Map<A, F>
where
    A: Asset,
    F: for<'a> MapperMut<'a, A>,
{
    fn generate(&mut self) -> asset::Output<'_, Self> {
        self.mapper.call_mut(self.asset.generate())
    }

    type Time = A::Time;
    fn modified(&mut self) -> Self::Time {
        self.asset.modified()
    }

    fn sources<W: asset::SourceWalker<Self>>(&mut self, walker: &mut W) -> Result<(), W::Error> {
        self.asset.sources(walker)
    }
}

impl<A, F> asset::Shared for Map<A, F>
where
    A: asset::Shared,
    F: for<'a> MapperRef<'a, A>,
{
    fn generate_shared(&self) -> asset::Output<'_, Self> {
        self.mapper.call_ref(self.asset.generate_shared())
    }

    fn modified_shared(&self) -> Self::Time {
        self.asset.modified_shared()
    }

    fn sources_shared<W: asset::SourceWalker<Self>>(&self, walker: &mut W) -> Result<(), W::Error> {
        self.asset.sources_shared(walker)
    }
}

pub trait MapperMut<'a, A: Asset, ImplicitBounds = &'a A>: Sized {
    type Output;
    fn call_mut(&mut self, output: asset::Output<'a, A>) -> Self::Output;
}
impl<'a, A: Asset, F, O> MapperMut<'a, A> for F
where
    F: FnMut(asset::Output<'a, A>) -> O,
{
    type Output = O;
    fn call_mut(&mut self, output: asset::Output<'a, A>) -> Self::Output {
        self(output)
    }
}

pub trait MapperRef<'a, A: Asset, ImplicitBounds = &'a A>:
    Sized + MapperMut<'a, A, ImplicitBounds>
{
    fn call_ref(&self, output: asset::Output<'a, A>) -> Self::Output;
}
impl<'a, A: Asset, F, O> MapperRef<'a, A> for F
where
    F: Fn(asset::Output<'a, A>) -> O,
{
    fn call_ref(&self, output: asset::Output<'a, A>) -> Self::Output {
        self(output)
    }
}
