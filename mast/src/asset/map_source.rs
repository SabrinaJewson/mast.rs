use super::{Asset, AssetLifetime, SourceWalker};

/// An asset whose [source](AssetLifetime::source) is mapped to another type by a closure,
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

impl<'a, A, F> AssetLifetime<'a> for MapSource<A, F>
where
    A: Asset,
    F: for<'b> SourceMapper<'b, A>,
{
    type Output = <A as AssetLifetime<'a>>::Output;

    fn generate(&'a mut self) -> Self::Output {
        self.asset.generate()
    }

    type Source = <F as SourceMapper<'a, A>>::Output;
}

impl<A, F> Asset for MapSource<A, F>
where
    A: Asset,
    F: for<'a> SourceMapper<'a, A>,
{
    type Time = A::Time;

    fn last_modified(&mut self) -> Self::Time {
        self.asset.last_modified()
    }

    fn sources(&mut self, walker: &mut dyn SourceWalker<Self>) {
        self.asset
            .sources(&mut |source| walker((self.mapper)(source)));
    }
}

pub trait SourceMapper<'a, A: Asset + 'a>:
    FnMut(<A as AssetLifetime<'a>>::Source) -> <Self as SourceMapper<'a, A>>::Output + Sized
{
    type Output;
}
impl<'a, A: 'a, F, O> SourceMapper<'a, A> for F
where
    A: Asset,
    F: FnMut(<A as AssetLifetime<'a>>::Source) -> O,
{
    type Output = O;
}
