use internment::Intern;

use crate::{DebugData, EvalError, ExprKind, Program};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program
    .funcs
    .insert(Intern::from_ref("collect"), |program, _| {
      let list = core::mem::take(&mut program.stack);
      program.push(ExprKind::List(list).into_expr(DebugData::default()))
    });

  program
    .funcs
    .insert(Intern::from_ref("clear"), |program, _| {
      program.stack.clear();
      Ok(())
    });

  program
    .funcs
    .insert(Intern::from_ref("drop"), |program, trace_expr| {
      program.pop(trace_expr)?;
      Ok(())
    });

  program
    .funcs
    .insert(Intern::from_ref("dup"), |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      program.push(item.clone())?;
      program.push(item)
    });

  program
    .funcs
    .insert(Intern::from_ref("swap"), |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      program.push(rhs)?;
      program.push(lhs)
    });

  program
    .funcs
    .insert(Intern::from_ref("rot"), |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let mid = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      program.push(rhs)?;
      program.push(lhs)?;
      program.push(mid)
    });

  program
    .funcs
    .insert(Intern::from_ref("lazy"), |program, trace_expr| {
      let item = program.pop(trace_expr)?;
      program
        .push(ExprKind::Lazy(Box::new(item)).into_expr(DebugData::default()))
    });

  program
    .funcs
    .insert(Intern::from_ref("call"), |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      program.auto_call(trace_expr, item)
    });

  Ok(())
}

#[cfg(test)]

mod tests {
  use super::*;
  use crate::{simple_expr, simple_exprs, TestExpr};

  #[test]
  fn clearing_stack() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 2 clear").unwrap();
    assert_eq!(simple_exprs(program.stack), vec![]);
  }

  #[test]
  fn dropping_from_stack() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 2 drop").unwrap();
    assert_eq!(simple_exprs(program.stack), vec![TestExpr::Integer(1)]);
  }

  #[test]
  fn duplicating() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 dup").unwrap();
    assert_eq!(
      simple_exprs(program.stack),
      vec![TestExpr::Integer(1), TestExpr::Integer(1)]
    );
  }

  #[test]
  fn swapping() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 2 swap").unwrap();
    assert_eq!(
      simple_exprs(program.stack),
      vec![TestExpr::Integer(2), TestExpr::Integer(1)]
    );
  }

  #[test]
  fn rotating() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 2 3 rot").unwrap();
    assert_eq!(
      simple_exprs(program.stack),
      vec![
        TestExpr::Integer(3),
        TestExpr::Integer(1),
        TestExpr::Integer(2)
      ]
    );
  }

  #[test]
  fn collect() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 2 3 collect").unwrap();
    assert_eq!(
      simple_exprs(program.stack),
      vec![TestExpr::List(vec![
        TestExpr::Integer(1),
        TestExpr::Integer(2),
        TestExpr::Integer(3)
      ])]
    );
  }

  #[test]
  fn collect_and_unwrap() {
    let mut program = Program::new().with_core().unwrap();
    program
      .eval_string("1 2 3 collect 'a def 'a get unwrap")
      .unwrap();

    assert_eq!(
      simple_exprs(program.stack),
      vec![
        TestExpr::Integer(1),
        TestExpr::Integer(2),
        TestExpr::Integer(3)
      ]
    );

    let a = program
      .scopes
      .last()
      .unwrap()
      .get_val(Intern::from_ref("a"))
      .unwrap();

    assert_eq!(
      simple_expr(a),
      TestExpr::List(vec![
        TestExpr::Integer(1),
        TestExpr::Integer(2),
        TestExpr::Integer(3)
      ])
    );
  }
}
