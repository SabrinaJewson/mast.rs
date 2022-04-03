use super::{funnel_source_walker, Asset, Output, Shared, Source, SourceWalker, Types};

/// An asset whose [source](Types::Source) is mapped to another type by a closure,
/// created by [`Asset::map_source`].
#[derive(Debug, Clone, Copy)]
pub struct MapSource<A, F> {
    asset: A,
    mapper: F,
}

impl<A, F> MapSource<A, F> {
    pub(crate) fn new(asset: A, mapper: F) -> Self {
        Self { asset, mapper }
    }
}

impl<'a, A, F> Types<'a> for MapSource<A, F>
where
    A: Asset,
    F: for<'b> SourceMapperMut<'b, A>,
{
    type Output = Output<'a, A>;
    type Source = <F as SourceMapperMut<'a, A>>::Output;
}

impl<A, F> Asset for MapSource<A, F>
where
    A: Asset,
    F: for<'a> SourceMapperMut<'a, A>,
{
    fn generate(&mut self) -> Output<'_, Self> {
        self.asset.generate()
    }

    type Time = A::Time;
    fn modified(&mut self) -> Self::Time {
        self.asset.modified()
    }

    fn sources<W: SourceWalker<Self>>(&mut self, walker: &mut W) -> Result<(), W::Error> {
        self.asset.sources(&mut funnel_source_walker(|source| {
            walker(self.mapper.call_mut(source))
        }))
    }
}

impl<A, F> Shared for MapSource<A, F>
where
    A: Shared,
    F: for<'a> SourceMapperRef<'a, A>,
{
    fn ref_generate(&self) -> Output<'_, Self> {
        self.asset.ref_generate()
    }

    fn ref_modified(&self) -> Self::Time {
        self.asset.ref_modified()
    }

    fn ref_sources<W: SourceWalker<Self>>(&self, walker: &mut W) -> Result<(), W::Error> {
        self.asset.ref_sources(&mut funnel_source_walker(|source| {
            walker(self.mapper.call_ref(source))
        }))
    }
}

pub trait SourceMapperMut<'a, A: Asset, ImpliedBounds = &'a A>: Sized {
    type Output;
    fn call_mut(&mut self, source: Source<'a, A>) -> Self::Output;
}

impl<'a, A, F, O> SourceMapperMut<'a, A> for F
where
    A: Asset,
    F: FnMut(Source<'a, A>) -> O,
{
    type Output = O;
    fn call_mut(&mut self, source: Source<'a, A>) -> Self::Output {
        self(source)
    }
}

pub trait SourceMapperRef<'a, A: Asset, ImpliedBounds = &'a A>:
    Sized + SourceMapperMut<'a, A, ImpliedBounds>
{
    fn call_ref(&self, source: Source<'a, A>) -> Self::Output;
}

impl<'a, A, F, O> SourceMapperRef<'a, A> for F
where
    A: Asset,
    F: Fn(Source<'a, A>) -> O,
{
    fn call_ref(&self, source: Source<'a, A>) -> Self::Output {
        self(source)
    }
}
