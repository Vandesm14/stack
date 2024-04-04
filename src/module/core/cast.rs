use crate::{interner::interner, EvalError, Expr, Program};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  // TODO: For primitives, if we are taking the item and placing it back on the stack (casting to the same type),
  // optimize by using program.push and program.pop so we're not creating a new entry in the AST
  program.funcs.insert(
    interner().get_or_intern_static("toboolean"),
    |program, trace_expr| {
      let item = program.pop_expr(trace_expr)?;

      match item {
        Expr::String(string) => {
          let string_str = interner().resolve(&string);

          program.push_expr(
            string_str
              .parse()
              .ok()
              .map(Expr::Boolean)
              .unwrap_or(Expr::Nil),
          )?;

          Ok(())
        }
        found => {
          program.push_expr(found.to_boolean().unwrap_or(Expr::Nil))?;

          Ok(())
        }
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("tointeger"),
    |program, trace_expr| {
      let item = program.pop_expr(trace_expr)?;

      match item {
        Expr::String(string) => {
          let string_str = interner().resolve(&string);

          program.push_expr(
            string_str
              .parse()
              .ok()
              .map(Expr::Integer)
              .unwrap_or(Expr::Nil),
          )?;

          Ok(())
        }
        found => {
          program.push_expr(found.to_integer().unwrap_or(Expr::Nil))?;

          Ok(())
        }
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("tofloat"),
    |program, trace_expr| {
      let item = program.pop_expr(trace_expr)?;

      match item {
        Expr::String(string) => {
          let string_str = interner().resolve(&string);

          program.push_expr(
            string_str
              .parse()
              .ok()
              .map(Expr::Float)
              .unwrap_or(Expr::Nil),
          )?;

          Ok(())
        }
        found => {
          program.push_expr(found.to_float().unwrap_or(Expr::Nil))?;

          Ok(())
        }
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("tostring"),
    |program, trace_expr| {
      let index = program.pop(trace_expr)?;
      let item = program.ast_expr(trace_expr, index)?;

      match item {
        string @ Expr::String(_) => {
          program.push(index)?;

          Ok(())
        }
        found => {
          let string =
            Expr::String(interner().get_or_intern(found.to_string()));
          program.push_expr(string)?;

          Ok(())
        }
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("tolist"),
    |program, trace_expr| {
      let index = program.pop(trace_expr)?;
      let item = program.ast_expr(trace_expr, index)?;

      match item {
        list @ Expr::List(_) => program.push(index),

        // TODO: reimplement (can't figure out how to handle errors for push_expr while within the map)
        // Expr::String(s) => {
        //   let str = interner().resolve(&s).to_owned();
        //   program.push_expr(Expr::List(
        //     str
        //       .chars()
        //       .map(|c| Expr::String(interner().get_or_intern(c.to_string())))
        //       .map(|expr| program.push_expr(expr))
        //       .collect::<Vec<_>>(),
        //   ))?;

        //   Ok(())
        // }
        found => {
          program.push_expr(Expr::List(vec![index]))?;

          Ok(())
        }
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("tocall"),
    |program, trace_expr| {
      let index = program.pop(trace_expr)?;
      let item = program.ast_expr(trace_expr, index)?;

      match item {
        call @ Expr::Call(_) => {
          program.push(index);
          Ok(())
        }
        Expr::String(string) => {
          program.push_expr(Expr::Call(*string))?;

          Ok(())
        }
        found => {
          let call = Expr::Call(interner().get_or_intern(found.to_string()));
          program.push_expr(call)?;

          Ok(())
        }
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("typeof"),
    |program, trace_expr| {
      // TODO: Reimplement. Need to either standardize errors created when ast functions such as type_of don't work,
      // or something else

      // let index = program.pop(trace_expr)?;
      // let item = program.ast_expr(trace_expr, index)?;

      // let string = Expr::String(
      //   interner().get_or_intern(program.ast.type_of(index).to_string()),
      // );
      // program.push_expr(string)?;

      Ok(())
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
