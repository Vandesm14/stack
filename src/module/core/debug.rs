use crate::{interner::interner, EvalError, Program};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program.funcs.insert(
    interner().get_or_intern_static("panic"),
    |program, trace_expr| {
      let string = program.pop(trace_expr)?;

      Err(EvalError {
        expr: trace_expr,
        program: program.clone(),
        message: format!(
          "panic: {}",
          program
            .ast
            .expr(string)
            .map(|expr| expr.to_string())
            .unwrap_or("no error message".into())
        ),
      })
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("debug"),
    |program, trace_expr| {
      let expr = program.pop_expr(trace_expr)?;
      println!("{}", expr);
      Ok(())
    },
  );

  Ok(())
}
