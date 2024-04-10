use internment::Intern;

use crate::{DebugData, EvalError, Expr, ExprKind, Program};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program
    .funcs
    .insert(Intern::from_ref("toboolean"), |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item.val {
        ExprKind::Boolean(_) => program.push(item),
        ExprKind::String(string) => program.push(Expr {
          val: string
            .parse()
            .ok()
            .map(ExprKind::Boolean)
            .unwrap_or(ExprKind::Nil),
          debug_data: DebugData::default(),
        }),
        found => program.push(Expr {
          val: found.to_boolean().unwrap_or(ExprKind::Nil),
          debug_data: DebugData::default(),
        }),
      }
    });

  program
    .funcs
    .insert(Intern::from_ref("tointeger"), |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item.val {
        ExprKind::Integer(_) => program.push(item),
        ExprKind::String(string) => program.push(
          string
            .parse()
            .ok()
            .map(ExprKind::Integer)
            .unwrap_or(ExprKind::Nil)
            .into_expr(DebugData::default()),
        ),
        found => program.push(
          found
            .to_integer()
            .unwrap_or(ExprKind::Nil)
            .into_expr(item.debug_data),
        ),
      }
    });

  program
    .funcs
    .insert(Intern::from_ref("tofloat"), |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item.val {
        ExprKind::Float(_) => program.push(item),
        ExprKind::String(string) => program.push(
          string
            .parse()
            .ok()
            .map(ExprKind::Float)
            .unwrap_or(ExprKind::Nil)
            .into_expr(DebugData::default()),
        ),
        found => program.push(
          found
            .to_float()
            .unwrap_or(ExprKind::Nil)
            .into_expr(DebugData::default()),
        ),
      }
    });

  program
    .funcs
    .insert(Intern::from_ref("tostring"), |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item.val {
        ExprKind::String(_) => program.push(item),
        found => {
          let string = ExprKind::String(found.to_string());
          program.push(string.into_expr(DebugData::default()))
        }
      }
    });

  program
    .funcs
    .insert(Intern::from_ref("tolist"), |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item.val {
        ExprKind::List(_) => program.push(item),
        ExprKind::String(str) => program.push(
          ExprKind::List(
            str
              .chars()
              .map(|c| {
                ExprKind::String(c.to_string()).into_expr(DebugData::default())
              })
              .collect::<Vec<_>>(),
          )
          .into_expr(DebugData::default()),
        ),
        found => program.push(
          ExprKind::List(vec![found.into_expr(item.debug_data)])
            .into_expr(DebugData::default()),
        ),
      }
    });

  program
    .funcs
    .insert(Intern::from_ref("tocall"), |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item.val {
        ExprKind::Call(_) => program.push(item),
        ExprKind::String(string) => program.push(
          ExprKind::Call(Intern::new(string)).into_expr(DebugData::default()),
        ),
        found => {
          let call = ExprKind::Call(Intern::new(found.to_string()));
          program.push(call.into_expr(DebugData::default()))
        }
      }
    });

  program
    .funcs
    .insert(Intern::from_ref("typeof"), |program, trace_expr| {
      let item = program.pop(trace_expr)?;
      let string = ExprKind::String(item.val.type_of().to_string());
      program.push(string.into_expr(DebugData::default()))
    });

  Ok(())
}

#[cfg(test)]
mod tests {
  use crate::{simple_exprs, TestExpr};

  use super::*;

  #[test]
  fn to_string() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 tostring").unwrap();
    assert_eq!(
      simple_exprs(program.stack),
      vec![TestExpr::String("1".into())]
    );
  }

  #[test]
  fn to_call() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("\"a\" tocall").unwrap();
    assert_eq!(
      simple_exprs(program.stack),
      vec![TestExpr::Call(Intern::from_ref("a"))]
    );
  }

  #[test]
  fn to_integer() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("\"1\" tointeger").unwrap();
    assert_eq!(simple_exprs(program.stack), vec![TestExpr::Integer(1)]);
  }

  #[test]
  fn type_of() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 typeof").unwrap();
    assert_eq!(
      simple_exprs(program.stack),
      vec![TestExpr::String("integer".into())]
    );
  }

  #[test]
  fn list_to_list() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("(1 2 3) tolist").unwrap();
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
  fn list_into_lazy() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("(1 2 3) lazy").unwrap();
    assert_eq!(
      simple_exprs(program.stack),
      vec![TestExpr::Lazy(
        TestExpr::List(vec![
          TestExpr::Integer(1),
          TestExpr::Integer(2),
          TestExpr::Integer(3)
        ])
        .into()
      )]
    );
  }

  #[test]
  fn call_into_lazy() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("'set lazy").unwrap();
    assert_eq!(
      simple_exprs(program.stack),
      vec![TestExpr::Lazy(
        TestExpr::Call(Intern::from_ref("set")).into()
      )]
    );
  }
}
