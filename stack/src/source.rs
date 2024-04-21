use core::fmt;
use std::{io, path::Path, rc::Rc};

/// Contains information for a source.
#[derive(Debug, Clone, Eq)]
pub struct Source {
  name: Rc<str>,
  source: Rc<str>,
  line_starts: Rc<[usize]>,
}

impl Source {
  /// Creates a [`Source`].
  pub fn new<N, S>(name: N, source: S) -> Self
  where
    N: AsRef<str>,
    S: AsRef<str>,
  {
    Self {
      name: name.as_ref().into(),
      source: source.as_ref().into(),
      line_starts: core::iter::once(0)
        .chain(
          source
            .as_ref()
            .char_indices()
            .filter(|&(_, c)| c == '\n')
            .map(|(i, _)| i + 1),
        )
        .collect::<Vec<_>>()
        .into(),
    }
  }

  /// Creates a [`Source`] from a file at the provided `path`.
  pub fn from_path<P>(path: P) -> io::Result<Self>
  where
    P: AsRef<Path>,
  {
    let source = std::fs::read_to_string(&path)?;
    let name = path.as_ref().to_string_lossy();

    Ok(Self::new(name, source))
  }

  /// Returns a reference to the name.
  #[inline]
  pub fn name(&self) -> &str {
    &self.name
  }

  /// Returns a reference to the source.
  #[inline]
  pub fn source(&self) -> &str {
    &self.source
  }

  /// Returns a [`Location`] in the source.
  pub fn location(&self, index: usize) -> Option<Location> {
    if index > self.source.len() {
      None
    } else {
      let line = match self.line_starts.binary_search(&index) {
        Ok(x) => x,
        Err(x) => x - 1,
      };
      let line_start = self.line_starts[line];

      Some(Location {
        line,
        column: index - line_start,
      })
    }
  }

  /// Returns the <code>&[str]</code> for a line in the source.
  pub fn line(&self, line: usize) -> Option<&str> {
    if let Some(&line_start) = self.line_starts.get(line) {
      let line_end = self
        .line_starts
        .get(line + 1)
        .copied()
        .unwrap_or(self.source.len());
      Some(&self.source[line_start..line_end])
    } else {
      None
    }
  }
}

impl PartialEq for Source {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name && self.source == other.source
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Location {
  /// The line byte index.
  pub line: usize,
  /// The column byte index.
  pub column: usize,
}

impl fmt::Display for Location {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}:{}", self.line + 1, self.column + 1)
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
    Source::new("", source).line_starts.to_vec()
  }

  #[case("", 0 => Some(Location { line: 0, column: 0 }) ; "empty 0")]
  #[case("", 1 => None ; "empty 1")]
  #[case("hello", 0 => Some(Location { line: 0, column: 0 }) ; "single 0")]
  #[case("hello", 1 => Some(Location { line: 0, column: 1 }) ; "single 1")]
  #[case("hello", 5 => Some(Location { line: 0, column: 5 }) ; "single 5")]
  #[case("hello", 6 => None ; "single 6")]
  #[case("hello\n", 0 => Some(Location { line: 0, column: 0 }) ; "single with newline 0")]
  #[case("hello\n", 1 => Some(Location { line: 0, column: 1 }) ; "single with newline 1")]
  #[case("hello\n", 5 => Some(Location { line: 0, column: 5 }) ; "single with newline 5")]
  #[case("hello\n", 6 => Some(Location { line: 1, column: 0 }) ; "single with newline 6")]
  #[case("hello\n", 7 => None ; "single with newline 7")]
  #[case("hello\nworld\n", 0 => Some(Location { line: 0, column: 0 }) ; "multiple 0")]
  #[case("hello\nworld\n", 1 => Some(Location { line: 0, column: 1 }) ; "multiple 1")]
  #[case("hello\nworld\n", 5 => Some(Location { line: 0, column: 5 }) ; "multiple 5")]
  #[case("hello\nworld\n", 6 => Some(Location { line: 1, column: 0 }) ; "multiple 6")]
  #[case("hello\nworld\n", 9 => Some(Location { line: 1, column: 3 }) ; "multiple 9")]
  fn location(source: &str, index: usize) -> Option<Location> {
    Source::new("", source).location(index)
  }

  #[case("", 0 => Some("".into()) ; "empty 0")]
  #[case("", 1 => None ; "empty 1")]
  #[case("hello", 0 => Some("hello".into()) ; "single 0")]
  #[case("hello", 1 => None ; "single 1")]
  #[case("hello\n", 0 => Some("hello\n".into()) ; "single with newline 0")]
  #[case("hello\n", 1 => Some("".into()) ; "single with newline 1")]
  #[case("hello\n", 2 => None ; "single with newline 2")]
  #[case("hello\nworld", 0 => Some("hello\n".into()) ; "multiple 0")]
  #[case("hello\nworld", 1 => Some("world".into()) ; "multiple 1")]
  #[case("hello\nworld", 2 => None ; "multiple 2")]
  fn line(source: &str, line: usize) -> Option<String> {
    Source::new("", source).line(line).map(Into::into)
  }
}
