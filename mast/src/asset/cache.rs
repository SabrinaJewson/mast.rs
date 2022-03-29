use super::{Asset, FixedOutput, Output, Source, SourceWalker, Types};

/// An asset whose output value is cached, created by [`Asset::cache`].
#[derive(Debug, Clone, Copy)]
#[must_use]
pub struct Cache<A: FixedOutput> {
    asset: A,
    cached: Option<(A::Time, A::FixedOutput)>,
}

impl<A: FixedOutput> Cache<A> {
    pub(crate) fn new(asset: A) -> Self {
        Self {
            asset,
            cached: None,
        }
    }
}

impl<'a, A: FixedOutput> Types<'a> for Cache<A> {
    type Output = &'a mut A::FixedOutput;
    type Source = Source<'a, A>;
}

impl<A: FixedOutput> Asset for Cache<A> {
    fn generate(&mut self) -> Output<'_, Self> {
        let inner_modified = self.asset.last_modified();
        if self
            .cached
            .as_ref()
            .map_or(true, |(modified, _)| inner_modified > *modified)
        {
            self.cached = Some((inner_modified, self.asset.generate()));
        }
        &mut self.cached.as_mut().unwrap().1
    }

    type Time = A::Time;
    fn last_modified(&mut self) -> Self::Time {
        self.asset.last_modified()
    }

    fn sources<W: SourceWalker<Self>>(&mut self, walker: &mut W) -> Result<(), W::Error> {
        self.asset.sources(walker)
    }
}
