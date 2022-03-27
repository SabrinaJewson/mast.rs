//! Mast is a flexible build system configured by Rust code.
#![warn(
    noop_method_call,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    missing_docs,
    missing_debug_implementations,
    clippy::pedantic
)]
#![cfg_attr(doc_nightly, feature(doc_cfg))]
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

pub mod time;
#[doc(no_inline)]
pub use time::Time;

pub mod asset;
#[doc(no_inline)]
pub use asset::Asset;

#[cfg(feature = "fs")]
pub mod fs;
