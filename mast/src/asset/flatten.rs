use super::{Asset, FixedOutput, Output, Source, SourceWalker, Types};

/// An asset that has been flattened, created by [`Asset::flatten`]
#[derive(Debug, Clone, Copy)]
#[must_use]
pub struct Flatten<A> {
    asset: A,
}

impl<A> Flatten<A> {
    pub(crate) fn new(asset: A) -> Self {
        Self { asset }
    }
}

impl<'a, A> Types<'a> for Flatten<A>
where
    A: Asset,
    for<'b, 'c> Output<'b, A>: FixedOutput<Time = A::Time> + Types<'c, Source = Source<'c, A>>,
{
    type Output = <Output<'a, A> as FixedOutput>::FixedOutput;
    type Source = Source<'a, A>;
}

impl<A> Asset for Flatten<A>
where
    A: Asset,
    for<'b, 'c> Output<'b, A>: FixedOutput<Time = A::Time> + Types<'c, Source = Source<'c, A>>,
{
    fn generate(&mut self) -> Output<'_, Self> {
        self.asset.generate().generate()
    }

    type Time = A::Time;
    fn modified(&mut self) -> Self::Time {
        Ord::max(self.asset.modified(), self.asset.generate().modified())
    }

    fn sources<W: SourceWalker<Self>>(&mut self, walker: &mut W) -> Result<(), W::Error> {
        self.asset.sources(walker)?;
        self.asset.generate().sources(walker)?;
        Ok(())
    }
}
