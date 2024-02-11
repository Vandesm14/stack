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
          );
        }
        found => program.push(found.to_boolean().unwrap_or(Expr::Nil)),
      }

      Ok(())
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
          );
        }
        found => program.push(found.to_integer().unwrap_or(Expr::Nil)),
      }

      Ok(())
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
          );
        }
        found => program.push(found.to_float().unwrap_or(Expr::Nil)),
      }

      Ok(())
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("tostring"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item {
        string @ Expr::String(_) => {
          program.push(string);
          Ok(())
        }
        found => {
          let string =
            Expr::String(interner().get_or_intern(found.to_string()));
          program.push(string);

          Ok(())
        }
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("tolist"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item {
        list @ Expr::List(_) => {
          program.push(list);
        }
        Expr::String(s) => {
          let str = interner().resolve(&s).to_owned();
          program.push(Expr::List(
            str
              .chars()
              .map(|c| Expr::String(interner().get_or_intern(c.to_string())))
              .collect::<Vec<_>>(),
          ));
        }
        found => {
          program.push(Expr::List(vec![found]));
        }
      }

      Ok(())
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("tocall"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item {
        call @ Expr::Call(_) => {
          program.push(call);
          Ok(())
        }
        Expr::String(string) => {
          program.push(Expr::Call(string));
          Ok(())
        }
        found => {
          let call = Expr::Call(interner().get_or_intern(found.to_string()));
          program.push(call);

          Ok(())
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
      program.push(string);

      Ok(())
    },
  );

  Ok(())
}
