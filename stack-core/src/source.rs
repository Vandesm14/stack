// See the file footer for the licence text.

use core::{fmt, num::NonZeroUsize};
use std::{fs, io, path::Path, sync::Arc};

use unicode_segmentation::UnicodeSegmentation;

/// Contains metadata for a source.
///
/// This internally stores an [`Rc`], hence it is *cheap* to clone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source(Arc<SourceInner>);

impl Source {
  /// Creates a new [`Source`].
  pub fn new<N, S>(name: N, source: S) -> Self
  where
    N: Into<String>,
    S: Into<String>,
  {
    let name = name.into();
    let source = source.into();
    let line_starts = core::iter::once(0)
      .chain(
        source
          .char_indices()
          .filter(|&(_, c)| c == '\n')
          .map(|(i, _)| i + 1),
      )
      .collect::<Vec<_>>();

    Self(Arc::new(SourceInner {
      name,
      source,
      line_starts,
    }))
  }

  /// Creates a new [`Source`] from the contents read from a file.
  pub fn from_path<P>(path: P) -> io::Result<Self>
  where
    P: AsRef<Path>,
  {
    let source = fs::read_to_string(&path)?;
    let name = path.as_ref().to_string_lossy().into_owned();

    Ok(Self::new(name, source))
  }

  /// Returns the name as a <code>&[str]</code>.
  #[inline]
  #[must_use]
  pub fn name(&self) -> &str {
    self.0.name.as_str()
  }

  /// Returns the source as a <code>&[str]</code>.
  #[inline]
  #[must_use]
  pub fn source(&self) -> &str {
    self.0.source.as_str()
  }

  /// Returns the [`Location`] calculated from a byte index.
  ///
  /// [`None`] is returned when `index` is out-of-bounds, or `index` is not on
  /// UTF-8 sequence boundaries.
  ///
  /// This internally uses a binary search.
  #[must_use]
  pub fn location(&self, index: usize) -> Option<Location> {
    if index > self.0.source.len() {
      None
    } else {
      let line = match self.0.line_starts.binary_search(&index) {
        Ok(x) => x,
        Err(x) => x - 1,
      };

      let line_start = self.0.line_starts[line];
      let line_str = self.line(NonZeroUsize::new(line + 1).unwrap()).unwrap();

      let column_index = index - line_start;
      let column = line_str.get(0..column_index)?.graphemes(true).count();

      Some(Location {
        line: NonZeroUsize::new(line + 1).unwrap(),
        column: NonZeroUsize::new(column + 1).unwrap(),
      })
    }
  }

  /// Returns the line <code>&[str]</code> from a line number.
  ///
  /// The line number can be calculated via [`location`].
  ///
  /// [`location`]: Self::location
  #[must_use]
  pub fn line(&self, line: NonZeroUsize) -> Option<&str> {
    if let Some(&line_start) = self.0.line_starts.get(line.get() - 1) {
      let line_end = self
        .0
        .line_starts
        .get(line.get())
        .copied()
        .unwrap_or(self.0.source.len());
      Some(&self.0.source[line_start..line_end])
    } else {
      None
    }
  }
}

/// A human-readable location in a [`Source`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Location {
  /// The line number.
  pub line: NonZeroUsize,
  /// The column number.
  pub column: NonZeroUsize,
}

impl fmt::Display for Location {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}:{}", self.line, self.column)
  }
}

#[derive(Debug, Clone, Eq)]
struct SourceInner {
  name: String,
  source: String,
  line_starts: Vec<usize>,
}

impl PartialEq for SourceInner {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name && self.source == other.source
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use test_case::case;

  #[case("" => vec![0] ; "empty")]
  #[case("hello\n" => vec![0, 6] ; "one")]
  #[case("hello\nthere\r\nworld\n" => vec![0, 6, 13, 19] ; "multiple")]
  fn line_starts(source: &str) -> Vec<usize> {
    Source::new("", source).0.line_starts.to_vec()
  }

