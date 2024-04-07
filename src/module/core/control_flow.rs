use crate::{interner::interner, EvalError, ExprKind, Program, Type};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program.funcs.insert(
    interner().get_or_intern_static("ifelse"),
    |program, trace_expr| {
      let cond = program.pop(trace_expr)?;
      let then = program.pop(trace_expr)?;
      let r#else = program.pop(trace_expr)?;

      match (cond.val, then.val, r#else.val) {
        (
          ExprKind::List(cond),
          ExprKind::List(then),
          ExprKind::List(r#else),
        ) => {
          program.eval(cond)?;
          let cond = program.pop(trace_expr)?;

          if cond.val.is_truthy() {
            program.eval(then)
          } else {
            program.eval(r#else)
          }
        }
        (cond, then, r#else) => Err(EvalError {
          kind: crate::EvalErrorKind::ExpectedFound(
            Type::List(vec![
              Type::List(vec![Type::Boolean]),
              Type::List(vec![]),
              Type::List(vec![]),
            ]),
            Type::List(vec![cond.type_of(), then.type_of(), r#else.type_of()]),
          ),
          expr: Some(trace_expr),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("if"),
    |program, trace_expr| {
      let cond = program.pop(trace_expr)?;
      let then = program.pop(trace_expr)?;

      match (cond.val, then.val) {
        (ExprKind::List(cond), ExprKind::List(then)) => {
          program.eval(cond)?;
          let cond = program.pop(trace_expr)?;

          if cond.val.is_truthy() {
            program.eval(then)
          } else {
            Ok(())
          }
        }
        (cond, then) => Err(EvalError {
          kind: crate::EvalErrorKind::ExpectedFound(
            Type::List(vec![
              Type::List(vec![Type::Boolean]),
              Type::List(vec![]),
            ]),
            Type::List(vec![cond.type_of(), then.type_of()]),
          ),
          expr: Some(trace_expr),
        }),
      }

      // program.push(ExprKind::List(vec![]));
      // program.eval_intrinsic(trace_expr, Intrinsic::IfElse)
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("while"),
    |program, trace_expr| {
      let cond = program.pop(trace_expr)?;
      let block = program.pop(trace_expr)?;

      match (cond.val, block.val) {
        (ExprKind::List(cond), ExprKind::List(block)) => loop {
          program.eval(cond.clone())?;
          let cond = program.pop(trace_expr)?;

          if cond.val.is_truthy() {
            program.eval(block.clone())?;
          } else {
            break Ok(());
          }
        },
        (cond, block) => Err(EvalError {
          kind: crate::EvalErrorKind::ExpectedFound(
            Type::List(vec![
              Type::List(vec![Type::Boolean]),
              Type::List(vec![]),
            ]),
            Type::List(vec![cond.type_of(), block.type_of()]),
          ),
          expr: Some(trace_expr),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("halt"),
    |program, trace_expr| {
      Err(EvalError {
        kind: crate::EvalErrorKind::Halt,
        expr: Some(trace_expr),
      })
    },
  );

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  mod control_flow {
    use super::*;

    #[test]
    fn if_true() {
      let mut program = Program::new().with_core().unwrap();
      program
        .eval_string("1 2 + '(\"correct\") '(3 =) if")
        .unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::String(interner().get_or_intern_static("correct"))]
      );
    }

    #[test]
    fn if_empty_condition() {
      let mut program = Program::new().with_core().unwrap();
      program
        .eval_string("1 2 + 3 = '(\"correct\") '() if")
        .unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::String(interner().get_or_intern_static("correct"))]
      );
    }

    #[test]
    fn if_else_true() {
      let mut program = Program::new().with_core().unwrap();
      program
        .eval_string("1 2 + 3 = '(\"incorrect\") '(\"correct\") '() ifelse")
        .unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::String(interner().get_or_intern_static("correct"))]
      );
    }

    #[test]
    fn if_else_false() {
      let mut program = Program::new().with_core().unwrap();
      program
        .eval_string("1 2 + 2 = '(\"incorrect\") '(\"correct\") '() ifelse")
        .unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::String(interner().get_or_intern_static("incorrect"))]
      );
    }
  }

  mod loops {
    use super::*;

    #[test]
    fn while_loop() {
      let mut program = Program::new().with_core().unwrap();
      program
        .eval_string(
          ";; Set i to 3
           3 'i def

           '(
             ;; Decrement i by 1
             i 1 -
             ;; Set i
             'i set

             i
           ) '(
             ;; If i is 0, break
             i 0 !=
           ) while",
        )
        .unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::Integer(2), Expr::Integer(1), Expr::Integer(0)]
      );
    }
  }
}
