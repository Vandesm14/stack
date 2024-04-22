pub mod chain;
pub mod context;
pub mod engine;
pub mod expr;
pub mod intrinsic;
pub mod journal;
pub mod lexer;
pub mod module;
pub mod parser;
pub mod scope;
pub mod source;
pub mod symbol;
pub mod unit;

pub mod prelude {
  //! Re-exports commonly used items.

  use super::*;

  pub use context::Context;
  pub use engine::{Engine, RunError, RunErrorReason};
  pub use expr::{Error, Expr, ExprInfo, ExprKind};
  pub use intrinsic::Intrinsic;
  pub use lexer::Lexer;
  pub use module::Module;
  pub use parser::{parse, ParseError, ParseErrorKind};
  pub use source::Source;
  pub use symbol::Symbol;
}

mod vec_one;
