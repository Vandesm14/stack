use crate::{EvalError, Program};

// TODO: Split `core` into `list` and `module` modules

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  let core_lib = include_str!("./core.stack");
  program.eval_string(core_lib)?;

  Ok(())
}
