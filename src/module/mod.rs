pub mod core;

use crate::{EvalError, Expr, Program};

// TODO: Check for name collisions with other modules.

pub type Func = fn(&mut Program, &Expr) -> Result<(), EvalError>;

pub trait Module {
  fn link(&self, program: &mut Program) -> Result<(), EvalError>;
}

impl<F> Module for F
where
  F: Fn(&mut Program) -> Result<(), EvalError>,
{
  #[inline]
  fn link(&self, program: &mut Program) -> Result<(), EvalError> {
    self(program)
  }
}
