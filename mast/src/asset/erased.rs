#![allow(clippy::module_name_repetitions)]

use {
    crate::{
        asset::{self, Asset},
        bounds,
        time::Time,
    },
    ::core::convert::Infallible,
};

/// A trait-object-safe version of the [`Asset`] trait.
///
/// This trait's `'this` lifetime represents the lifetime of the trait object itself.
///
/// You can specify the output and source types of the type-erased asset
/// via setting the trait object's `Types` associated type
/// to a `dyn` trait object of [`ErasedTypes`].
///
/// # Examples
///
/// ```
/// use ::mast::asset::{self, Asset};
///
/// fn box_asset<'t, T: 't + Asset>(asset: T) -> Box<dyn 't + asset::Erased<
///     't,
///     Time = T::Time,
///     Types = dyn for<'a> asset::ErasedTypes<
///         'a,
///         't,
///         Output = <T as asset::Lifetime<'a>>::Output,
///         Source = <T as asset::Lifetime<'a>>::Source,
///     >,
/// >> {
///     Box::new(asset)
/// }
/// ```
#[must_use]
pub trait Erased<'this>: Sealed<'this> {}

/// Extension methods for [`Erased`] providing the regular methods of [`Asset`].
///
/// This can be used to implement [`Asset`]
/// for your own trait object types
/// that have [`Erased`] as a supertrait.
///
/// This has to be on a separate trait
/// to allow [`Erased`] to be object safe.
pub trait ErasedExt<'this>: Erased<'this> {
    /// Call [`Asset::generate`] on the inner asset.
    fn generate_erased(&mut self) -> <Self::Types as ErasedTypes<'_, 'this>>::Output {
        self.generate_inner(Token)
    }

    /// Call [`Asset::modified`] on the inner asset.
    fn modified_erased(&mut self) -> Self::Time {
        self.modified_inner(Token)
    }

    /// Call [`Asset::sources`] on the inner asset.
    #[allow(clippy::missing_errors_doc)] // see Asset::sources
    fn sources_erased<W, E>(&mut self, walker: &mut W) -> Result<(), E>
    where
        W: FnMut(<Self::Types as ErasedTypes<'_, 'this>>::Source) -> Result<(), E>,
    {
        let mut res = Ok(());
        self.sources_inner(Token, &mut |item| res = walker(item));
        res
    }
}
impl<'this, T: ?Sized + Erased<'this>> ErasedExt<'this> for T {}

/// A token passed into the methods of `ErasedSealed`
/// that cannot be externally constructed.
///
/// This is to prevent users from calling `ErasedSealed` methods
/// when they have that trait in scope,
/// which can happen if `Erased` is used as a trait bound on a function.
#[allow(missing_debug_implementations)]
pub struct Token;

pub trait Sealed<'this>: 'this {
    type Types: ?Sized + for<'a> ErasedTypes<'a, 'this>;

    fn generate_inner(&mut self, token: Token) -> <Self::Types as ErasedTypes<'_, 'this>>::Output;

    type Time: Time;
    fn modified_inner(&mut self, token: Token) -> Self::Time;

    fn sources_inner(&mut self, token: Token, walker: DynSourceWalker<'_, 'this, Self>);
}

/// Types associated with a type-erased asset for a given lifetime.
///
/// This trait should only be used as a `dyn` trait object
/// as a kind of HKT;
/// see [`Erased`] for more.
pub trait ErasedTypes<'a, 'this, ImplicitBounds: bounds::Sealed = bounds::Bounds<&'a &'this ()>>:
    'this + TypesSealed
{
    /// The equivalent of [`asset::Lifetime::Output`].
    type Output;

    /// The equivalent of [`asset::Lifetime::Source`].
    type Source;
}

// Intentionally not implemented for any types.
pub trait TypesSealed {}

type DynSourceWalker<'walker, 'this, A> = &'walker mut dyn for<'a> ErasedSourceWalker<
    'a,
    'this,
    <<A as Sealed<'this>>::Types as ErasedTypes<'a, 'this>>::Source,
