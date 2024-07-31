use core::{borrow::Borrow, fmt, hash::Hash};

use compact_str::{CompactString, ToCompactString};
use internment::Intern;
use serde::{
  de::{self, Visitor},
  Deserialize, Deserializer,
};

use crate::expr::ExprKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Symbol(Intern<CompactString>);

impl<'de> Deserialize<'de> for Symbol {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    struct SymbolVisitor;

    impl<'de> Visitor<'de> for SymbolVisitor {
      type Value = Symbol;

      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string")
      }

      fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        // Convert the &str to CompactString and intern it
        let compact_string = CompactString::new(value);
        Ok(Symbol(Intern::new(compact_string)))
      }

      fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        // Convert the String to CompactString and intern it
        let compact_string = CompactString::new(value);
        Ok(Symbol(Intern::new(compact_string)))
      }
    }

    deserializer.deserialize_str(SymbolVisitor)
  }
}

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
    Self::from_ref(value.to_compact_string().as_str())
  }
}
