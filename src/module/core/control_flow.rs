use crate::{interner::interner, EvalError, Expr, Program, Type};

pub fn module(program: &mut Program) {
  program.funcs.insert(
    interner().get_or_intern_static("ifelse"),
    |program, trace_expr| {
      let cond = program.pop(trace_expr)?;
      let then = program.pop(trace_expr)?;
      let r#else = program.pop(trace_expr)?;

      match (cond, then, r#else) {
        (Expr::List(cond), Expr::List(then), Expr::List(r#else)) => {
          program.eval(cond)?;
          let cond = program.pop(trace_expr)?;

          if cond.is_truthy() {
            program.eval(then)
          } else {
            program.eval(r#else)
          }
        }
        (cond, then, r#else) => Err(EvalError {
          expr: trace_expr.clone(),
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::List(vec![
              // TODO: A type to represent functions.
              Type::List(vec![Type::Boolean]),
              Type::List(vec![]),
              Type::List(vec![]),
            ]),
            Type::List(vec![cond.type_of(), then.type_of(), r#else.type_of(),]),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("if"),
    |program, trace_expr| {
      let cond = program.pop(trace_expr)?;
      let then = program.pop(trace_expr)?;

      match (cond, then) {
        (Expr::List(cond), Expr::List(then)) => {
          program.eval(cond)?;
          let cond = program.pop(trace_expr)?;

          if cond.is_truthy() {
            program.eval(then)
          } else {
            Ok(())
          }
        }
        (cond, then) => Err(EvalError {
          expr: trace_expr.clone(),
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::List(vec![
              // TODO: A type to represent functions.
              Type::List(vec![Type::Boolean]),
              Type::List(vec![]),
            ]),
            Type::List(vec![cond.type_of(), then.type_of(),]),
          ),
        }),
      }

      // program.push(Expr::List(vec![]));
      // program.eval_intrinsic(trace_expr, Intrinsic::IfElse)
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("while"),
    |program, trace_expr| {
      let cond = program.pop(trace_expr)?;
      let block = program.pop(trace_expr)?;

      match (cond, block) {
        (Expr::List(cond), Expr::List(block)) => loop {
          program.eval(cond.clone())?;
          let cond = program.pop(trace_expr)?;

          if cond.is_truthy() {
            program.eval(block.clone())?;
          } else {
            break Ok(());
          }
        },
        (cond, block) => Err(EvalError {
          expr: trace_expr.clone(),
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::List(vec![
              // TODO: A type to represent functions.
              Type::List(vec![Type::Boolean]),
              Type::List(vec![]),
            ]),
            Type::List(vec![cond.type_of(), block.type_of(),]),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("halt"),
    |program, trace_expr| {
      Err(EvalError {
        expr: trace_expr.clone(),
        program: program.clone(),
        message: "halt".to_string(),
      })
    },
  );
}