  #[case("", 0 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(1).unwrap() }) ; "empty 0")]
  #[case("", 1 => None ; "empty 1")]
  #[case("hello", 0 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(1).unwrap() }) ; "single 0")]
  #[case("hello", 1 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(2).unwrap() }) ; "single 1")]
  #[case("hello", 2 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(3).unwrap() }) ; "single 2")]
  #[case("hello", 3 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(4).unwrap() }) ; "single 3")]
  #[case("hello", 4 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(5).unwrap() }) ; "single 4")]
  #[case("hello", 5 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(6).unwrap() }) ; "single 5")]
  #[case("hello", 6 => None ; "single 6")]
  #[case("h💣llo", 0 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(1).unwrap() }) ; "single with bomb 0")]
  #[case("h💣llo", 1 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(2).unwrap() }) ; "single with bomb 1")]
  #[case("h💣llo", 2 => None ; "single with bomb 2")]
  #[case("h💣llo", 3 => None ; "single with bomb 3")]
  #[case("h💣llo", 4 => None ; "single with bomb 4")]
  #[case("h💣llo", 5 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(3).unwrap() }) ; "single with bomb 5")]
  #[case("h💣llo", 6 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(4).unwrap() }) ; "single with bomb 6")]
  #[case("h💣llo", 7 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(5).unwrap() }) ; "single with bomb 7")]
  #[case("h💣llo", 8 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(6).unwrap() }) ; "single with bomb 8")]
  #[case("h💣llo", 9 => None ; "single with bomb 9")]
  #[case("hello\n", 0 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(1).unwrap() }) ; "single with newline 0")]
  #[case("hello\n", 1 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(2).unwrap() }) ; "single with newline 1")]
  #[case("hello\n", 2 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(3).unwrap() }) ; "single with newline 2")]
  #[case("hello\n", 3 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(4).unwrap() }) ; "single with newline 3")]
  #[case("hello\n", 4 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(5).unwrap() }) ; "single with newline 4")]
  #[case("hello\n", 5 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(6).unwrap() }) ; "single with newline 5")]
  #[case("hello\n", 6 => Some(Location { line: NonZeroUsize::new(2).unwrap(), column: NonZeroUsize::new(1).unwrap() }) ; "single with newline 6")]
  #[case("hello\n", 7 => None ; "single with newline 7")]
  #[case("h💣llo\n", 0 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(1).unwrap() }) ; "single with bomb with newline 0")]
  #[case("h💣llo\n", 1 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(2).unwrap() }) ; "single with bomb with newline 1")]
  #[case("h💣llo\n", 2 => None ; "single with bomb with newline 2")]
  #[case("h💣llo\n", 3 => None ; "single with bomb with newline 3")]
  #[case("h💣llo\n", 4 => None ; "single with bomb with newline 4")]
  #[case("h💣llo\n", 5 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(3).unwrap() }) ; "single with bomb with newline 5")]
  #[case("h💣llo\n", 6 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(4).unwrap() }) ; "single with bomb with newline 6")]
  #[case("h💣llo\n", 7 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(5).unwrap() }) ; "single with bomb with newline 7")]
  #[case("h💣llo\n", 8 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(6).unwrap() }) ; "single with bomb with newline 8")]
  #[case("h💣llo\n", 9 => Some(Location { line: NonZeroUsize::new(2).unwrap(), column: NonZeroUsize::new(1).unwrap() }) ; "single with bomb with newline 9")]
  #[case("hello\nworld\n", 0 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(1).unwrap() }) ; "multiple 0")]
  #[case("hello\nworld\n", 1 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(2).unwrap() }) ; "multiple 1")]
  #[case("hello\nworld\n", 2 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(3).unwrap() }) ; "multiple 2")]
  #[case("hello\nworld\n", 3 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(4).unwrap() }) ; "multiple 3")]
  #[case("hello\nworld\n", 4 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(5).unwrap() }) ; "multiple 4")]
  #[case("hello\nworld\n", 5 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(6).unwrap() }) ; "multiple 5")]
  #[case("hello\nworld\n", 6 => Some(Location { line: NonZeroUsize::new(2).unwrap(), column: NonZeroUsize::new(1).unwrap() }) ; "multiple 6")]
  #[case("hello\nworld\n", 7 => Some(Location { line: NonZeroUsize::new(2).unwrap(), column: NonZeroUsize::new(2).unwrap() }) ; "multiple 7")]
  #[case("hello\nworld\n", 8 => Some(Location { line: NonZeroUsize::new(2).unwrap(), column: NonZeroUsize::new(3).unwrap() }) ; "multiple 8")]
  #[case("hello\nworld\n", 9 => Some(Location { line: NonZeroUsize::new(2).unwrap(), column: NonZeroUsize::new(4).unwrap() }) ; "multiple 9")]
  #[case("hello\nworld\n", 10 => Some(Location { line: NonZeroUsize::new(2).unwrap(), column: NonZeroUsize::new(5).unwrap() }) ; "multiple 10")]
  #[case("hello\nworld\n", 11 => Some(Location { line: NonZeroUsize::new(2).unwrap(), column: NonZeroUsize::new(6).unwrap() }) ; "multiple 11")]
  #[case("hello\nworld\n", 12 => Some(Location { line: NonZeroUsize::new(3).unwrap(), column: NonZeroUsize::new(1).unwrap() }) ; "multiple 12")]
  #[case("hello\nworld\n", 13 => None ; "multiple 13")]
  #[case("h💣llo\nworld\n", 0 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(1).unwrap() }) ; "multiple with bomb 0")]
  #[case("h💣llo\nworld\n", 1 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(2).unwrap() }) ; "multiple with bomb 1")]
  #[case("h💣llo\nworld\n", 2 => None ; "multiple with bomb 2")]
  #[case("h💣llo\nworld\n", 3 => None ; "multiple with bomb 3")]
  #[case("h💣llo\nworld\n", 4 => None ; "multiple with bomb 4")]
  #[case("h💣llo\nworld\n", 5 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(3).unwrap() }) ; "multiple with bomb 5")]
  #[case("h💣llo\nworld\n", 6 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(4).unwrap() }) ; "multiple with bomb 6")]
  #[case("h💣llo\nworld\n", 7 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(5).unwrap() }) ; "multiple with bomb 7")]
  #[case("h💣llo\nworld\n", 8 => Some(Location { line: NonZeroUsize::new(1).unwrap(), column: NonZeroUsize::new(6).unwrap() }) ; "multiple with bomb 8")]
  #[case("h💣llo\nworld\n", 9 => Some(Location { line: NonZeroUsize::new(2).unwrap(), column: NonZeroUsize::new(1).unwrap() }) ; "multiple with bomb 9")]
  #[case("h💣llo\nworld\n", 10 => Some(Location { line: NonZeroUsize::new(2).unwrap(), column: NonZeroUsize::new(2).unwrap() }) ; "multiple with bomb 10")]
  #[case("h💣llo\nworld\n", 11 => Some(Location { line: NonZeroUsize::new(2).unwrap(), column: NonZeroUsize::new(3).unwrap() }) ; "multiple with bomb 11")]
  #[case("h💣llo\nworld\n", 12 => Some(Location { line: NonZeroUsize::new(2).unwrap(), column: NonZeroUsize::new(4).unwrap() }) ; "multiple with bomb 12")]
  #[case("h💣llo\nworld\n", 13 => Some(Location { line: NonZeroUsize::new(2).unwrap(), column: NonZeroUsize::new(5).unwrap() }) ; "multiple with bomb 13")]
  #[case("h💣llo\nworld\n", 14 => Some(Location { line: NonZeroUsize::new(2).unwrap(), column: NonZeroUsize::new(6).unwrap() }) ; "multiple with bomb 14")]
  #[case("h💣llo\nworld\n", 15 => Some(Location { line: NonZeroUsize::new(3).unwrap(), column: NonZeroUsize::new(1).unwrap() }) ; "multiple with bomb 15")]
  #[case("h💣llo\nworld\n", 16 => None ; "multiple with bomb 16")]
  fn location(source: &str, index: usize) -> Option<Location> {
    Source::new("", source).location(index)
  }

