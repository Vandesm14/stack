use crate::{
  interner::interner, DebugData, EvalError, EvalErrorKind, ExprKind, Program,
  Type,
};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program.funcs.insert(
    interner().get_or_intern_static("+"),
    |program, trace_expr| {
      let rhs_expr = program.pop(trace_expr)?;
      let lhs_expr = program.pop(trace_expr)?;

      match lhs_expr.val.coerce_same_float(&rhs_expr.val) {
        Some((ExprKind::Integer(lhs), ExprKind::Integer(rhs))) => program
          .push(ExprKind::Integer(lhs + rhs).into_expr(DebugData::default())),
        Some((ExprKind::Float(lhs), ExprKind::Float(rhs))) => program
          .push(ExprKind::Float(lhs + rhs).into_expr(DebugData::default())),
        _ => Err(EvalError {
          expr: Some(trace_expr),
          kind: EvalErrorKind::ExpectedFound(
            Type::List(vec![
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
            ]),
            Type::List(vec![lhs_expr.val.type_of(), rhs_expr.val.type_of()]),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("-"),
    |program, trace_expr| {
      let rhs_expr = program.pop(trace_expr)?;
      let lhs_expr = program.pop(trace_expr)?;

      match lhs_expr.val.coerce_same_float(&rhs_expr.val) {
        Some((ExprKind::Integer(lhs), ExprKind::Integer(rhs))) => program
          .push(ExprKind::Integer(lhs - rhs).into_expr(DebugData::default())),
        Some((ExprKind::Float(lhs), ExprKind::Float(rhs))) => program
          .push(ExprKind::Float(lhs - rhs).into_expr(DebugData::default())),
        _ => Err(EvalError {
          expr: Some(trace_expr),
          kind: EvalErrorKind::ExpectedFound(
            Type::List(vec![
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
            ]),
            Type::List(vec![lhs_expr.val.type_of(), rhs_expr.val.type_of()]),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("*"),
    |program, trace_expr| {
      let rhs_expr = program.pop(trace_expr)?;
      let lhs_expr = program.pop(trace_expr)?;

      match lhs_expr.val.coerce_same_float(&rhs_expr.val) {
        Some((ExprKind::Integer(lhs), ExprKind::Integer(rhs))) => program
          .push(ExprKind::Integer(lhs * rhs).into_expr(DebugData::default())),
        Some((ExprKind::Float(lhs), ExprKind::Float(rhs))) => program
          .push(ExprKind::Float(lhs * rhs).into_expr(DebugData::default())),
        _ => Err(EvalError {
          expr: Some(trace_expr),
          kind: EvalErrorKind::ExpectedFound(
            Type::List(vec![
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
            ]),
            Type::List(vec![lhs_expr.val.type_of(), rhs_expr.val.type_of()]),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("/"),
    |program, trace_expr| {
      let rhs_expr = program.pop(trace_expr)?;
      let lhs_expr = program.pop(trace_expr)?;

      match lhs_expr.val.coerce_same_float(&rhs_expr.val) {
        Some((ExprKind::Integer(lhs), ExprKind::Integer(rhs))) => program
          .push(ExprKind::Integer(lhs / rhs).into_expr(DebugData::default())),
        Some((ExprKind::Float(lhs), ExprKind::Float(rhs))) => program
          .push(ExprKind::Float(lhs / rhs).into_expr(DebugData::default())),
        _ => Err(EvalError {
          expr: Some(trace_expr),
          kind: EvalErrorKind::ExpectedFound(
            Type::List(vec![
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
            ]),
            Type::List(vec![lhs_expr.val.type_of(), rhs_expr.val.type_of()]),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("%"),
    |program, trace_expr| {
      let rhs_expr = program.pop(trace_expr)?;
      let lhs_expr = program.pop(trace_expr)?;

      match lhs_expr.val.coerce_same_float(&rhs_expr.val) {
        Some((ExprKind::Integer(lhs), ExprKind::Integer(rhs))) => program
          .push(ExprKind::Integer(lhs % rhs).into_expr(DebugData::default())),
        Some((ExprKind::Float(lhs), ExprKind::Float(rhs))) => program
          .push(ExprKind::Float(lhs % rhs).into_expr(DebugData::default())),
        _ => Err(EvalError {
          expr: Some(trace_expr),
          kind: EvalErrorKind::ExpectedFound(
            Type::List(vec![
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
            ]),
            Type::List(vec![lhs_expr.val.type_of(), rhs_expr.val.type_of()]),
          ),
        }),
      }
    },
  );

  Ok(())
}
