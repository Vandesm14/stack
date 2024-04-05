use std::iter;

use crate::{interner::interner, Ast, EvalError, Expr, Program, Type};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program.funcs.insert(
    interner().get_or_intern_static("len"),
    |program, trace_expr| {
      let (list, list_index) = program.pop_with_index(trace_expr)?;

      match list {
        Expr::List(list) => match i64::try_from(list.len()) {
          Ok(i) => {
            program.push(list_index)?;
            program.push_expr(Expr::Integer(i))?;
            Ok(())
          }
          Err(_) => {
            program.push_expr(Expr::Nil)?;
            Ok(())
          }
        },
        _ => {
          program.push_expr(Expr::Nil)?;
          Ok(())
        }
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("index"),
    |program, trace_expr| {
      let index = program.pop_expr(trace_expr)?;
      let (list, list_index) = program.pop_with_index(trace_expr)?;

      match index {
        Expr::Integer(index) => match usize::try_from(index) {
          Ok(i) => match list {
            Expr::List(list) => {
              program.push(list_index)?;
              program.push(list.get(i).cloned().unwrap_or(Ast::NIL))
            }
            _ => {
              program.push_expr(Expr::Nil)?;
              Ok(())
            }
          },
          Err(_) => {
            program.push_expr(Expr::Nil)?;
            Ok(())
          }
        },
        _ => {
          program.push_expr(Expr::Nil)?;
          Ok(())
        }
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("split"),
    |program, trace_expr| {
      let index = program.pop_expr(trace_expr)?;
      let list = program.pop_expr(trace_expr)?;

      match index {
        Expr::Integer(index) => match usize::try_from(index) {
          Ok(i) => match list {
            Expr::List(mut list) => {
              if i <= list.len() {
                let rest = list.split_off(i);
                program.push_expr(Expr::List(list))?;
                program.push_expr(Expr::List(rest))?;
                Ok(())
              } else {
                program.push_expr(Expr::Nil)?;
                Ok(())
              }
            }
            _ => {
              program.push_expr(Expr::Nil)?;
              Ok(())
            }
          },
          Err(_) => {
            program.push_expr(Expr::Nil)?;
            Ok(())
          }
        },
        _ => {
          program.push_expr(Expr::Nil)?;
          Ok(())
        }
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("join"),
    |program, trace_expr| {
      let delimiter = program.pop_expr(trace_expr)?;
      let list = program.pop_expr(trace_expr)?;

      match (delimiter, list) {
        (Expr::String(delimiter), Expr::List(list)) => {
          let delimiter_str = interner().resolve(&delimiter);

          let string = list
            .into_iter()
            .map(|expr| match program.ast_expr(trace_expr, expr) {
              Ok(Expr::String(string)) => {
                Ok(interner().resolve(string).to_string())
              }
              Ok(expr) => Ok(expr.to_string()),
              Err(err) => Err(err),
            })
            .enumerate()
            .try_fold(String::new(), |mut string, (i, result)| {
              if i > 0 {
                string.push_str(delimiter_str);
              }

              match result {
                Ok(chunk) => string.push_str(&chunk),
                Err(err) => return Err(err),
              }

              Ok(string)
            })?;
          let string = Expr::String(interner().get_or_intern(string));
          program.push_expr(string)?;
          Ok(())
        }
        (delimiter, list) => Err(EvalError {
          expr: trace_expr,
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::List(vec![Type::List(vec![]), Type::String]),
            Type::List(vec![
              list.type_of(&program.ast),
              delimiter.type_of(&program.ast)
            ]),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("concat"),
    |program, trace_expr| {
      let list_rhs = program.pop_expr(trace_expr)?;
      let list_lhs = program.pop_expr(trace_expr)?;

      match (list_lhs, list_rhs) {
        (Expr::List(mut list_lhs), Expr::List(list_rhs)) => {
          list_lhs.extend(list_rhs);
          program.push_expr(Expr::List(list_lhs))?;
          Ok(())
        }
        _ => {
          program.push_expr(Expr::Nil)?;
          Ok(())
        }
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("unwrap"),
    |program, trace_expr| {
      let (list, index) = program.pop_with_index(trace_expr)?;

      match list {
        Expr::List(list) => {
          program.stack.extend(list);
          Ok(())
        }
        _ => {
          program.push(index)?;
          Ok(())
        }
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("wrap"),
    |program, trace_expr| {
      let any = program.pop(trace_expr)?;
      program.push_expr(Expr::List(vec![any]))?;
      Ok(())
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("call-list"),
    |program, trace_expr| {
      let item = program.pop_expr(trace_expr)?;

      match item.clone() {
        Expr::List(list) => {
          let stack_len = program.stack.len();
          let list = program.ast.expr_many(list);

          program.eval(list)?;

          let list_len = program.stack.len() - stack_len;

          let mut list = iter::repeat_with(|| program.pop(trace_expr).unwrap())
            .take(list_len)
            .collect::<Vec<_>>();
          list.reverse();

          program.push_expr(Expr::List(list))?;
          Ok(())
        }
        _ => Err(EvalError {
          expr: trace_expr,
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::List(vec![Type::FnScope, Type::Any]),
            item.type_of(&program.ast),
          ),
        }),
      }
    },
  );

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::ExprTree;

  #[test]
  fn concatenating_lists() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("(1 2) (3 \"4\") concat").unwrap();
    assert_eq!(
      program.stack_exprs(),
      vec![ExprTree::List(vec![
        ExprTree::Integer(1),
        ExprTree::Integer(2),
        ExprTree::Integer(3),
        ExprTree::String(interner().get_or_intern_static("4"))
      ])]
    );
  }

  #[test]
  fn concatenating_blocks() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("(1 2) ('+) concat").unwrap();
    assert_eq!(
      program.stack_exprs(),
      vec![ExprTree::List(vec![
        ExprTree::Integer(1),
        ExprTree::Integer(2),
        ExprTree::Call(interner().get_or_intern_static("+"))
      ])]
    );
  }

  #[test]
  fn getting_length_of_list() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("(1 2 3) len").unwrap();
    assert_eq!(
      program.stack_exprs(),
      vec![
        ExprTree::List(vec![
          ExprTree::Integer(1),
          ExprTree::Integer(2),
          ExprTree::Integer(3)
        ]),
        ExprTree::Integer(3)
      ]
    );
  }

  #[test]
  fn getting_indexed_item_of_list() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("(1 2 3) 1 index").unwrap();
    assert_eq!(
      program.stack_exprs(),
      vec![
        ExprTree::List(vec![
          ExprTree::Integer(1),
          ExprTree::Integer(2),
          ExprTree::Integer(3)
        ]),
        ExprTree::Integer(2)
      ]
    );
  }

  #[test]
  fn calling_lists() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("'(2 2 +) call").unwrap();
    assert_eq!(program.stack_exprs(), vec![ExprTree::Integer(4)]);
  }

  #[test]
  fn calling_lists_special() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("'(2 2 +) call-list").unwrap();
    assert_eq!(
      program.stack_exprs(),
      vec![ExprTree::List(vec![ExprTree::Integer(4)])]
    );
  }

  mod wrap {
    use super::*;
    use crate::{ExprTree, Program};

    #[test]
    fn wrap_integer() {
      let mut program = Program::new().with_core().unwrap();
      program.eval_string("42 wrap").unwrap();
      assert_eq!(
        program.stack_exprs(),
        vec![ExprTree::List(vec![ExprTree::Integer(42)])]
      );
    }

    #[test]
    fn wrap_string() {
      let mut program = Program::new().with_core().unwrap();
      program.eval_string("\"hello\" wrap").unwrap();
      assert_eq!(
        program.stack_exprs(),
        vec![ExprTree::List(vec![ExprTree::String(
          interner().get_or_intern_static("hello")
        )])]
      );
    }

    #[test]
    fn wrap_list() {
      let mut program = Program::new().with_core().unwrap();
      program.eval_string("(1 2 3) wrap").unwrap();
      assert_eq!(
        program.stack_exprs(),
        vec![ExprTree::List(vec![ExprTree::List(vec![
          ExprTree::Integer(1),
          ExprTree::Integer(2),
          ExprTree::Integer(3)
        ])])]
      );
    }

    #[test]
    fn wrap_boolean() {
      let mut program = Program::new().with_core().unwrap();
      program.eval_string("true wrap").unwrap();
      assert_eq!(
        program.stack_exprs(),
        vec![ExprTree::List(vec![ExprTree::Boolean(true)])]
      );
    }
  }
}
