use crate::{
  interner::interner, DebugData, EvalError, Expr, ExprKind, Program,
};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program.funcs.insert(
    interner().get_or_intern_static("or"),
    |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      program.push(
        ExprKind::Boolean(lhs.val.is_truthy() || rhs.val.is_truthy())
          .into_expr(DebugData::only_ingredients(vec![
            lhs,
            rhs,
            trace_expr.clone(),
          ])),
      )
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("and"),
    |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      program.push(
        ExprKind::Boolean(lhs.val.is_truthy() && rhs.val.is_truthy())
          .into_expr(DebugData::only_ingredients(vec![
            lhs,
            rhs,
            trace_expr.clone(),
          ])),
      )
    },
  );

  Ok(())
}
