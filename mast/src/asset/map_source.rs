use super::{Asset, Output, Source, SourceWalker};

/// An asset whose [source](Asset::Source) is mapped to another type by a closure,
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

impl<A, F> Asset for MapSource<A, F>
where
    A: Asset,
    F: SourceMapper<A>,
{
    type Output = A::Output;
    fn generate(&mut self) -> <Self::Output as Output<'_>>::Type {
        self.asset.generate()
    }

    type Time = A::Time;
    fn last_modified(&mut self) -> Self::Time {
        self.asset.last_modified()
    }

    type Source = <F as SourceMapper<A>>::Output;
    fn sources(&mut self, walker: SourceWalker<'_, Self>) {
        self.asset
            .sources(&mut |source| walker(self.mapper.call(source)));
    }
}

// Why do I have two layers of trait indirection here?
// I have no idea.
// But I'm not going to touch it, because it works.
pub trait SourceMapper<A: Asset> {
    type Output: for<'a> Source<'a>;
    fn call<'a>(
        &mut self,
        source: <A::Source as Source<'a>>::Type,
    ) -> <Self::Output as Source<'a>>::Type;
}

impl<A, F> SourceMapper<A> for F
where
    A: Asset,
    F: ?Sized + for<'a> FnMut1<<A::Source as Source<'a>>::Type>,
{
    type Output = fn(&()) -> <F as FnMut1<<A::Source as Source<'_>>::Type>>::Output;
    fn call<'a>(
        &mut self,
        source: <A::Source as Source<'a>>::Type,
    ) -> <Self::Output as Source<'a>>::Type {
        self(source)
    }
}

pub trait FnMut1<I>: FnMut(I) -> <Self as FnMut1<I>>::Output {
    type Output;
}
impl<F, I, O> FnMut1<I> for F
where
    F: ?Sized + FnMut(I) -> O,
{
    type Output = O;
}
