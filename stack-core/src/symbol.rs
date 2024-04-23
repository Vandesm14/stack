use core::{borrow::Borrow, fmt, hash::Hash};

use compact_str::CompactString;
use internment::Intern;

use crate::expr::ExprKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Symbol(Intern<CompactString>);

impl Symbol {
  /// Creates a [`Symbol`].
  #[inline]
  pub fn new(value: CompactString) -> Self {
    Self(Intern::new(value))
  }

  /// Creates a [`Symbol`] from a reference.
  #[inline]
  pub fn from_ref<'q, Q>(value: &'q Q) -> Self
  where
    Q: 'q + ?Sized + Eq + Hash,
    CompactString: Borrow<Q> + From<&'q Q>,
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

impl From<ExprKind> for Symbol {
  fn from(value: ExprKind) -> Self {
    Self::from_ref(value.to_string().as_str())
  }
}
