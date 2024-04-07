use crate::{
  interner::interner, DebugData, EvalError, Expr, ExprKind, Program,
};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program.funcs.insert(
    interner().get_or_intern_static("toboolean"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item.val {
        ExprKind::Boolean(_) => program.push(item),
        ExprKind::String(string) => {
          let string_str = interner().resolve(&string);

          program.push(Expr {
            val: string_str
              .parse()
              .ok()
              .map(ExprKind::Boolean)
              .unwrap_or(ExprKind::Nil),
            debug_data: DebugData::default(),
          })
        }
        found => program.push(Expr {
          val: found.to_boolean().unwrap_or(ExprKind::Nil),
          debug_data: DebugData::default(),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("tointeger"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item.val {
        ExprKind::Integer(_) => program.push(item),
        ExprKind::String(string) => {
          let string_str = interner().resolve(&string);

          program.push(
            string_str
              .parse()
              .ok()
              .map(ExprKind::Integer)
              .unwrap_or(ExprKind::Nil)
              .into_expr(DebugData::default()),
          )
        }
        found => program.push(
          found
            .to_integer()
            .unwrap_or(ExprKind::Nil)
            .into_expr(item.debug_data),
        ),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("tofloat"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item.val {
        ExprKind::Float(_) => program.push(item),
        ExprKind::String(string) => {
          let string_str = interner().resolve(&string);

          program.push(
            string_str
              .parse()
              .ok()
              .map(ExprKind::Float)
              .unwrap_or(ExprKind::Nil)
              .into_expr(DebugData::default()),
          )
        }
        found => program.push(
          found
            .to_float()
            .unwrap_or(ExprKind::Nil)
            .into_expr(DebugData::default()),
        ),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("tostring"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item.val {
        ExprKind::String(_) => program.push(item),
        found => {
          let string =
            ExprKind::String(interner().get_or_intern(found.to_string()));
          program.push(string.into_expr(DebugData::default()))
        }
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("tolist"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item.val {
        ExprKind::List(_) => program.push(item),
        ExprKind::String(s) => {
          let str = interner().resolve(&s).to_owned();
          program.push(
            ExprKind::List(
              str
                .chars()
                .map(|c| {
                  ExprKind::String(interner().get_or_intern(c.to_string()))
                    .into_expr(DebugData::default())
                })
                .collect::<Vec<_>>(),
            )
            .into_expr(DebugData::default()),
          )
        }
        found => program.push(
          ExprKind::List(vec![found.into_expr(item.debug_data)])
            .into_expr(DebugData::default()),
        ),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("tocall"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item.val {
        ExprKind::Call(_) => program.push(item),
        ExprKind::String(string) => {
          program.push(ExprKind::Call(string).into_expr(DebugData::default()))
        }
        found => {
          let call =
            ExprKind::Call(interner().get_or_intern(found.to_string()));
          program.push(call.into_expr(DebugData::default()))
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
      program.push(string.into_expr(DebugData::default()))
    },
  );

  Ok(())
}

#[cfg(test)]
mod tests {
  use crate::{simple_exprs, ExprSimple};

  use super::*;

  #[test]
  fn to_string() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 tostring").unwrap();
    assert_eq!(
      simple_exprs(program.stack),
      vec![ExprSimple::String(interner().get_or_intern_static("1"))]
    );
  }

  #[test]
  fn to_call() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("\"a\" tocall").unwrap();
    assert_eq!(
      simple_exprs(program.stack),
      vec![ExprSimple::Call(interner().get_or_intern_static("a"))]
    );
  }

  #[test]
  fn to_integer() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("\"1\" tointeger").unwrap();
    assert_eq!(simple_exprs(program.stack), vec![ExprSimple::Integer(1)]);
  }

  #[test]
  fn type_of() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 typeof").unwrap();
    assert_eq!(
      simple_exprs(program.stack),
      vec![ExprSimple::String(
        interner().get_or_intern_static("integer")
      )]
    );
  }

  #[test]
  fn list_to_list() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("(1 2 3) tolist").unwrap();
    assert_eq!(
      simple_exprs(program.stack),
      vec![ExprSimple::List(vec![
        ExprSimple::Integer(1),
        ExprSimple::Integer(2),
        ExprSimple::Integer(3)
      ])]
    );
  }

  #[test]
  fn list_into_lazy() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("(1 2 3) lazy").unwrap();
    assert_eq!(
      simple_exprs(program.stack),
      vec![ExprSimple::Lazy(
        ExprSimple::List(vec![
          ExprSimple::Integer(1),
          ExprSimple::Integer(2),
          ExprSimple::Integer(3)
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
      vec![ExprSimple::Lazy(
        ExprSimple::Call(interner().get_or_intern_static("set")).into()
      )]
    );
  }
}