  #[case("", NonZeroUsize::new(1).unwrap() => Some("".into()) ; "empty 1")]
  #[case("", NonZeroUsize::new(2).unwrap() => None ; "empty 2")]
  #[case("hello", NonZeroUsize::new(1).unwrap() => Some("hello".into()) ; "single 1")]
  #[case("hello", NonZeroUsize::new(2).unwrap() => None ; "single 2")]
  #[case("h💣llo", NonZeroUsize::new(1).unwrap() => Some("h💣llo".into()) ; "single with bomb 1")]
  #[case("h💣llo", NonZeroUsize::new(2).unwrap() => None ; "single with bomb 2")]
  #[case("hello\n", NonZeroUsize::new(1).unwrap() => Some("hello\n".into()) ; "single with newline 1")]
  #[case("hello\n", NonZeroUsize::new(2).unwrap() => Some("".into()) ; "single with newline 2")]
  #[case("hello\n", NonZeroUsize::new(3).unwrap() => None ; "single with newline 3")]
  #[case("h💣llo\n", NonZeroUsize::new(1).unwrap() => Some("h💣llo\n".into()) ; "single with bomb with newline 1")]
  #[case("h💣llo\n", NonZeroUsize::new(2).unwrap() => Some("".into()) ; "single with bomb with newline 2")]
  #[case("h💣llo\n", NonZeroUsize::new(3).unwrap() => None ; "single with bomb with newline 3")]
  #[case("hello\nworld", NonZeroUsize::new(1).unwrap() => Some("hello\n".into()) ; "multiple 1")]
  #[case("hello\nworld", NonZeroUsize::new(2).unwrap() => Some("world".into()) ; "multiple 2")]
  #[case("hello\nworld", NonZeroUsize::new(3).unwrap() => None ; "multiple 3")]
  #[case("h💣llo\nworld", NonZeroUsize::new(1).unwrap() => Some("h💣llo\n".into()) ; "multiple with bomb 1")]
  #[case("h💣llo\nworld", NonZeroUsize::new(2).unwrap() => Some("world".into()) ; "multiple with bomb 2")]
  #[case("h💣llo\nworld", NonZeroUsize::new(3).unwrap() => None ; "multiple with bomb 3")]
  fn line(source: &str, line: NonZeroUsize) -> Option<String> {
    Source::new("", source).line(line).map(Into::into)
  }
}

// SPDX: 0BSD
//
// Copyright (C) Leon Davis <leonskidev@pm.me>
// Copyright (C) Shane Vandegrift <vandesm14@gmail.com>
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted.
//
// THE SOFTWARE IS PROVIDED “AS IS” AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
// REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
// AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
// INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
// LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
// OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
// PERFORMANCE OF THIS SOFTWARE.
