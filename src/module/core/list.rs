use itertools::Itertools as _;

use crate::{interner::interner, EvalError, Expr, Program, Type};

pub fn module(program: &mut Program) {
  program.funcs.insert(
    interner().get_or_intern_static("len"),
    |program, trace_expr| {
      let list = program.stack.last().ok_or_else(|| EvalError {
        expr: trace_expr.clone(),
        program: program.clone(),
        message: "Stack underflow".into(),
      })?;

      match list {
        Expr::List(list) => match i64::try_from(list.len()) {
          Ok(i) => program.push(Expr::Integer(i)),
          Err(_) => program.push(Expr::Nil),
        },
        _ => program.push(Expr::Nil),
      }

      Ok(())
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("index"),
    |program, trace_expr| {
      let index = program.pop(trace_expr)?;
      let list = program.stack.last().ok_or_else(|| EvalError {
        expr: trace_expr.clone(),
        program: program.clone(),
        message: "Stack underflow".into(),
      })?;

      match index {
        Expr::Integer(index) => match usize::try_from(index) {
          Ok(i) => match list {
            Expr::List(list) => {
              program.push(list.get(i).cloned().unwrap_or(Expr::Nil))
            }
            _ => program.push(Expr::Nil),
          },
          Err(_) => program.push(Expr::Nil),
        },
        _ => program.push(Expr::Nil),
      }

      Ok(())
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("split"),
    |program, trace_expr| {
      let index = program.pop(trace_expr)?;
      let list = program.pop(trace_expr)?;

      match index {
        Expr::Integer(index) => match usize::try_from(index) {
          Ok(i) => match list {
            Expr::List(mut list) => {
              if i <= list.len() {
                let rest = list.split_off(i);
                program.push(Expr::List(list));
                program.push(Expr::List(rest));
              } else {
                program.push(Expr::Nil);
              }
            }
            _ => program.push(Expr::Nil),
          },
          Err(_) => program.push(Expr::Nil),
        },
        _ => program.push(Expr::Nil),
      }

      Ok(())
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("join"),
    |program, trace_expr| {
      let delimiter = program.pop(trace_expr)?;
      let list = program.pop(trace_expr)?;

      match (delimiter, list) {
        (Expr::String(delimiter), Expr::List(list)) => {
          let delimiter_str = interner().resolve(&delimiter);

          let string = list
            .into_iter()
            .map(|expr| match expr {
              Expr::String(string) => interner().resolve(&string).to_string(),
              _ => expr.to_string(),
            })
            .join(delimiter_str);
          let string = Expr::String(interner().get_or_intern(string));
          program.push(string);

          Ok(())
        }
        (delimiter, list) => Err(EvalError {
          expr: trace_expr.clone(),
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
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
      let list_rhs = program.pop(trace_expr)?;
      let list_lhs = program.pop(trace_expr)?;

      match (list_lhs, list_rhs) {
        (Expr::List(mut list_lhs), Expr::List(list_rhs)) => {
          list_lhs.extend(list_rhs);
          program.push(Expr::List(list_lhs));
        }
        _ => program.push(Expr::Nil),
      }

      Ok(())
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("unwrap"),
    |program, trace_expr| {
      let list = program.pop(trace_expr)?;

      match list {
        Expr::List(list) => program.stack.extend(list),
        list => program.push(list),
      }

      Ok(())
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("wrap"),
    |program, trace_expr| {
      let any = program.pop(trace_expr)?;
      program.push(Expr::List(vec![any]));
      Ok(())
    },
  );
}
