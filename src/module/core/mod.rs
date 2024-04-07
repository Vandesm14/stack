use crate::{EvalError, Module, Program};

pub mod cast;
pub mod compare;
pub mod control_flow;
#[allow(clippy::module_inception)]
pub mod core;
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
    // Native Intrinsics
    stack::module.link(program)?;
    scope::module.link(program)?;
    math::module.link(program)?;
    compare::module.link(program)?;
    logical::module.link(program)?;
    list::module.link(program)?;
    cast::module.link(program)?;
    debug::module.link(program)?;
    control_flow::module.link(program)?;
    io::module.link(program)?;
    eval::module.link(program)?;

    // In-Language Definitions
    core::module(program)?;

    Ok(())
  }
}

impl Default for Core {
  #[inline]
  fn default() -> Self {
    Self { eval: true }
  }
}