>;

pub trait ErasedSourceWalker<'a, 'this, S, ImplicitBounds = (&'a &'this (), &'a S)>:
    FnMut(S)
{
}
impl<'a, 'this, S, F> ErasedSourceWalker<'a, 'this, S> for F where F: ?Sized + FnMut(S) {}

impl<'this, A: 'this + ?Sized + Asset> Erased<'this> for A {}

impl<'this, A: 'this + ?Sized + Asset> Sealed<'this> for A {
    type Types = dyn 'this
        + for<'a> ErasedTypes<
            'a,
            'this,
            Output = <A as asset::Lifetime<'a>>::Output,
            Source = <A as asset::Lifetime<'a>>::Source,
        >;

    fn generate_inner(&mut self, Token: Token) -> <A as asset::Lifetime<'_>>::Output {
        Asset::generate(self)
    }

    type Time = A::Time;
    fn modified_inner(&mut self, Token: Token) -> Self::Time {
        A::modified(self)
    }

    fn sources_inner(&mut self, Token: Token, walker: DynSourceWalker<'_, 'this, Self>) {
        A::sources(
            self,
            &mut asset::funnel_source_walker(|source| {
                walker(source);
                Ok::<_, Infallible>(())
            }),
        )
        .unwrap_or_else(|infallible| match infallible {});
    }
}

macro_rules! impl_asset_for_trait_objects {
    ($($ty:ty,)*) => { $(
        impl<'a, 'this, T, Types> asset::Lifetime<'a> for $ty
        where
            T: Time,
            Types: ?Sized + for<'b> ErasedTypes<'b, 'this>,
        {
            type Output = <Types as ErasedTypes<'a, 'this>>::Output;
            type Source = <Types as ErasedTypes<'a, 'this>>::Source;
        }

        impl<'this, T, Types> Asset for $ty
        where
            T: Time,
            Types: ?Sized + for<'b> ErasedTypes<'b, 'this>,
        {
            fn generate(&mut self) -> asset::Output<'_, Self> {
                self.generate_erased()
            }

            type Time = T;
            fn modified(&mut self) -> Self::Time {
                self.modified_erased()
            }

            fn sources<W: asset::SourceWalker<Self>>(&mut self, walker: &mut W) -> Result<(), W::Error> {
                self.sources_erased(walker)
            }
        }
    )* }
}

impl_asset_for_trait_objects! {
    dyn 'this + Erased<'this, Time = T, Types = Types>,
    dyn 'this + Send + Erased<'this, Time = T, Types = Types>,
    dyn 'this + Sync + Erased<'this, Time = T, Types = Types>,
    dyn 'this + Send + Sync + Erased<'this, Time = T, Types = Types>,
}

#[test]
#[cfg(feature = "fs")]
fn is_object_safe() {
    use {super::constant, ::std::time::SystemTime};

    fn funnel<F: FnMut(&u32) -> &u32>(f: F) -> F {
        f
    }

    let mut asset = constant(5).map(funnel(|x| x));
    assert_is_asset(&asset);

    fn type_erase<'t, T>(
        val: &mut T,
    ) -> &mut (dyn 't
                 + Erased<
        't,
        Types = dyn 't + for<'a> ErasedTypes<'a, 't, Output = &'a u32, Source = ()>,
        Time = SystemTime,
    >)
    where
        T: 't
            + Asset<Time = SystemTime>
            + for<'a> asset::Lifetime<'a, Output = &'a u32, Source = ()>,
    {
        val
    }

    let erased = type_erase(&mut asset);

    assert_is_asset(&erased);

    fn assert_is_asset<A>(_: &A)
    where
        A: ?Sized
            + Asset<Time = SystemTime>
            + for<'a> asset::Lifetime<'a, Output = &'a u32, Source = ()>,
    {
    }
}
