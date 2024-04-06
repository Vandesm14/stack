use crate::{interner::interner, EvalError, Expr, ExprKind, Program, Type};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program.funcs.insert(
    interner().get_or_intern_static("collect"),
    |program, _| {
      let list = core::mem::take(&mut program.stack);
      program.push(ExprKind::List(list).into_expr())
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("clear"),
    |program, _| {
      program.stack.clear();
      Ok(())
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("drop"),
    |program, _| {
      program.stack.pop();
      Ok(())
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("dup"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      program.push(item.clone())?;
      program.push(item)
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("swap"),
    |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      program.push(rhs)?;
      program.push(lhs)
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("rot"),
    |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let mid = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      program.push(rhs)?;
      program.push(lhs)?;
      program.push(mid)
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("lazy"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;
      program.push(ExprKind::Lazy(Box::new(item)).into_expr())
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("call"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item.val {
        call @ ExprKind::Call(_) => program.eval_expr(call.into_expr()),
        // This is where auto-call is defined and functions are evaluated when
        // they are called via an identifier
        // TODO: Get this working again.
        item @ ExprKind::List(_) => match item.is_function() {
          true => {
            let fn_symbol = item.fn_symbol().unwrap();
            let fn_body = item.fn_body().unwrap();

            if fn_symbol.scoped {
              program.push_scope(fn_symbol.scope.clone());
            }

            match program.eval(fn_body.to_vec()) {
              Ok(_) => {
                if fn_symbol.scoped {
                  program.pop_scope();
                }
                Ok(())
              }
              Err(err) => Err(err),
            }
          }
          false => {
            let ExprKind::List(list) = item else {
              unreachable!()
            };
            program.eval(list)
          }
        },
        _ => Err(EvalError {
          expr: trace_expr.clone(),
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::Set(vec![
              Type::Call,
              Type::List(vec![Type::FnScope, Type::Any])
            ]),
            item.val.type_of(),
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

  #[test]
  fn clearing_stack() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 2 clear").unwrap();
    assert_eq!(program.stack, vec![]);
  }

  #[test]
  fn dropping_from_stack() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 2 drop").unwrap();
    assert_eq!(program.stack, vec![ExprKind::Integer(1)]);
  }

  #[test]
  fn duplicating() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 dup").unwrap();
    assert_eq!(
      program.stack,
      vec![ExprKind::Integer(1), ExprKind::Integer(1)]
    );
  }

  #[test]
  fn swapping() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 2 swap").unwrap();
    assert_eq!(
      program.stack,
      vec![ExprKind::Integer(2), ExprKind::Integer(1)]
    );
  }

  #[test]
  fn rotating() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 2 3 rot").unwrap();
    assert_eq!(
      program.stack,
      vec![
        ExprKind::Integer(3),
        ExprKind::Integer(1),
        ExprKind::Integer(2)
      ]
    );
  }

  #[test]
  fn collect() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 2 3 collect").unwrap();
    assert_eq!(
      program.stack,
      vec![ExprKind::List(vec![
        ExprKind::Integer(1),
        ExprKind::Integer(2),
        ExprKind::Integer(3)
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
      program.stack,
      vec![
        ExprKind::Integer(1),
        ExprKind::Integer(2),
        ExprKind::Integer(3)
      ]
    );

    let a = program
      .scopes
      .last()
      .unwrap()
      .get_val(interner().get_or_intern("a"))
      .unwrap();

    assert_eq!(
      a,
      ExprKind::List(vec![
        ExprKind::Integer(1),
        ExprKind::Integer(2),
        ExprKind::Integer(3)
      ])
    );
  }
}
