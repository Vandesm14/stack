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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::ExprTree;

  #[test]
  fn test_or() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("true false or").unwrap();
    assert_eq!(program.stack_exprs(), vec![ExprTree::Boolean(true)]);
  }

  #[test]
  fn test_and_false() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("true false and").unwrap();
    assert_eq!(program.stack_exprs(), vec![ExprTree::Boolean(false)]);
  }

  #[test]
  fn test_and_true() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("true true and").unwrap();
    assert_eq!(program.stack_exprs(), vec![ExprTree::Boolean(true)]);
  }
}
