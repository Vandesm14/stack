use crate::{interner::interner, EvalError, Expr, Program, Type};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program.funcs.insert(
    interner().get_or_intern_static("ifelse"),
    |program, trace_expr| {
      let (cond, cond_index) = program.pop_with_index(trace_expr)?;
      let (then, then_index) = program.pop_with_index(trace_expr)?;
      let (r#else, else_index) = program.pop_with_index(trace_expr)?;

      match (cond, then, r#else) {
        (Expr::List(cond), Expr::List(then), Expr::List(r#else)) => {
          // FIXME: Cloning the vec probably isn't the best option here.
          program.eval(program.ast.expr_many(cond.clone()))?;
          let cond = program.pop_expr(trace_expr)?;

          // if cond.is_truthy() {
          if matches!(program.ast.is_truthy(cond_index), Some(true)) {
            // FIXME: Cloning the vec probably isn't the best option here.
            program.eval(program.ast.expr_many(then.clone()))
          } else {
            // FIXME: Cloning the vec probably isn't the best option here.
            program.eval(program.ast.expr_many(r#else.clone()))
          }
        }
        (cond, then, r#else) => Err(EvalError {
          expr: trace_expr.clone(),
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::List(vec![
              Type::List(vec![Type::Boolean]),
              Type::List(vec![]),
              Type::List(vec![]),
            ]),
            Type::List(vec![
              // FIXME: Maybe unwrapping the type_of call isn't great, but this should be fine?
              // TODO: refactor the AST stuff to hopefully remove the fact that EVERYTHING is always an option
              cond.type_of(&program.ast),
              then.type_of(&program.ast),
              r#else.type_of(&program.ast)
            ]),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("if"),
    |program, trace_expr| {
      let (cond, cond_index) = program.pop_with_index(trace_expr)?;
      let (then, then_index) = program.pop_with_index(trace_expr)?;

      match (cond, then) {
        (Expr::List(cond), Expr::List(then)) => {
          program.eval(program.ast.expr_many(cond.clone()))?;
          let cond = program.pop_expr(trace_expr)?;

          if matches!(program.ast.is_truthy(cond_index), Some(true)) {
            program.eval(program.ast.expr_many(then.clone()))
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
            Type::List(vec![
              cond.type_of(&program.ast),
              then.type_of(&program.ast),
            ]),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("while"),
    |program, trace_expr| {
      let (cond, cond_index) = program.pop_with_index(trace_expr)?;
      let (block, block_index) = program.pop_with_index(trace_expr)?;

      match (cond, block) {
        (Expr::List(cond), Expr::List(block)) => loop {
          program.eval(program.ast.expr_many(cond.clone()))?;
          let cond = program.pop_expr(trace_expr)?;

          if matches!(program.ast.is_truthy(cond_index), Some(true)) {
            program.eval(program.ast.expr_many(block.clone()))?;
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
              Type::List(vec![Type::Boolean]),
              Type::List(vec![]),
            ]),
            Type::List(vec![
              cond.type_of(&program.ast),
              block.type_of(&program.ast),
            ]),
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
