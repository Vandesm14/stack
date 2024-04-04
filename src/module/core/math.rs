use crate::{interner::interner, EvalError, Expr, Program, Type};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program.funcs.insert(
    interner().get_or_intern_static("+"),
    |program, trace_expr| {
      let rhs = program.pop_expr(trace_expr)?;
      let lhs = program.pop_expr(trace_expr)?;

      match lhs.coerce_same_float(&rhs) {
        Some((Expr::Integer(lhs), Expr::Integer(rhs))) => {
          program.push_expr(Expr::Integer(lhs + rhs))?;
          Ok(())
        }
        Some((Expr::Float(lhs), Expr::Float(rhs))) => {
          program.push_expr(Expr::Float(lhs + rhs))?;
          Ok(())
        }
        _ => Err(EvalError {
          expr: trace_expr,
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::List(vec![
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
            ]),
            Type::List(vec![
              lhs.type_of(&program.ast),
              rhs.type_of(&program.ast)
            ]),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("-"),
    |program, trace_expr| {
      let rhs = program.pop_expr(trace_expr)?;
      let lhs = program.pop_expr(trace_expr)?;

      match lhs.coerce_same_float(&rhs) {
        Some((Expr::Integer(lhs), Expr::Integer(rhs))) => {
          program.push_expr(Expr::Integer(lhs - rhs))?;
          Ok(())
        }
        Some((Expr::Float(lhs), Expr::Float(rhs))) => {
          program.push_expr(Expr::Float(lhs - rhs))?;
          Ok(())
        }
        _ => Err(EvalError {
          expr: trace_expr,
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::List(vec![
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
            ]),
            Type::List(vec![
              lhs.type_of(&program.ast),
              rhs.type_of(&program.ast)
            ]),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("*"),
    |program, trace_expr| {
      let rhs = program.pop_expr(trace_expr)?;
      let lhs = program.pop_expr(trace_expr)?;

      match lhs.coerce_same_float(&rhs) {
        Some((Expr::Integer(lhs), Expr::Integer(rhs))) => {
          program.push_expr(Expr::Integer(lhs * rhs))?;
          Ok(())
        }
        Some((Expr::Float(lhs), Expr::Float(rhs))) => {
          program.push_expr(Expr::Float(lhs * rhs))?;
          Ok(())
        }
        _ => Err(EvalError {
          expr: trace_expr,
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::List(vec![
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
            ]),
            Type::List(vec![
              lhs.type_of(&program.ast),
              rhs.type_of(&program.ast)
            ]),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("/"),
    |program, trace_expr| {
      let rhs = program.pop_expr(trace_expr)?;
      let lhs = program.pop_expr(trace_expr)?;

      match lhs.coerce_same_float(&rhs) {
        Some((Expr::Integer(lhs), Expr::Integer(rhs))) => {
          program.push_expr(Expr::Integer(lhs / rhs))?;
          Ok(())
        }
        Some((Expr::Float(lhs), Expr::Float(rhs))) => {
          program.push_expr(Expr::Float(lhs / rhs))?;
          Ok(())
        }
        _ => Err(EvalError {
          expr: trace_expr,
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::List(vec![
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
            ]),
            Type::List(vec![
              lhs.type_of(&program.ast),
              rhs.type_of(&program.ast)
            ]),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("%"),
    |program, trace_expr| {
      let rhs = program.pop_expr(trace_expr)?;
      let lhs = program.pop_expr(trace_expr)?;

      match lhs.coerce_same_float(&rhs) {
        Some((Expr::Integer(lhs), Expr::Integer(rhs))) => {
          program.push_expr(Expr::Integer(lhs % rhs))?;
          Ok(())
        }
        Some((Expr::Float(lhs), Expr::Float(rhs))) => {
          program.push_expr(Expr::Float(lhs % rhs))?;
          Ok(())
        }
        _ => Err(EvalError {
          expr: trace_expr,
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::List(vec![
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
              Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
            ]),
            Type::List(vec![
              lhs.type_of(&program.ast),
              rhs.type_of(&program.ast)
            ]),
          ),
        }),
      }
    },
  );

  Ok(())
}
