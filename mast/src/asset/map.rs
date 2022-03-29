use super::{Asset, Output, Source, SourceWalker, Types};

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

impl<'a, A, F> Types<'a> for Map<A, F>
where
    A: Asset,
    F: for<'b> Mapper<'b, A>,
{
    type Output = <F as Mapper<'a, A>>::Output;
    type Source = Source<'a, A>;
}

impl<A, F> Asset for Map<A, F>
where
    A: Asset,
    F: for<'a> Mapper<'a, A>,
{
    fn generate(&mut self) -> Output<'_, Self> {
        self.mapper.call(self.asset.generate())
    }

    type Time = A::Time;
    fn last_modified(&mut self) -> Self::Time {
        self.asset.last_modified()
    }

    fn sources<W: SourceWalker<Self>>(&mut self, walker: &mut W) -> Result<(), W::Error> {
        self.asset.sources(walker)
    }
}

pub trait Mapper<'a, A: Asset, ImplicitBounds = &'a A>: Sized {
    type Output;
    fn call(&mut self, output: Output<'a, A>) -> Self::Output;
}
impl<'a, A: Asset, F, O> Mapper<'a, A> for F
where
    F: FnMut(Output<'a, A>) -> O,
{
    type Output = O;
    fn call(&mut self, output: Output<'a, A>) -> Self::Output {
        self(output)
    }
}
