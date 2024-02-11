use crate::{interner::interner, EvalError, Expr, Program, Type};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program.funcs.insert(
    interner().get_or_intern_static("def"),
    |program, trace_expr| {
      let key = program.pop(trace_expr)?;
      let val = program.pop(trace_expr)?;

      match key {
        Expr::Call(ref key) => match program.funcs.contains_key(key) {
          true => Err(EvalError {
            expr: trace_expr.clone(),
            program: program.clone(),
            message: format!(
              "cannot shadow a native function {}",
              interner().resolve(key)
            ),
          }),
          false => {
            program.def_scope_item(trace_expr, interner().resolve(key), val)
          }
        },
        key => Err(EvalError {
          expr: trace_expr.clone(),
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::List(vec![
              // TODO: A type to represent functions.
              Type::Any,
              Type::Call,
            ]),
            Type::List(vec![val.type_of(), key.type_of(),]),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("undef"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item {
        Expr::Call(key) => {
          let key_str = interner().resolve(&key).to_owned();
          program.remove_scope_item(&key_str);

          Ok(())
        }
        item => Err(EvalError {
          expr: trace_expr.clone(),
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::Call,
            item.type_of(),
          ),
        }),
      }
    }
  );

  program.funcs.insert(
    interner().get_or_intern_static("set"),
    |program, trace_expr| {
      let key = program.pop(trace_expr)?;
      let val = program.pop(trace_expr)?;

      match key {
        Expr::Call(ref key) => match program.funcs.contains_key(key) {
          true => Err(EvalError {
            expr: trace_expr.clone(),
            program: program.clone(),
            message: format!(
              "cannot shadow a native function {}",
              interner().resolve(key)
            ),
          }),
          false => {
            program.set_scope_item(trace_expr, interner().resolve(key), val)
          }
        },
        key => Err(EvalError {
          expr: trace_expr.clone(),
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::List(vec![
              // TODO: A type to represent functions.
              Type::Any,
              Type::Call,
            ]),
            Type::List(vec![val.type_of(), key.type_of(),]),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("get"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item {
        Expr::Call(ref key) => {
          if let Some(func) = program.funcs.get(key) {
            func(program, trace_expr)?;
          } else {
            let key_str = interner().resolve(key);

            // Always push something, otherwise it can get tricky to manage the
            // stack in-langauge.
            program.push(program.scope_item(key_str).unwrap_or(Expr::Nil));
          }

          Ok(())
        }
        item => Err(EvalError {
          expr: trace_expr.clone(),
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::Call,
            item.type_of(),
          ),
        }),
      }
    }
  );

  Ok(())
}
