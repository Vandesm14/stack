use crate::{interner::interner, EvalError, Expr, Program};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program.funcs.insert(
    interner().get_or_intern_static("toboolean"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item {
        Expr::String(string) => {
          let string_str = interner().resolve(&string);

          program.push(
            string_str
              .parse()
              .ok()
              .map(Expr::Boolean)
              .unwrap_or(Expr::Nil),
          )
        }
        found => program.push(found.to_boolean().unwrap_or(Expr::Nil)),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("tointeger"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item {
        Expr::String(string) => {
          let string_str = interner().resolve(&string);

          program.push(
            string_str
              .parse()
              .ok()
              .map(Expr::Integer)
              .unwrap_or(Expr::Nil),
          )
        }
        found => program.push(found.to_integer().unwrap_or(Expr::Nil)),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("tofloat"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item {
        Expr::String(string) => {
          let string_str = interner().resolve(&string);

          program.push(
            string_str
              .parse()
              .ok()
              .map(Expr::Float)
              .unwrap_or(Expr::Nil),
          )
        }
        found => program.push(found.to_float().unwrap_or(Expr::Nil)),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("tostring"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item {
        string @ Expr::String(_) => program.push(string),
        found => {
          let string =
            Expr::String(interner().get_or_intern(found.to_string()));
          program.push(string)
        }
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("tolist"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item {
        list @ Expr::List(_) => program.push(list),
        Expr::String(s) => {
          let str = interner().resolve(&s).to_owned();
          program.push(Expr::List(
            str
              .chars()
              .map(|c| Expr::String(interner().get_or_intern(c.to_string())))
              .collect::<Vec<_>>(),
          ))
        }
        found => program.push(Expr::List(vec![found])),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("tocall"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item {
        call @ Expr::Call(_) => program.push(call),
        Expr::String(string) => program.push(Expr::Call(string)),
        found => {
          let call = Expr::Call(interner().get_or_intern(found.to_string()));
          program.push(call)
        }
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("typeof"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;
      let string =
        Expr::String(interner().get_or_intern(item.type_of().to_string()));
      program.push(string)
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
      vec![Expr::String(interner().get_or_intern_static("1"))]
    );
  }

  #[test]
  fn to_call() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("\"a\" tocall").unwrap();
    assert_eq!(
      program.stack,
      vec![Expr::Call(interner().get_or_intern_static("a"))]
    );
  }

  #[test]
  fn to_integer() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("\"1\" tointeger").unwrap();
    assert_eq!(program.stack, vec![Expr::Integer(1)]);
  }

  #[test]
  fn type_of() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 typeof").unwrap();
    assert_eq!(
      program.stack,
      vec![Expr::String(interner().get_or_intern_static("integer"))]
    );
  }

  #[test]
  fn list_to_list() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("(1 2 3) tolist").unwrap();
    assert_eq!(
      program.stack,
      vec![Expr::List(vec![
        Expr::Integer(1),
        Expr::Integer(2),
        Expr::Integer(3)
      ])]
    );
  }

  #[test]
  fn list_into_lazy() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("(1 2 3) lazy").unwrap();
    assert_eq!(
      program.stack,
      vec![Expr::Lazy(
        Expr::List(vec![Expr::Integer(1), Expr::Integer(2), Expr::Integer(3)])
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
      vec![Expr::Lazy(
        Expr::Call(interner().get_or_intern_static("set")).into()
      )]
    );
  }
}
