use std::{
  io,
  path::{Path, PathBuf},
};

/// Implemented by types that represent a source file.
pub trait Source {
  /// Returns a reference to the source file [`Path`].
  fn path(&self) -> &Path;

  /// Returns a reference to the source file <code>&[str]</code> contents.
  fn content(&self) -> &str;
}

/// A [`Source`] read from a file path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileSource {
  path: PathBuf,
  content: String,
}

impl FileSource {
  /// Creates a [`FileSource`] by reading the contents of the file at the
  /// provided `path`.
  pub fn new(path: PathBuf) -> Result<Self, io::Error> {
    Ok(Self {
      content: std::fs::read_to_string(&path)?,
      path,
    })
  }
}

impl Source for FileSource {
  #[inline]
  fn path(&self) -> &Path {
    &self.path
  }

  #[inline]
  fn content(&self) -> &str {
    &self.content
  }
}

#[cfg(test)]
pub mod test {
  use super::*;

  /// A [`Source`] for use in tests.
  ///
  /// The [`TestSource::path`] method always returns the same [`Path`].
  pub struct TestSource {
    content: String,
  }

  impl TestSource {
    /// Creates a [`TestSource`] from a <code>&[str]</code>.
    #[inline]
    pub fn new<T>(content: T) -> Self
    where
      T: Into<String>,
    {
      Self {
        content: content.into(),
      }
    }
  }

  impl Source for TestSource {
    #[inline]
    fn path(&self) -> &Path {
      Path::new("test")
    }

    #[inline]
    fn content(&self) -> &str {
      &self.content
    }
  }
}
