#![cfg_attr(doc_nightly, doc(cfg(feature = "fs")))]
//! Assets that interact with the filesystem.

mod path;
pub use path::{path, Path};
