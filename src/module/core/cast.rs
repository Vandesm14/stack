use crate::{interner::interner, EvalError, Expr, ExprKind, Program};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program.funcs.insert(
    interner().get_or_intern_static("toboolean"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item.val {
        ExprKind::String(string) => {
          let string_str = interner().resolve(&string);

          program.push(
            string_str
              .parse()
              .ok()
              .map(ExprKind::Boolean)
              .unwrap_or(ExprKind::Nil)
              .into_expr(),
          )
        }
        found => {
          program.push(found.to_boolean().unwrap_or(ExprKind::Nil).into_expr())
        }
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("tointeger"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item.val {
        ExprKind::String(string) => {
          let string_str = interner().resolve(&string);

          program.push(
            string_str
              .parse()
              .ok()
              .map(ExprKind::Integer)
              .unwrap_or(ExprKind::Nil)
              .into_expr(),
          )
        }
        found => {
          program.push(found.to_integer().unwrap_or(ExprKind::Nil).into_expr())
        }
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("tofloat"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item.val {
        ExprKind::String(string) => {
          let string_str = interner().resolve(&string);

          program.push(
            string_str
              .parse()
              .ok()
              .map(ExprKind::Float)
              .unwrap_or(ExprKind::Nil)
              .into_expr(),
          )
        }
        found => {
          program.push(found.to_float().unwrap_or(ExprKind::Nil).into_expr())
        }
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("tostring"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item.val {
        string @ ExprKind::String(_) => program.push(string.into_expr()),
        found => {
          let string =
            ExprKind::String(interner().get_or_intern(found.to_string()));
          program.push(string.into_expr())
        }
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("tolist"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item.val {
        list @ ExprKind::List(_) => program.push(list.into_expr()),
        ExprKind::String(s) => {
          let str = interner().resolve(&s).to_owned();
          program.push(
            ExprKind::List(
              str
                .chars()
                .map(|c| {
                  ExprKind::String(interner().get_or_intern(c.to_string()))
                    .into_expr()
                })
                .collect::<Vec<_>>(),
            )
            .into_expr(),
          )
        }
        found => {
          program.push(ExprKind::List(vec![found.into_expr()]).into_expr())
        }
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("tocall"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item.val {
        call @ ExprKind::Call(_) => program.push(call.into_expr()),
        ExprKind::String(string) => {
          program.push(ExprKind::Call(string).into_expr())
        }
        found => {
          let call =
            ExprKind::Call(interner().get_or_intern(found.to_string()));
          program.push(call.into_expr())
        }
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("typeof"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;
      let string = ExprKind::String(
        interner().get_or_intern(item.val.type_of().to_string()),
      );
      program.push(string.into_expr())
    },
  );

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn to_string() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 tostring").unwrap();
    assert_eq!(
      program.stack,
      vec![ExprKind::String(interner().get_or_intern_static("1"))]
    );
  }

  #[test]
  fn to_call() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("\"a\" tocall").unwrap();
    assert_eq!(
      program.stack,
      vec![ExprKind::Call(interner().get_or_intern_static("a"))]
    );
  }

  #[test]
  fn to_integer() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("\"1\" tointeger").unwrap();
    assert_eq!(program.stack, vec![ExprKind::Integer(1)]);
  }

  #[test]
  fn type_of() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 typeof").unwrap();
    assert_eq!(
      program.stack,
      vec![ExprKind::String(interner().get_or_intern_static("integer"))]
    );
  }

  #[test]
  fn list_to_list() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("(1 2 3) tolist").unwrap();
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
  fn list_into_lazy() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("(1 2 3) lazy").unwrap();
    assert_eq!(
      program.stack,
      vec![ExprKind::Lazy(
        ExprKind::List(vec![
          ExprKind::Integer(1),
          ExprKind::Integer(2),
          ExprKind::Integer(3)
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
      program.stack,
      vec![ExprKind::Lazy(
        ExprKind::Call(interner().get_or_intern_static("set")).into()
      )]
    );
  }
}
