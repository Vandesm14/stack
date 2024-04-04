use crate::{interner::interner, EvalError, Expr, Program};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program.funcs.insert(
    interner().get_or_intern_static("or"),
    |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      program.push_expr(Expr::Boolean(
        program.ast.is_truthy(lhs).unwrap()
          || program.ast.is_truthy(rhs).unwrap(),
      ))?;

      Ok(())
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("and"),
    |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      program.push_expr(Expr::Boolean(
        program.ast.is_truthy(lhs).unwrap()
          && program.ast.is_truthy(rhs).unwrap(),
      ))?;

      Ok(())
    },
  );

  Ok(())
}
