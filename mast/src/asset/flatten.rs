use crate::asset::{self, Asset, Once as _};

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

impl<'a, A> asset::Lifetime<'a> for Flatten<A>
where
    A: Asset,
    for<'b> asset::Output<'b, A>: asset::Once,
    for<'b, 'c> <asset::Output<'b, A> as asset::Once>::Inner:
        Asset<Time = A::Time> + asset::Lifetime<'c, Source = asset::Source<'c, A>>,
{
    type Output = <asset::Output<'a, A> as asset::Once>::OutputOnce;
    type Source = asset::Source<'a, A>;
}

impl<A> Asset for Flatten<A>
where
    A: Asset,
    for<'b> asset::Output<'b, A>: asset::Once,
    for<'b, 'c> <asset::Output<'b, A> as asset::Once>::Inner:
        Asset<Time = A::Time> + asset::Lifetime<'c, Source = asset::Source<'c, A>>,
{
    fn generate(&mut self) -> asset::Output<'_, Self> {
        self.asset.generate().generate_once()
    }

    type Time = A::Time;
    fn modified(&mut self) -> Self::Time {
        Ord::max(
            self.asset.modified(),
            self.asset.generate().into_inner().modified(),
        )
    }

    fn sources<W: asset::SourceWalker<Self>>(&mut self, walker: &mut W) -> Result<(), W::Error> {
        self.asset.sources(walker)?;
        self.asset.generate().into_inner().sources(walker)?;
        Ok(())
    }
}

impl<A> asset::Shared for Flatten<A>
where
    A: asset::Shared,
    for<'b> asset::Output<'b, A>: asset::Once,
    for<'b, 'c> <asset::Output<'b, A> as asset::Once>::Inner:
        Asset<Time = A::Time> + asset::Lifetime<'c, Source = asset::Source<'c, A>>,
{
    fn generate_shared(&self) -> asset::Output<'_, Self> {
        self.asset.generate_shared().generate_once()
    }

    fn modified_shared(&self) -> Self::Time {
        Ord::max(
            self.asset.modified_shared(),
            self.asset.generate_shared().into_inner().modified(),
        )
    }

    fn sources_shared<W: asset::SourceWalker<Self>>(&self, walker: &mut W) -> Result<(), W::Error> {
        self.asset.sources_shared(walker)?;
        self.asset.generate_shared().into_inner().sources(walker)?;
        Ok(())
    }
}
