/// Asset for [`Asset::map`].
pub struct Map<A, F> {
    asset: A,
    f: F,
}

impl<A, F> Map<A, F> {
    pub(crate) fn new(asset: A, f: F) -> Self {
        Self { asset, f }
    }
}

impl<A: Debug, F> Debug for Map<A, F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Map").field("asset", &self.asset).finish()
    }
}

impl<'c, A, F, O> Asset<'c> for Map<A, F>
where
    A: Asset<'c>,
    F: FnOnce(A::Output) -> O,
{
    type Etag = A::Etag;
    type Output = O;
    type Generator = Generator<A::Generator, F>;

    fn update(self, cx: Context<'c>, etag: &'c mut Self::Etag) -> Tracked<Self::Generator> {
        self.asset.update(cx, etag).map(|generator| Generator {
            generator,
            f: self.f,
        })
    }
}

pub struct Generator<G, F> {
    generator: G,
    f: F,
}

impl<G, F, O> super::Generator for Generator<G, F>
where
    G: super::Generator,
    F: FnOnce(G::Output) -> O,
{
    type Output = O;

    fn generate(self) -> Self::Output {
        (self.f)(self.generator.generate())
    }
}

use super::Asset;
use super::Context;
use crate::Tracked;
use core::fmt;
use core::fmt::Debug;
use core::fmt::Formatter;
