use {
    crate::{
        asset::{self, Asset},
        time::Time,
    },
    ::std::{fs, path::Path as StdPath, time::SystemTime},
};

/// Create an asset that sources from a path on the filesystem.
///
/// This type outputs a shared reference to the `Path` given into it,
/// because it might be useful for users.
pub fn path<P: AsRef<StdPath>>(path: P) -> Path<P> {
    Path { path }
}

/// No-op asset that sources from a path on the filesystem.
#[derive(Debug, Clone, Copy)]
#[must_use]
pub struct Path<P> {
    path: P,
}

impl<P: AsRef<StdPath>> Asset for Path<P> {
    type Output = fn(&()) -> &StdPath;
    fn generate(&mut self) -> <Self::Output as asset::Output<'_>>::Type {
        self.path.as_ref()
    }

    type Time = SystemTime;
    fn last_modified(&mut self) -> Self::Time {
        fs::symlink_metadata(self.path.as_ref())
            .and_then(|meta| meta.modified())
            .ok()
            .unwrap_or_else(SystemTime::earliest)
    }

    type Source = fn(&()) -> &StdPath;
    fn sources(&mut self, walker: asset::SourceWalker<'_, Self>) {
        walker.visit(self.path.as_ref());
    }
}
