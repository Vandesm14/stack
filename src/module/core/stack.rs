use crate::{interner::interner, EvalError, Expr, Program, Scanner, Type};

pub fn module(program: &mut Program) {
  program.funcs.insert(
    interner().get_or_intern_static("collect"),
    |program, _| {
      let list = core::mem::take(&mut program.stack);
      program.push(Expr::List(list));
      Ok(())
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

      program.push(item.clone());
      program.push(item);

      Ok(())
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("swap"),
    |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      program.push(rhs);
      program.push(lhs);

      Ok(())
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("rot"),
    |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let mid = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      program.push(rhs);
      program.push(lhs);
      program.push(mid);

      Ok(())
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("lazy"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;
      program.push(Expr::Lazy(Box::new(item)));
      Ok(())
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("call"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item {
        call @ Expr::Call(_) => program.eval_expr(call),
        // This is where auto-call is defined and functions are evaluated when
        // they are called via an identifier
        // TODO: Get this working again.
        item @ Expr::List(_) => match item.is_function() {
          true => {
            let mut scanner =
              Scanner::new(program.scopes.last().unwrap().duplicate());
            let item = scanner.scan(item.clone());

            match item {
              Ok(item) => {
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
              Err(message) => Err(EvalError {
                expr: trace_expr.clone(),
                program: program.clone(),
                message,
              }),
            }
          }
          false => {
            let Expr::List(list) = item else {
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
            item.type_of(),
          ),
        }),
      }
    },
  );
}
