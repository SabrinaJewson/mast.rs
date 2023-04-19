/// Asset for [`Asset::then`].
pub struct Then<A, F> {
    asset: A,
    f: F,
}

impl<A, F> Then<A, F> {
    pub(crate) fn new(asset: A, f: F) -> Self {
        Self { asset, f }
    }
}

impl<A: Debug, F> Debug for Then<A, F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Then").field("asset", &self.asset).finish()
    }
}

impl<'c, A1, A2, F> Asset<'c> for Then<A1, F>
where
    A1: Asset<'c>,
    F: FnOnce(Tracked<A1::Generator>) -> A2,
    A2: Asset<'c>,
{
    type Etag = (A1::Etag, A2::Etag);
    type Output = A2::Output;
    type Generator = A2::Generator;
    fn update(self, cx: Context<'c>, etag: &'c mut Self::Etag) -> Tracked<Self::Generator> {
        (self.f)(self.asset.update(cx, &mut etag.0)).update(cx, &mut etag.1)
    }
}

use super::Asset;
use super::Context;
use crate::Tracked;
use core::fmt;
use core::fmt::Debug;
use core::fmt::Formatter;
