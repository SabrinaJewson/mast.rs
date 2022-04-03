use super::{Asset, Once, Output, Shared, Source, SourceWalker, Types};

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
    for<'b> Output<'b, A>: Once,
    for<'b, 'c> <Output<'b, A> as Once>::Inner:
        Asset<Time = A::Time> + Types<'c, Source = Source<'c, A>>,
{
    type Output = <Output<'a, A> as Once>::OutputOnce;
    type Source = Source<'a, A>;
}

impl<A> Asset for Flatten<A>
where
    A: Asset,
    for<'b> Output<'b, A>: Once,
    for<'b, 'c> <Output<'b, A> as Once>::Inner:
        Asset<Time = A::Time> + Types<'c, Source = Source<'c, A>>,
{
    fn generate(&mut self) -> Output<'_, Self> {
        self.asset.generate().generate_once()
    }

    type Time = A::Time;
    fn modified(&mut self) -> Self::Time {
        Ord::max(
            self.asset.modified(),
            self.asset.generate().into_inner().modified(),
        )
    }

    fn sources<W: SourceWalker<Self>>(&mut self, walker: &mut W) -> Result<(), W::Error> {
        self.asset.sources(walker)?;
        self.asset.generate().into_inner().sources(walker)?;
        Ok(())
    }
}

impl<A> Shared for Flatten<A>
where
    A: Shared,
    for<'b> Output<'b, A>: Once,
    for<'b, 'c> <Output<'b, A> as Once>::Inner:
        Asset<Time = A::Time> + Types<'c, Source = Source<'c, A>>,
{
    fn ref_generate(&self) -> Output<'_, Self> {
        self.asset.ref_generate().generate_once()
    }

    fn ref_modified(&self) -> Self::Time {
        Ord::max(
            self.asset.ref_modified(),
            self.asset.ref_generate().into_inner().modified(),
        )
    }

    fn ref_sources<W: SourceWalker<Self>>(&self, walker: &mut W) -> Result<(), W::Error> {
        self.asset.ref_sources(walker)?;
        self.asset.ref_generate().into_inner().sources(walker)?;
        Ok(())
    }
}
