//! Easily thread miscellaneous miscellaneous context types to all [`Asset`]s.

/// A type used to easily thread miscellaneous context types to all [`Asset`](super::Asset)s.
#[derive(Clone, Copy)]
pub struct Context<'cx> {
    inner: &'cx dyn Inner,
}

impl<'cx> Context<'cx> {
    fn with_values<O>(self, f: impl FnOnce(&[&'cx dyn Value]) -> O) -> O {
        let mut output = None;
        let mut f = Some(f);
        self.inner
            .__with_values_erased(Token, &mut |slice| output = Some(f.take().unwrap()(slice)));
        output.unwrap()
    }

    #[track_caller]
    fn new<I: Inner>(inner: &'cx I) -> Self {
        let this = Self { inner };
        let res = this.with_values(|values| {
            for (i, &lhs) in values.iter().enumerate() {
                for &rhs in &values[i + 1..] {
                    if lhs.__as_dyn_any(Token).type_id() == rhs.__as_dyn_any(Token).type_id() {
                        return Err(lhs.__type_name(Token));
                    }
                }
            }
            Ok(())
        });
        if let Err(type_name) = res {
            panic!("attempted to add more than one {type_name} to `Context`");
        }
        this
    }

    /// Construct a new `Context` from an array of references to [`Value`]s.
    ///
    /// This is most convenient when
    /// you wish to retain local ownership of the values outside of the [`Context`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use mast::asset;
    /// #[derive(Debug)]
    /// struct MyContextType(String);
    ///
    /// #[derive(Debug)]
    /// struct OtherContextType(u32);
    ///
    /// let my = MyContextType("Hello world!".to_owned());
    /// let other = OtherContextType(37);
    /// let cx = [&my as &dyn asset::context::Value, &other];
    /// let cx = asset::Context::from_array(&cx);
    /// assert_eq!(cx.get::<MyContextType>().0, "Hello world!");
    /// assert_eq!(cx.get::<OtherContextType>().0, 37);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if there is more than one instance of any given type.
    #[must_use]
    #[track_caller]
    pub fn from_array<const N: usize>(inner: &'cx [&'cx dyn Value; N]) -> Self {
        Self::new(inner)
    }

    /// Construct a new `Context` from a tuple of values.
    ///
    /// This is most convenient when
    /// you are constructing values exclusively to be used inside the [`Context`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use mast::asset;
    /// #[derive(Debug)]
    /// struct MyContextType(String);
    ///
    /// #[derive(Debug)]
    /// struct OtherContextType(u32);
    ///
    /// let cx = (MyContextType("Hello world!".to_owned()), OtherContextType(37));
    /// let cx = asset::Context::from_tuple(&cx);
    /// assert_eq!(cx.get::<MyContextType>().0, "Hello world!");
    /// assert_eq!(cx.get::<OtherContextType>().0, 37);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if there is more than one instance of any given type.
    #[must_use]
    #[track_caller]
    pub fn from_tuple<T: Tuple>(tuple: &'cx T) -> Self {
        Self::new(tuple)
    }

    /// Retrieve a reference to the value of type `T` stored in the `Context`.
    ///
    /// # Panics
    ///
    /// Panics if there is no value of type `T` in the context.
    #[must_use]
    #[track_caller]
    pub fn get<T: 'static>(self) -> &'cx T {
        match self.try_get() {
            Some(val) => val,
            None => panic!("no value of type {} found in `Context`", type_name::<T>()),
        }
    }

    /// Attempt to retrieve a reference to the value of type `T` stored in the `Context`.
    /// Returns [`None`] if there is no `T` in the context.
    ///
    /// # Example
    ///
    /// ```
    /// # use mast::asset;
    /// #[derive(Debug, PartialEq, Eq)]
    /// struct MyContextType(u32);
    ///
    /// let cx = (MyContextType(37),);
    /// let cx = asset::Context::from_tuple(&cx);
    /// assert_eq!(cx.try_get::<MyContextType>(), Some(&MyContextType(37)));
    /// assert_eq!(cx.try_get::<u32>(), None);
    /// ```
    #[must_use]
    pub fn try_get<T: 'static>(self) -> Option<&'cx T> {
        self.with_values(|values| {
            values
                .iter()
                .find_map(|&value| value.__as_dyn_any(Token).downcast_ref::<T>())
        })
    }
}

impl Debug for Context<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.with_values(|values| {
            f.debug_map()
                .entries(
                    values
                        .iter()
                        .map(|&value| (value.__type_name(Token), value)),
                )
                .finish()
        })
    }
}

impl Default for Context<'_> {
    fn default() -> Self {
        Self::from_tuple(&())
    }
}

/// A value that can be stored in a [`Context`].
///
/// This is automatically implemented any type that is:
/// - `'static`
/// - [`Sync`]
/// - [`Debug`]
pub trait Value: sealed::Value {}

impl<T: 'static + Sync + Debug> Value for T {}

/// A tuple of [`Value`]s.
///
/// This tuple is automatically implemented for any tuple of [`Value`]s.
pub trait Tuple: Inner {}

mod sealed {
    pub trait Inner: Sync {
        fn __with_values_erased<'this>(
            &'this self,
            token: Token,
            f: &mut dyn FnMut(&[&'this dyn super::Value]),
        );
    }
    pub trait Value: 'static + Sync + Debug {
        fn __type_name(&self, token: Token) -> &'static str;
        fn __as_dyn_any(&self, token: Token) -> &dyn Any;
    }
    impl<T: 'static + Sync + Debug> Value for T {
        fn __type_name(&self, Token: Token) -> &'static str {
            type_name::<T>()
        }
        fn __as_dyn_any(&self, Token: Token) -> &dyn Any {
            self
        }
    }

    // Since our traits have public traits subtraits,
    // bounding on that trait also enables calling methods from its supertraits.
    // We prevent this by requiring an unconstructable `Token` type.
    #[allow(missing_debug_implementations)]
    pub struct Token;
    use core::any::type_name;
    use core::any::Any;
    use core::fmt::Debug;
}
use sealed::Inner;
use sealed::Token;

impl<const N: usize> Inner for [&'_ dyn Value; N] {
    fn __with_values_erased<'this>(
        &'this self,
        Token: Token,
        f: &mut dyn FnMut(&[&'this dyn Value]),
    ) {
        f(self);
    }
}

macro_rules! impl_for_tuple {
        ($name:ident: $($t:ident)*) => {
            #[allow(non_snake_case)]
            impl<$($t: Value,)*> Inner for ($($t,)*) {
                fn __with_values_erased<'this>(
                    &'this self,
                    Token: Token,
                    f: &mut dyn FnMut(&[&'this dyn Value]),
                ) {
                    let ($($t,)*) = self;
                    $(let $t: &'this dyn Value = $t;)*
                    f(&[$($t,)*]);
                }
            }
            impl<$($t: Value,)*> Tuple for ($($t,)*) {}
        };
    }
crate::for_tuples!(impl_for_tuple);

use core::any::type_name;
use core::fmt;
use core::fmt::Debug;
use core::fmt::Formatter;
