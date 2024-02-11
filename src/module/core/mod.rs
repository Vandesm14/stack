use crate::{EvalError, Module, Program};

pub mod cast;
pub mod compare;
pub mod control_flow;
pub mod debug;
pub mod eval;
pub mod io;
pub mod list;
pub mod logical;
pub mod math;
pub mod scope;
pub mod stack;

pub struct Core {
  pub eval: bool,
}

impl Module for Core {
  fn link(&self, program: &mut Program) -> Result<(), EvalError> {
    stack::module(program)?;
    scope::module(program)?;
    math::module(program)?;
    compare::module(program)?;
    logical::module(program)?;
    list::module(program)?;
    cast::module(program)?;
    debug::module(program)?;
    control_flow::module(program)?;
    io::module(program)?;
    eval::module(program)?;

    let core_lib = include_str!("./core.stack");
    program.eval_string(core_lib)?;

    Ok(())
  }
}

impl Default for Core {
  #[inline]
  fn default() -> Self {
    Self { eval: true }
  }
}
