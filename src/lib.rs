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

      // Arithmetic
    interner.get_or_intern_static("+");
    interner.get_or_intern_static("-");
    interner.get_or_intern_static("*");
    interner.get_or_intern_static("/");
    interner.get_or_intern_static("%");

    // Comparison
    interner.get_or_intern_static("=");
    interner.get_or_intern_static("!=");
    interner.get_or_intern_static(">");
    interner.get_or_intern_static("<");
    interner.get_or_intern_static("or");
    interner.get_or_intern_static("and");

    // Code/IO
    interner.get_or_intern_static("parse");
    interner.get_or_intern_static("read-file");
    interner.get_or_intern_static("syscall0");
    interner.get_or_intern_static("syscall1");
    interner.get_or_intern_static("syscall2");
    interner.get_or_intern_static("syscall3");
    interner.get_or_intern_static("syscall4");
    interner.get_or_intern_static("syscall5");
    interner.get_or_intern_static("syscall6");

    // List
    interner.get_or_intern_static("explode");
    interner.get_or_intern_static("len");
    interner.get_or_intern_static("nth");
    interner.get_or_intern_static("join");
    interner.get_or_intern_static("insert");
    interner.get_or_intern_static("list-pop");
    interner.get_or_intern_static("list-shift");
    interner.get_or_intern_static("concat");
    interner.get_or_intern_static("unwrap");

    // Control Flow
    interner.get_or_intern_static("ifelse");
    interner.get_or_intern_static("if");
    interner.get_or_intern_static("while");
    interner.get_or_intern_static("halt");

    // Scope
    interner.get_or_intern_static("set");
    interner.get_or_intern_static("get");
    interner.get_or_intern_static("unset");

    // Stack
    interner.get_or_intern_static("collect");
    interner.get_or_intern_static("clear");
    interner.get_or_intern_static("pop");
    interner.get_or_intern_static("dup");
    interner.get_or_intern_static("swap");
    interner.get_or_intern_static("rot");

    // Functions/Data
    interner.get_or_intern_static("call");
    interner.get_or_intern_static("call_native");
    interner.get_or_intern_static("lazy");

    // Type
    interner.get_or_intern_static("toboolean");
    interner.get_or_intern_static("tointeger");
    interner.get_or_intern_static("tofloat");
    interner.get_or_intern_static("topointer");
    interner.get_or_intern_static("tolist");
    interner.get_or_intern_static("tostring");
    interner.get_or_intern_static("tocall");
    interner.get_or_intern_static("typeof");

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
