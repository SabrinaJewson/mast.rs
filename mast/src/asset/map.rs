use super::{Asset, AssetLifetime, SourceWalker};

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

impl<'a, A, F> AssetLifetime<'a> for Map<A, F>
where
    A: Asset,
    F: for<'b> Mapper<'b, A>,
{
    type Output = <F as Mapper<'a, A>>::Output;

    fn generate(&'a mut self) -> Self::Output {
        (self.mapper)(self.asset.generate())
    }

    type Source = <A as AssetLifetime<'a>>::Source;
}

impl<A, F> Asset for Map<A, F>
where
    A: Asset,
    F: for<'a> Mapper<'a, A>,
{
    type Time = A::Time;

    fn last_modified(&mut self) -> Self::Time {
        self.asset.last_modified()
    }

    fn sources(&mut self, walker: &mut dyn SourceWalker<Self>) {
        self.asset.sources(&mut |source| walker(source));
    }
}

pub trait Mapper<'a, A: Asset + 'a>:
    FnMut(<A as AssetLifetime<'a>>::Output) -> <Self as Mapper<'a, A>>::Output + Sized
{
    type Output;
}
impl<'a, A: 'a, F, O> Mapper<'a, A> for F
where
    A: Asset,
    F: FnMut(<A as AssetLifetime<'a>>::Output) -> O,
{
    type Output = O;
}

pub trait Outlives<'a, ImplicitBounds = &'a Self> {}
impl<'a, T: ?Sized> Outlives<'a> for T {}
