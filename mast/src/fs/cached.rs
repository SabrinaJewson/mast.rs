use {
    crate::asset::{self, Asset},
    ::std::{path::Path, time::SystemTime},
};

/// An asset whose result is cached on the filesystem,
/// created by [`Asset::fs_cached`].
///
/// This asset outputs a tuple of
/// the path of the cache
/// and the inner asset's output.
#[derive(Debug, Clone, Copy)]
#[must_use]
pub struct Cached<A, P> {
    asset: A,
    path: P,
}

impl<A, P> Cached<A, P> {
    pub(crate) fn new(asset: A, path: P) -> Self {
        Self { asset, path }
    }
}

impl<'a, A, P> asset::Lifetime<'a> for Cached<A, P>
where
    A: Asset<Time = SystemTime>,
    P: AsRef<Path>,
{
    type Output = (&'a Path, Option<asset::Output<'a, A>>);
    type Source = asset::Source<'a, A>;
}

impl<A, P> Asset for Cached<A, P>
where
    A: Asset<Time = SystemTime>,
    P: AsRef<Path>,
{
    fn generate(&mut self) -> asset::Output<'_, Self> {
        let output = (self.asset.modified() >= self.modified()).then(|| self.asset.generate());
        (self.path.as_ref(), output)
    }

    type Time = SystemTime;
    fn modified(&mut self) -> Self::Time {
        self.asset.modified()
    }

    fn sources<W: asset::SourceWalker<Self>>(&mut self, walker: &mut W) -> Result<(), W::Error> {
        self.asset.sources(walker)
    }
}
