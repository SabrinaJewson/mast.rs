use {
    crate::{
        asset::{self, Asset},
        time::Time,
    },
    ::std::{path::Path as StdPath, time::SystemTime},
};

/// Create a no-op asset that sources its modification time from a path on the filesystem.
///
/// This is useful as a base asset
/// to all other higher-level assets that deal with file sources
/// (like [`text`](mod@super::text)).
/// Without it,
/// Mast will not be able to correctly track the modification time of the resultant assets
/// which can lead to redundant or skipped rebuilds.
///
/// The returned asset outputs a shared reference to the [`Path`](StdPath) given into it,
/// because it might be useful for users.
pub fn path<P: AsRef<StdPath>>(path: P) -> Path<P> {
    Path { path }
}

/// A no-op asset that sources its modification time from a path on the filesystem.
#[derive(Debug, Clone, Copy)]
#[must_use]
pub struct Path<P> {
    path: P,
}

impl<'a, P: AsRef<StdPath>> asset::Types<'a> for Path<P> {
    type Output = &'a StdPath;
    type Source = &'a StdPath;
}

impl<P: AsRef<StdPath>> Asset for Path<P> {
    fn generate(&mut self) -> asset::Output<'_, Self> {
        self.path.as_ref()
    }

    type Time = SystemTime;
    fn modified(&mut self) -> Self::Time {
        crate::fs::path_modified(self.path.as_ref()).unwrap_or_else(SystemTime::earliest)
    }

    fn sources<W: asset::SourceWalker<Self>>(&mut self, walker: &mut W) -> Result<(), W::Error> {
        walker(self.path.as_ref())
    }
}
