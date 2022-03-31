use super::{funnel_source_walker, Asset, Output, Source, SourceWalker, Types};

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
    F: for<'b> SourceMapper<'b, A>,
{
    type Output = Output<'a, A>;
    type Source = <F as SourceMapper<'a, A>>::Output;
}

impl<A, F> Asset for MapSource<A, F>
where
    A: Asset,
    F: for<'a> SourceMapper<'a, A>,
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
            walker(self.mapper.call(source))
        }))
    }
}

pub trait SourceMapper<'a, A: Asset, ImpliedBounds = &'a A> {
    type Output;
    fn call(&mut self, source: Source<'a, A>) -> Self::Output;
}

impl<'a, A, F, O> SourceMapper<'a, A> for F
where
    A: Asset,
    F: FnMut(Source<'a, A>) -> O,
{
    type Output = O;
    fn call(&mut self, source: Source<'a, A>) -> Self::Output {
        self(source)
    }
}
