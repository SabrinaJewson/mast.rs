use {
    crate::{
        asset::{self, Asset},
        time::{self, Time},
    },
    ::core::marker::PhantomData,
};

/// An asset with an immutable value.
///
/// In constrast to [`constant`],
/// this type is for values that are determined at runtime
/// but aren't changed from that point on
/// (like `let` in Rust).
/// If you know the value at compile time,
/// use [`constant`] instead.
///
/// This asset outputs an `&mut V` for flexibility reasons,
/// but you should not be mutating its value.
///
/// [`constant`]: super::constant()
#[derive(Debug, Clone, Copy)]
pub struct Immutable<V, T, S> {
    value: V,
    created: T,
    _source: PhantomData<S>,
}

impl<V, T, S> Immutable<V, T, S> {
    /// Create a new immutable asset whose value is recorded as being created now.
    #[must_use]
    pub fn created_now(value: V) -> Self
    where
        T: time::Now,
    {
        Self::created_at(T::now(), value)
    }

    /// Create a new immutable asset
    /// whose value is recorded as being created at some specific point in time.
    #[must_use]
    pub const fn created_at(created: T, value: V) -> Self {
        Self {
            value,
            created,
            _source: PhantomData,
        }
    }

    /// Get a shared reference to the value stored by the asset.
    #[must_use]
    pub fn value(&self) -> &V {
        &self.value
    }
}

impl<V, T: Time, S: for<'a> asset::Source<'a>> Asset for Immutable<V, T, S> {
    type Output = fn(&()) -> &mut V;
    fn generate(&mut self) -> &mut V {
        &mut self.value
    }

    type Time = T;
    fn last_modified(&mut self) -> Self::Time {
        self.created.clone()
    }

    type Source = S;
    fn sources(&mut self, _walker: asset::SourceWalker<'_, Self>) {}
}