use crate::{EvalError, Program};

// TODO: Split `core` into `list` and `module` modules

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program.eval_file("std/core.stack")?;

  Ok(())
}
