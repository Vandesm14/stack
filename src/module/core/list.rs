use std::iter;

use itertools::Itertools;

use crate::{
  interner::interner, DebugData, EvalError, EvalErrorKind, ExprKind, Program,
  Type,
};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program.funcs.insert(
    interner().get_or_intern_static("len"),
    |program, trace_expr| {
      let list_expr = program.pop(trace_expr)?;
      program.push(list_expr.clone())?;

      match list_expr.val {
        ExprKind::List(list) => match i64::try_from(list.len()) {
          Ok(i) => {
            program.push(ExprKind::Integer(i).into_expr(DebugData::default()))
          }
          Err(_) => {
            todo!("Create a list type to not exceed the i64 bounds")
          }
        },
        _ => Err(EvalError {
          expr: Some(trace_expr.clone()),
          kind: EvalErrorKind::ExpectedFound(
            Type::List(vec![]),
            list_expr.val.type_of(),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("index"),
    |program, trace_expr| {
      let index_expr = program.pop(trace_expr)?;
      let list_expr = program.pop(trace_expr)?;
      program.push(list_expr.clone())?;

      match index_expr.val {
        ExprKind::Integer(index) => match usize::try_from(index) {
          Ok(i) => match list_expr.val {
            ExprKind::List(list) => program.push(
              list
                .get(i)
                .cloned()
                .unwrap_or(ExprKind::Nil.into_expr(DebugData::default())),
            ),
            _ => Err(EvalError {
              expr: Some(trace_expr.clone()),
              kind: EvalErrorKind::ExpectedFound(
                Type::List(vec![Type::List(vec![]), Type::Integer]),
                Type::List(vec![
                  list_expr.val.type_of(),
                  index_expr.val.type_of(),
                ]),
              ),
            }),
          },
          Err(_) => Err(EvalError {
            kind: EvalErrorKind::Message(
              "could not convert index into integer".into(),
            ),
            expr: Some(trace_expr.clone()),
          }),
        },
        _ => Err(EvalError {
          expr: Some(trace_expr.clone()),
          kind: EvalErrorKind::ExpectedFound(
            Type::List(vec![Type::List(vec![]), Type::Integer]),
            Type::List(vec![list_expr.val.type_of(), index_expr.val.type_of()]),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("split"),
    |program, trace_expr| {
      let index_expr = program.pop(trace_expr)?;
      let list_expr = program.pop(trace_expr)?;

      match index_expr.val {
        ExprKind::Integer(index) if index >= 0 => {
          let i = index as usize;
          match list_expr.val {
            ExprKind::List(mut list) => {
              if i <= list.len() {
                let rest = list.split_off(i);
                program
                  .push(ExprKind::List(list).into_expr(DebugData::default()))?;
                program
                  .push(ExprKind::List(rest).into_expr(DebugData::default()))
              } else {
                program.push(ExprKind::Nil.into_expr(DebugData::default()))
              }
            }
            _ => Err(EvalError {
              expr: Some(trace_expr.clone()),
              kind: EvalErrorKind::ExpectedFound(
                Type::List(vec![Type::List(vec![]), Type::Integer]),
                Type::List(vec![
                  list_expr.val.type_of(),
                  index_expr.val.type_of(),
                ]),
              ),
            }),
          }
        }
        _ => Err(EvalError {
          expr: Some(trace_expr.clone()),
          kind: EvalErrorKind::ExpectedFound(
            Type::List(vec![Type::List(vec![]), Type::Integer]),
            Type::List(vec![list_expr.val.type_of(), index_expr.val.type_of()]),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("join"),
    |program, trace_expr| {
      let delimiter_expr = program.pop(trace_expr)?;
      let list_expr = program.pop(trace_expr)?;

      match (delimiter_expr.val, list_expr.val) {
        (ExprKind::String(delimiter), ExprKind::List(list)) => {
          let delimiter_str = interner().resolve(&delimiter);

          let string = list
            .into_iter()
            .map(|expr| match expr.val {
              ExprKind::String(string) => {
                interner().resolve(&string).to_string()
              }
              expr_kind => expr_kind.to_string(),
            })
            .join(delimiter_str);
          let string = ExprKind::String(interner().get_or_intern(string));
          program.push(string.into_expr(DebugData::default()))
        }
        (delimiter, list) => Err(EvalError {
          expr: Some(trace_expr.clone()),
          kind: EvalErrorKind::ExpectedFound(
            Type::List(vec![Type::List(vec![]), Type::String]),
            Type::List(vec![list.type_of(), delimiter.type_of()]),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("concat"),
    |program, trace_expr| {
      let list_rhs_expr = program.pop(trace_expr)?;
      let list_lhs_expr = program.pop(trace_expr)?;

      match (list_lhs_expr.val, list_rhs_expr.val) {
        (ExprKind::List(mut list_lhs), ExprKind::List(list_rhs)) => {
          list_lhs.extend(list_rhs);
          let list_expr =
            ExprKind::List(list_lhs).into_expr(DebugData::default());
          program.push(list_expr)
        }
        (list_lhs, list_rhs) => Err(EvalError {
          expr: Some(trace_expr.clone()),
          kind: EvalErrorKind::ExpectedFound(
            Type::List(vec![Type::List(vec![]), Type::List(vec![])]),
            Type::List(vec![list_lhs.type_of(), list_rhs.type_of()]),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("unwrap"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item.val {
        ExprKind::List(list) => {
          program.stack.extend(list);
          Ok(())
        }
        _ => program.push(item),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("wrap"),
    |program, trace_expr| {
      let any = program.pop(trace_expr)?;
      program.push(ExprKind::List(vec![any]).into_expr(DebugData::default()))
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("call-list"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;
      let item_clone = item.clone();

      match item.val {
        ExprKind::List(list) => {
          let stack_len = program.stack.len();

          program.eval(list)?;

          let list_len = program.stack.len() - stack_len;

          let mut list = iter::from_fn(|| program.pop(&item_clone).ok())
            .take(list_len)
            .collect::<Vec<_>>();
          list.reverse();

          program.push(ExprKind::List(list).into_expr(DebugData::default()))
        }
        _ => Err(EvalError {
          expr: Some(trace_expr.clone()),
          kind: EvalErrorKind::ExpectedFound(
            Type::List(vec![Type::FnScope, Type::Any]),
            Type::List(vec![item.val.type_of()]),
          ),
        }),
      }
    },
  );

  Ok(())
}

#[cfg(test)]
mod tests {
  use crate::{simple_exprs, TestExpr};

  use super::*;

  #[test]
  fn concatenating_lists() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("(1 2) (3 \"4\") concat").unwrap();
    assert_eq!(
      simple_exprs(program.stack),
      vec![TestExpr::List(vec![
        TestExpr::Integer(1),
        TestExpr::Integer(2),
        TestExpr::Integer(3),
        TestExpr::String(interner().get_or_intern_static("4"))
      ])]
    );
  }

  #[test]
  fn concatenating_blocks() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("(1 2) ('+) concat").unwrap();
    assert_eq!(
      simple_exprs(program.stack),
      vec![TestExpr::List(vec![
        TestExpr::Integer(1),
        TestExpr::Integer(2),
        TestExpr::Call(interner().get_or_intern_static("+"))
      ])]
    );
  }

  #[test]
  fn getting_length_of_list() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("(1 2 3) len").unwrap();
    assert_eq!(
      simple_exprs(program.stack),
      vec![
        TestExpr::List(vec![
          TestExpr::Integer(1),
          TestExpr::Integer(2),
          TestExpr::Integer(3)
        ]),
        TestExpr::Integer(3)
      ]
    );
  }

  #[test]
  fn getting_indexed_item_of_list() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("(1 2 3) 1 index").unwrap();
    assert_eq!(
      simple_exprs(program.stack),
      vec![
        TestExpr::List(vec![
          TestExpr::Integer(1),
          TestExpr::Integer(2),
          TestExpr::Integer(3)
        ]),
        TestExpr::Integer(2)
      ]
    );
  }

  #[test]
  fn calling_lists() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("'(2 2 +) call").unwrap();
    assert_eq!(simple_exprs(program.stack), vec![TestExpr::Integer(4)]);
  }

  #[test]
  fn calling_lists_special() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("'(2 2 +) call-list").unwrap();
    assert_eq!(
      simple_exprs(program.stack),
      vec![TestExpr::List(vec![TestExpr::Integer(4)])]
    );
  }
}
