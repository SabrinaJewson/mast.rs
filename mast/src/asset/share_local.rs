use {
    crate::asset::{self, Asset},
    ::core::cell::RefCell,
};

/// An asset implementing [`asset::Shared`]
/// by use of single-threaded shared mutability,
/// created by [`Asset::share_local`].
#[derive(Debug)]
#[must_use]
pub struct ShareLocal<A> {
    asset: RefCell<A>,
}

impl<A> ShareLocal<A> {
    pub(crate) fn new(asset: A) -> Self {
        Self {
            asset: RefCell::new(asset),
        }
    }
}

impl<'a, A: asset::FixedOutput> asset::Lifetime<'a> for ShareLocal<A> {
    type Output = A::FixedOutput;
    type Source = asset::Source<'a, A>;
}

impl<A: asset::FixedOutput> Asset for ShareLocal<A> {
    type Time = A::Time;

    asset::forward_to_shared!();
}

impl<A: asset::FixedOutput> asset::Shared for ShareLocal<A> {
    fn generate_shared(&self) -> asset::Output<'_, Self> {
        self.asset.borrow_mut().generate()
    }

    fn modified_shared(&self) -> Self::Time {
        self.asset.borrow_mut().modified()
    }

    fn sources_shared<W: asset::SourceWalker<Self>>(&self, walker: &mut W) -> Result<(), W::Error> {
        self.asset.borrow_mut().sources(walker)
    }
}
