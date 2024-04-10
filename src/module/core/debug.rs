use internment::Intern;

use crate::{EvalError, EvalErrorKind, Program};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program
    .funcs
    .insert(Intern::from_ref("panic"), |program, trace_expr| {
      let string = program.pop(trace_expr)?;

      Err(EvalError {
        expr: Some(trace_expr.clone()),
        kind: EvalErrorKind::Panic(format!("{}", string.val)),
      })
    });

  program
    .funcs
    .insert(Intern::from_ref("debug"), |program, trace_expr| {
      let item = program.pop(trace_expr)?;
      println!("{}", item);
      Ok(())
    });

  Ok(())
}
