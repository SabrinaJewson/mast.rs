//! Assets implementing [`asset::Shared`](crate::asset::Shared)
//! by use of a mutex exclusion primitive.
//!
//! Mast is intentionally generic over underlying mutex implementations,
//! so this module enables you to easily choose which one to use.
//! Each mutex implementation comes with
//! an `Asset` extension trait providing a `.share_mutex()` method
//! and the actual asset wrapper type.
//! You can import the extension trait of your chosen mutex
//! (and potentially the type as well if you need to name it)
//! and then call `.share_mutex()`
//! which will automatically use your preferred mutex.
//! For example, if you want to use std's built-in mutex:
//!
//! ```
//! use ::mast::asset::{self, Asset, share_mutex::StdExt as _};
//!
//! let asset = asset::constant(5)
//!     .map(|x: &i32| *x)
//!     .cache()
//!     .map(|x: &i32| *x)
//!     .share_mutex();
//! # type_infer(asset).generate();
//! # fn type_infer<A: Asset<Time = std::time::SystemTime, Source = ()>>(a: A) -> A { a }
//! ```
//!
//! This system enables your code
//! to avoid repeating the chosen mutex's name more than once,
//! making it easy to switch from one mutex implementation to another.

#[cfg(feature = "std")]
mod std_impl {
    use {
        crate::asset::{self, Asset},
        ::std::sync::Mutex,
    };

    /// A shared asset backed by the standard library's mutex type.
    #[cfg_attr(doc_nightly, doc(cfg(feature = "std")))]
    #[derive(Debug)]
    #[must_use]
    pub struct Std<A> {
        asset: Mutex<A>,
    }

    impl<'a, A: asset::FixedOutput> asset::Lifetime<'a> for Std<A> {
        type Output = A::FixedOutput;
        type Source = asset::Source<'a, A>;
    }

    impl<A: asset::FixedOutput> Asset for Std<A> {
        type Time = A::Time;

        asset::forward_to_shared!();
    }

    impl<A: asset::FixedOutput> asset::Shared for Std<A> {
        fn generate_shared(&self) -> asset::Output<'_, Self> {
            self.asset.lock().unwrap().generate()
        }

        fn modified_shared(&self) -> Self::Time {
            self.asset.lock().unwrap().modified()
        }

        fn sources_shared<W: asset::SourceWalker<Self>>(
            &self,
            walker: &mut W,
        ) -> Result<(), W::Error> {
            self.asset.lock().unwrap().sources(walker)
        }
    }

    /// Extension trait for [`Asset`] for use with the standard library mutex.
    #[cfg_attr(doc_nightly, doc(cfg(feature = "std")))]
    pub trait StdExt: Sized + asset::FixedOutput {
        /// Implement [`asset::Shared`] for this asset using a mutual exclusion primitive.
        fn share_mutex(self) -> Std<Self> {
            self.share_mutex_std()
        }

        /// The same as above,
        /// but with a disambiguated name
        /// for when you have multiple `Ext` traits in scope.
        fn share_mutex_std(self) -> Std<Self> {
            Std {
                asset: Mutex::new(self),
            }
        }
    }
    impl<T: asset::FixedOutput> StdExt for T {}
}
#[cfg(feature = "std")]
pub use std_impl::{Std, StdExt};

#[cfg(feature = "lock_api_04")]
mod lock_api_04_impl {
    use {
        crate::asset::{self, Asset},
        ::lock_api_04::{Mutex, RawMutex},
    };

    /// A shared asset backed by a mutex compatible with [`lock_api`](lock_api_04) 0.4.
    #[cfg_attr(doc_nightly, doc(cfg(feature = "lock_api_04")))]
    #[derive(Debug)]
    #[must_use]
    pub struct LockApi04<R: RawMutex, A> {
        asset: Mutex<R, A>,
    }

    impl<'a, R: RawMutex, A: asset::FixedOutput> asset::Lifetime<'a> for LockApi04<R, A> {
        type Output = A::FixedOutput;
        type Source = asset::Source<'a, A>;
    }

    impl<R: RawMutex, A: asset::FixedOutput> Asset for LockApi04<R, A> {
        type Time = A::Time;

        asset::forward_to_shared!();
    }

    impl<R: RawMutex, A: asset::FixedOutput> asset::Shared for LockApi04<R, A> {
        fn generate_shared(&self) -> asset::Output<'_, Self> {
            self.asset.lock().generate()
        }

        fn modified_shared(&self) -> Self::Time {
            self.asset.lock().modified()
        }

        fn sources_shared<W: asset::SourceWalker<Self>>(
            &self,
            walker: &mut W,
        ) -> Result<(), W::Error> {
            self.asset.lock().sources(walker)
        }
    }

    /// Extension trait for [`Asset`] for use with the standard library mutex.
    #[cfg_attr(doc_nightly, doc(cfg(feature = "lock_api_04")))]
    pub trait LockApi04Ext: Sized + asset::FixedOutput {
        /// Implement [`asset::Shared`] for this asset using a mutual exclusion primitive.
        fn share_mutex<R: RawMutex>(self) -> LockApi04<R, Self> {
            self.share_mutex_lock_api_04()
        }

        /// The same as above,
        /// but with a disambiguated name
        /// for when you have multiple `Ext` traits in scope.
        fn share_mutex_lock_api_04<R: RawMutex>(self) -> LockApi04<R, Self> {
            LockApi04 {
                asset: Mutex::new(self),
            }
        }
    }
    impl<T: asset::FixedOutput> LockApi04Ext for T {}
}
#[cfg(feature = "lock_api_04")]
pub use lock_api_04_impl::{LockApi04, LockApi04Ext};
