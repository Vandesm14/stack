use crate::{interner::interner, EvalError, EvalErrorKind, Program};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program.funcs.insert(
    interner().get_or_intern_static("panic"),
    |program, trace_expr| {
      let string = program.pop(trace_expr)?;

      Err(EvalError {
        expr: Some(trace_expr),
        kind: EvalErrorKind::Panic(format!("{}", string.val)),
      })
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("debug"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;
      println!("{}", item);
      Ok(())
    },
  );

  Ok(())
}
