use {
    super::{Asset, Output, SourceWalker, Types},
    crate::time::Time,
    ::core::marker::PhantomData,
};

/// Create an asset with a constant value.
///
/// Like with `const` in Rust,
/// this should only be used for values that can't change between invocations of the binary.
/// If your value is determined at runtime but immutable
/// (like command-line arguments)
/// then use [`Immutable`](super::Immutable) instead.
///
/// The resulting asset outputs an `&mut V` for flexibility reasons,
/// but you should not be mutating its value.
#[must_use]
pub const fn constant<V, T, S>(value: V) -> Constant<V, T, S> {
    Constant {
        value,
        _time: PhantomData,
        _source: PhantomData,
    }
}

/// An asset with a constant value, created by [`constant`].
#[derive(Debug, Clone, Copy)]
pub struct Constant<V, T, S> {
    value: V,
    _time: PhantomData<T>,
    _source: PhantomData<S>,
}

impl<V, T, S> Constant<V, T, S> {
    /// Get a shared reference to the value stored by the asset.
    #[must_use]
    pub fn value(&self) -> &V {
        &self.value
    }
}

impl<'a, V, T: Time, S> Types<'a> for Constant<V, T, S> {
    type Output = &'a mut V;
    type Source = S;
}

impl<V, T: Time, S> Asset for Constant<V, T, S> {
    fn generate(&mut self) -> Output<'_, Self> {
        &mut self.value
    }

    type Time = T;
    fn last_modified(&mut self) -> Self::Time {
        T::earliest()
    }

    fn sources<W: SourceWalker<Self>>(&mut self, _: &mut W) -> Result<(), W::Error> {
        Ok(())
    }
}
