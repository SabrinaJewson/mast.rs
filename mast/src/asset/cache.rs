use crate::asset::{self, Asset};

/// An asset whose output value is cached, created by [`Asset::cache`].
#[derive(Debug, Clone, Copy)]
#[must_use]
pub struct Cache<A: asset::FixedOutput> {
    asset: A,
    cached: Option<(A::Time, A::FixedOutput)>,
}

impl<A: asset::FixedOutput> Cache<A> {
    pub(crate) fn new(asset: A) -> Self {
        Self {
            asset,
            cached: None,
        }
    }
}

impl<'a, A: asset::FixedOutput> asset::Lifetime<'a> for Cache<A> {
    type Output = &'a mut A::FixedOutput;
    type Source = asset::Source<'a, A>;
}

impl<A: asset::FixedOutput> Asset for Cache<A> {
    fn generate(&mut self) -> asset::Output<'_, Self> {
        let inner_modified = self.asset.modified();
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
    fn modified(&mut self) -> Self::Time {
        self.asset.modified()
    }

    fn sources<W: asset::SourceWalker<Self>>(&mut self, walker: &mut W) -> Result<(), W::Error> {
        self.asset.sources(walker)
    }
}
