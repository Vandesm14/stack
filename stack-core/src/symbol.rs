use core::{borrow::Borrow, fmt, hash::Hash};

use internment::Intern;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Symbol(Intern<String>);

impl Symbol {
  /// Creates a [`Symbol`].
  #[inline]
  pub fn new(value: String) -> Self {
    Self(Intern::new(value))
  }

  /// Creates a [`Symbol`] from a reference.
  #[inline]
  pub fn from_ref<'q, Q>(value: &'q Q) -> Self
  where
    Q: 'q + ?Sized + Eq + Hash,
    String: Borrow<Q> + From<&'q Q>,
  {
    Self(Intern::from_ref(value))
  }

  /// Returns the <code>&[str]</code> for this [`Symbol`].
  #[inline]
  pub fn as_str(&self) -> &str {
    self.0.as_str()
  }
}

impl fmt::Display for Symbol {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.as_str())
  }
}
