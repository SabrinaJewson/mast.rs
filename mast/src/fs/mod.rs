#![cfg_attr(doc_nightly, doc(cfg(feature = "fs")))]
//! Assets that interact with the filesystem.

use ::std::{fs, path::Path as StdPath, time::SystemTime};

mod path;
pub use path::{path, Path};

mod cached;
pub use cached::Cached;

pub mod bytes;
pub use bytes::{bytes, Bytes};

pub mod text;
pub use text::{text, Text};

pub mod dir;
pub use dir::{dir, Dir};

/// Utility function to obtain the "last modified" date of a path on the filesystem.
///
/// This intentionally returns an `Option` instead of a `Result`
/// because if an error if encountered
/// you will generally not want to report it,
/// but instead fall back on [`Time::earliest`](crate::time::Time::earliest)
/// or use some other default.
pub fn path_modified<P: AsRef<StdPath>>(path: P) -> Option<SystemTime> {
    fs::symlink_metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
}
