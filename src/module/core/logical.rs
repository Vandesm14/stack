use internment::Intern;

use crate::{DebugData, EvalError, ExprKind, Program};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program
    .funcs
    .insert(Intern::from_ref("or"), |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      program.push(
        ExprKind::Boolean(lhs.val.is_truthy() || rhs.val.is_truthy())
          .into_expr(DebugData::default()),
      )
    });

  program
    .funcs
    .insert(Intern::from_ref("and"), |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      program.push(
        ExprKind::Boolean(lhs.val.is_truthy() && rhs.val.is_truthy())
          .into_expr(DebugData::default()),
      )
    });

  Ok(())
}
