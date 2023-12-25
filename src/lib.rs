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
    interner.get_or_intern_static(Intrinsic::Add.as_str());
    interner.get_or_intern_static(Intrinsic::Subtract.as_str());
    interner.get_or_intern_static(Intrinsic::Multiply.as_str());
    interner.get_or_intern_static(Intrinsic::Divide.as_str());
    interner.get_or_intern_static(Intrinsic::Remainder.as_str());

    // Comparison
    interner.get_or_intern_static(Intrinsic::Equal.as_str());
    interner.get_or_intern_static(Intrinsic::NotEqual.as_str());
    interner.get_or_intern_static(Intrinsic::GreaterThan.as_str());
    interner.get_or_intern_static(Intrinsic::LessThan.as_str());
    interner.get_or_intern_static(Intrinsic::Or.as_str());
    interner.get_or_intern_static(Intrinsic::And.as_str());

    // Code/IO
    interner.get_or_intern_static(Intrinsic::Parse.as_str());
    interner.get_or_intern_static(Intrinsic::ReadFile.as_str());
    interner.get_or_intern_static(Intrinsic::Syscall { arity: 0 }.as_str());
    interner.get_or_intern_static(Intrinsic::Syscall { arity: 1 }.as_str());
    interner.get_or_intern_static(Intrinsic::Syscall { arity: 2 }.as_str());
    interner.get_or_intern_static(Intrinsic::Syscall { arity: 3 }.as_str());
    interner.get_or_intern_static(Intrinsic::Syscall { arity: 4 }.as_str());
    interner.get_or_intern_static(Intrinsic::Syscall { arity: 5 }.as_str());
    interner.get_or_intern_static(Intrinsic::Syscall { arity: 6 }.as_str());

    // List
    interner.get_or_intern_static(Intrinsic::Explode.as_str());
    interner.get_or_intern_static(Intrinsic::Length.as_str());
    interner.get_or_intern_static(Intrinsic::Nth.as_str());
    interner.get_or_intern_static(Intrinsic::Join.as_str());
    interner.get_or_intern_static(Intrinsic::Insert.as_str());
    interner.get_or_intern_static(Intrinsic::ListPop.as_str());
    interner.get_or_intern_static(Intrinsic::ListShift.as_str());
    interner.get_or_intern_static(Intrinsic::Concat.as_str());
    interner.get_or_intern_static(Intrinsic::Unwrap.as_str());

    // Control Flow
    interner.get_or_intern_static(Intrinsic::IfElse.as_str());
    interner.get_or_intern_static(Intrinsic::If.as_str());
    interner.get_or_intern_static(Intrinsic::While.as_str());
    interner.get_or_intern_static(Intrinsic::Halt.as_str());

    // Scope
    interner.get_or_intern_static(Intrinsic::Set.as_str());
    interner.get_or_intern_static(Intrinsic::Get.as_str());
    interner.get_or_intern_static(Intrinsic::Unset.as_str());

    // Stack
    interner.get_or_intern_static(Intrinsic::Collect.as_str());
    interner.get_or_intern_static(Intrinsic::Clear.as_str());
    interner.get_or_intern_static(Intrinsic::Pop.as_str());
    interner.get_or_intern_static(Intrinsic::Dup.as_str());
    interner.get_or_intern_static(Intrinsic::Swap.as_str());
    interner.get_or_intern_static(Intrinsic::Rot.as_str());

    // Functions/Data
    interner.get_or_intern_static(Intrinsic::Call.as_str());
    interner.get_or_intern_static(Intrinsic::CallNative.as_str());
    interner.get_or_intern_static(Intrinsic::Lazy.as_str());
    interner.get_or_intern_static(Intrinsic::Noop.as_str());

    // Type
    interner.get_or_intern_static(Intrinsic::ToBoolean.as_str());
    interner.get_or_intern_static(Intrinsic::ToInteger.as_str());
    interner.get_or_intern_static(Intrinsic::ToFloat.as_str());
    interner.get_or_intern_static(Intrinsic::ToPointer.as_str());
    interner.get_or_intern_static(Intrinsic::ToList.as_str());
    interner.get_or_intern_static(Intrinsic::ToString.as_str());
    interner.get_or_intern_static(Intrinsic::ToCall.as_str());
    interner.get_or_intern_static(Intrinsic::TypeOf.as_str());

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
