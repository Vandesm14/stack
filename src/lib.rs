mod evaluator;
mod expr;
mod intrinsics;
mod lexer;
mod parser;

pub use evaluator::*;
pub use expr::*;
pub use intrinsics::*;
pub use lexer::*;
pub use parser::*;

use lasso::{Rodeo, Spur};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
  interner: Rodeo<Spur>,
}

impl Context {
  pub fn new() -> Self {
    let mut interner = Rodeo::new();

    // TODO: Add all of the intrinsics.
    interner.get_or_intern_static("");

    interner.get_or_intern_static("+");
    interner.get_or_intern_static("-");
    interner.get_or_intern_static("*");
    interner.get_or_intern_static("/");
    interner.get_or_intern_static("%");
    interner.get_or_intern_static("=");
    interner.get_or_intern_static("!=");
    interner.get_or_intern_static(">");
    interner.get_or_intern_static("<");

    interner.get_or_intern_static("'");

    interner.get_or_intern_static("(");
    interner.get_or_intern_static(")");
    interner.get_or_intern_static("{");
    interner.get_or_intern_static("}");
    interner.get_or_intern_static("[");
    interner.get_or_intern_static("]");

    interner.get_or_intern_static("and");
    interner.get_or_intern_static("or");

    interner.get_or_intern_static("pop");
    interner.get_or_intern_static("dup");
    interner.get_or_intern_static("swap");
    interner.get_or_intern_static("rot");

    interner.get_or_intern_static("fn");

    Self { interner }
  }

  #[inline]
  pub fn intern<T>(&mut self, s: T) -> Spur
  where
    T: AsRef<str>,
  {
    self.interner.get_or_intern(s)
  }

  #[inline]
  pub fn resolve(&self, s: &Spur) -> &str {
    self.interner.resolve(s)
  }
}

impl Default for Context {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}
