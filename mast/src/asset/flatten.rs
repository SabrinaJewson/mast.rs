use super::{Asset, Output, SourceWalker};

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

impl<A> Asset for Flatten<A>
where
    A: Asset,
    for<'a> <A::Output as Output<'a>>::Type: FreeOutput<Time = A::Time, Source = A::Source>,
{
    type Output = fn(&()) -> <<A::Output as Output<'_>>::Type as FreeOutput>::FreeOutput;
    fn generate(&mut self) -> <Self::Output as Output<'_>>::Type {
        self.asset.generate().generate()
    }

    type Time = A::Time;
    fn last_modified(&mut self) -> Self::Time {
        Ord::max(
            self.asset.last_modified(),
            self.asset.generate().last_modified(),
        )
    }

    type Source = A::Source;
    fn sources(&mut self, walker: SourceWalker<'_, Self>) {
        self.asset.sources(walker);
        self.asset.generate().sources(walker);
    }
}

/// An asset whose `Output` only contains free variables,
/// i.e. does not depend on the asset's lifetime.
pub trait FreeOutput: Asset<Output = fn(&()) -> <Self as FreeOutput>::FreeOutput> {
    type FreeOutput;
}
impl<A, O> FreeOutput for A
where
    A: ?Sized + Asset<Output = fn(&()) -> O>,
{
    type FreeOutput = O;
}
