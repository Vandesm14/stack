use crate::{interner::interner, EvalError, Expr, Program};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program.funcs.insert(
    interner().get_or_intern_static("="),
    |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      program.push(Expr::Boolean(lhs == rhs))
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("!="),
    |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      program.push(Expr::Boolean(lhs != rhs))
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("<"),
    |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      program.push(Expr::Boolean(lhs < rhs))
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static(">"),
    |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      program.push(Expr::Boolean(lhs > rhs))
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("<="),
    |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      program.push(Expr::Boolean(lhs <= rhs))
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static(">="),
    |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      program.push(Expr::Boolean(lhs >= rhs))
    },
  );

  Ok(())
}
