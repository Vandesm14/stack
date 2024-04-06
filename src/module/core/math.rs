use crate::{interner::interner, EvalError, Expr, ExprKind, Program, Type};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program.funcs.insert(
    interner().get_or_intern_static("+"),
    |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      match lhs.val.val.coerce_same_float(&rhs.val.val) {
        Some((ExprKind::Integer(lhs), ExprKind::Integer(rhs))) => {
          program.push(ExprKind::Integer(lhs + rhs).into_expr())
        }
        Some((ExprKind::Float(lhs), ExprKind::Float(rhs))) => {
          program.push(ExprKind::Float(lhs + rhs).into_expr())
        }
        _ => Err(EvalError {
          expr: trace_expr.clone(),
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::List(vec![
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
            ]),
            Type::List(vec![lhs.val.type_of(), rhs.val.type_of()]),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("-"),
    |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      match lhs.val.coerce_same_float(&rhs.val) {
        Some((ExprKind::Integer(lhs), ExprKind::Integer(rhs))) => {
          program.push(ExprKind::Integer(lhs - rhs).into_expr())
        }
        Some((ExprKind::Float(lhs), ExprKind::Float(rhs))) => {
          program.push(ExprKind::Float(lhs - rhs).into_expr())
        }
        _ => Err(EvalError {
          expr: trace_expr.clone(),
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::List(vec![
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
            ]),
            Type::List(vec![lhs.val.type_of(), rhs.val.type_of()]),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("*"),
    |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      match lhs.val.coerce_same_float(&rhs.val) {
        Some((ExprKind::Integer(lhs), ExprKind::Integer(rhs))) => {
          program.push(ExprKind::Integer(lhs * rhs).into_expr())
        }
        Some((ExprKind::Float(lhs), ExprKind::Float(rhs))) => {
          program.push(ExprKind::Float(lhs * rhs).into_expr())
        }
        _ => Err(EvalError {
          expr: trace_expr.clone(),
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::List(vec![
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
            ]),
            Type::List(vec![lhs.val.type_of(), rhs.val.type_of()]),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("/"),
    |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      match lhs.val.coerce_same_float(&rhs.val) {
        Some((ExprKind::Integer(lhs), ExprKind::Integer(rhs))) => {
          program.push(ExprKind::Integer(lhs / rhs).into_expr())
        }
        Some((ExprKind::Float(lhs), ExprKind::Float(rhs))) => {
          program.push(ExprKind::Float(lhs / rhs).into_expr())
        }
        _ => Err(EvalError {
          expr: trace_expr.clone(),
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::List(vec![
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
            ]),
            Type::List(vec![lhs.val.type_of(), rhs.val.type_of()]),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("%"),
    |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      match lhs.val.coerce_same_float(&rhs.val) {
        Some((ExprKind::Integer(lhs), ExprKind::Integer(rhs))) => {
          program.push(ExprKind::Integer(lhs % rhs).into_expr())
        }
        Some((ExprKind::Float(lhs), ExprKind::Float(rhs))) => {
          program.push(ExprKind::Float(lhs % rhs).into_expr())
        }
        _ => Err(EvalError {
          expr: trace_expr.clone(),
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::List(vec![
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
            ]),
            Type::List(vec![lhs.val.type_of(), rhs.val.type_of()]),
          ),
        }),
      }
    },
  );

  Ok(())
}
