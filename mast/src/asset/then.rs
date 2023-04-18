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

impl<A, F, InnerEtag, InnerOutput> Asset for Then<A, F>
where
    A: Asset,
    F: for<'cx, 'e> FnOnce1<Tracked<A::Generator<'cx, 'e>>>,
    for<'cx, 'e> <F as FnOnce1<Tracked<A::Generator<'cx, 'e>>>>::Output:
        Asset<Etag = InnerEtag, Output = InnerOutput>,
    InnerEtag: Etag,
{
    type Etag = (A::Etag, InnerEtag);
    type Output = InnerOutput;
    type Generator<'cx, 'e> =
        <<F as FnOnce1<Tracked<A::Generator<'cx, 'e>>>>::Output as Asset>::Generator<'cx, 'e>;
    fn update<'cx, 'e>(
        self,
        cx: Context<'cx>,
        etag: &'e mut Self::Etag,
    ) -> Tracked<Self::Generator<'cx, 'e>> {
        (self.f)(self.asset.update(cx, &mut etag.0)).update(cx, &mut etag.1)
    }
}

pub trait FnOnce1<P1>: FnOnce(P1) -> <Self as FnOnce1<P1>>::Output {
    type Output;
}
impl<P1, O, F: FnOnce(P1) -> O> FnOnce1<P1> for F {
    type Output = O;
}

use super::Asset;
use super::Context;
use crate::Etag;
use crate::Tracked;
use core::fmt;
use core::fmt::Debug;
use core::fmt::Formatter;
