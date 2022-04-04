//! Reading the contents of a directory.

use {
    crate::asset::{self, Asset},
    ::std::{
        error::Error as StdError,
        fmt::{self, Display, Formatter},
        format, fs, io,
        path::Path,
        string::String,
        time::SystemTime,
    },
};

/// Create an asset that reads the top-level contents of a directory.
///
/// This function is logically equivalent to [`path`](super::path())
/// followed by [`::std::fs::read_dir`],
/// but is more convenient and provides better error messages.
///
/// The returned asset outputs a [`Result<'_>`],
/// which is an alias for `Result<`[`Entries<'_>`]`, `[`Error`]`>`.
///
/// [`Result<'_>`]: Result
/// [`Entries<'_>`]: Entries
pub fn dir<P: AsRef<Path>>(path: P) -> Dir<P> {
    Dir {
        inner: super::path(path),
    }
}

/// An asset that reads the top-level contents of a directory,
/// created by [`dir()`].
#[derive(Debug, Clone)]
#[must_use]
pub struct Dir<P> {
    inner: super::Path<P>,
}

impl<'a, P: AsRef<Path>> asset::Lifetime<'a> for Dir<P> {
    type Output = Result<'a>;
    type Source = &'a Path;
}

impl<P: AsRef<Path>> Asset for Dir<P> {
    fn generate(&mut self) -> asset::Output<'_, Self> {
        let base = self.inner.generate();
        Ok(Entries {
            inner: fs::read_dir(base).map_err(|inner| Error {
                message: format!("failed to open directory `{}`", base.display()),
                inner,
            })?,
            base,
        })
    }

    type Time = SystemTime;
    fn modified(&mut self) -> Self::Time {
        self.inner.modified()
    }

    fn sources<W: asset::SourceWalker<Self>>(
        &mut self,
        walker: &mut W,
    ) -> core::result::Result<(), W::Error> {
        self.inner.sources(walker)
    }
}

/// An iterator over the contents of a directory.
#[derive(Debug)]
pub struct Entries<'a> {
    inner: fs::ReadDir,
    base: &'a Path,
}

impl<'a> Entries<'a> {
    /// Returns the path of the directory being read.
    #[must_use]
    pub fn base(&self) -> &'a Path {
        self.base
    }
}

impl Iterator for Entries<'_> {
    type Item = ReadResult;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.inner.next()?.map_err(|inner| ReadError {
            message: format!("error reading directory `{}`", self.base.display()),
            inner,
        });
        Some(res)
    }
}

/// Type alias for [`::core::result::Result`]`<`[`Entries<'_>`]`, `[`Error`]`>`.
///
/// [`Entries<'_>`]: Entries
pub type Result<'a> = core::result::Result<Entries<'a>, Error>;

/// An error opening a directory.
#[derive(Debug)]
pub struct Error {
    message: String,
    inner: io::Error,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&*self.message)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.inner)
    }
}

/// Type alias for [`::core::result::Result`]`<`[`std::fs::DirEntry`]`, `[`ReadError`]`>`.
pub type ReadResult = core::result::Result<fs::DirEntry, ReadError>;

/// An error while reading a directory.
#[derive(Debug)]
pub struct ReadError {
    message: String,
    inner: io::Error,
}

impl Display for ReadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&*self.message)
    }
}

impl StdError for ReadError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.inner)
    }
}
