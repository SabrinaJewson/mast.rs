use super::{Asset, Output, SourceWalker};

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

impl<A, F> Asset for Map<A, F>
where
    A: Asset,
    F: for<'a> Mapper<'a, A>,
{
    type Output = fn(&()) -> <F as Mapper<'_, A>>::Output;
    fn generate(&mut self) -> <F as Mapper<'_, A>>::Output {
        (self.mapper)(self.asset.generate())
    }

    type Time = A::Time;
    fn last_modified(&mut self) -> Self::Time {
        self.asset.last_modified()
    }

    type Source = A::Source;
    fn sources(&mut self, walker: SourceWalker<'_, Self>) {
        self.asset.sources(&mut |source| walker(source));
    }
}

pub trait Mapper<'a, A: Asset>:
    FnMut(<A::Output as Output<'a>>::Type) -> <Self as Mapper<'a, A>>::Output + Sized
{
    type Output;
}
impl<'a, A: Asset, F, O> Mapper<'a, A> for F
where
    F: FnMut(<A::Output as Output<'a>>::Type) -> O,
{
    type Output = O;
}
