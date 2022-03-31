//! Reading an entire text file from a path.

use {
    crate::asset::{self, Asset},
    ::std::{
        error::Error as StdError,
        fmt::{self, Display, Formatter},
        format,
        fs::File,
        io::{self, Read as _},
        path::Path,
        string::String,
        time::SystemTime,
    },
};

/// Create an asset that reads a text file from a path.
///
/// This function is logically equivalent to [`path`](super::path())
/// followed by [`::std::fs::read_to_string`],
/// but is more efficient, convenient and provides better error messages.
///
/// The returned asset outputs an `(&`[`Path`]`, `[`Result`]`<&mut `[`String`]`>)` tuple, giving
/// the path of the read file
/// and an exclusive reference to a buffer containing the contents of it
/// respectively.
pub fn text<P: AsRef<Path>>(path: P) -> Text<P> {
    Text {
        inner: super::path(path),
        buffer: String::new(),
    }
}

/// An asset that reads a text file from a path, created by [`text`].
#[derive(Debug, Clone)]
#[must_use]
pub struct Text<P> {
    inner: super::Path<P>,
    buffer: String,
}

impl<'a, P: AsRef<Path>> asset::Types<'a> for Text<P> {
    type Output = (&'a Path, Result<&'a mut String>);
    type Source = &'a Path;
}

impl<P: AsRef<Path>> Asset for Text<P> {
    fn generate(&mut self) -> asset::Output<'_, Self> {
        let path = self.inner.generate();

        self.buffer.clear();

        let res = (|| File::open(path)?.read_to_string(&mut self.buffer))()
            .map_err(|inner| Error {
                message: format!("failed to read file `{}`", path.display()),
                inner,
            })
            .map(|_| &mut self.buffer);

        (path, res)
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

/// Type alias for [`::core::result::Result`]`<T, `[`Error`]`>`.
pub type Result<T> = core::result::Result<T, Error>;

/// An error that occurred reading a text file.
#[derive(Debug)]
pub struct Error {
    message: String,
    inner: io::Error,
}

impl Error {
    /// Classify the kind of error that occurred as an [`io::ErrorKind`].
    #[must_use]
    pub fn io_kind(&self) -> io::ErrorKind {
        self.inner.kind()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.inner)
    }
}
