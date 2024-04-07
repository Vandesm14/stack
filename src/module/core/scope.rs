use crate::{
  interner::interner, DebugData, EvalError, EvalErrorKind, ExprKind, Program,
  Type,
};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program.funcs.insert(
    interner().get_or_intern_static("def"),
    |program, trace_expr| {
      let key = program.pop(trace_expr)?;
      let val = program.pop(trace_expr)?;

      match key.val {
        ExprKind::Call(ref key) => match program.funcs.contains_key(key) {
          true => Err(EvalError {
            expr: Some(trace_expr),
            kind: EvalErrorKind::Message(
              "cannot shadow a native function".into(),
            ),
          }),
          false => {
            program.def_scope_item(trace_expr, interner().resolve(key), val);
            Ok(())
          }
        },
        _ => Err(EvalError {
          expr: Some(trace_expr),
          kind: EvalErrorKind::ExpectedFound(
            Type::List(vec![Type::Any, Type::Call]),
            Type::List(vec![val.val.type_of(), key.val.type_of()]),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("undef"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item.val {
        ExprKind::Call(key) => {
          let key_str = interner().resolve(&key).to_owned();
          program.remove_scope_item(&key_str);

          Ok(())
        }
        item => Err(EvalError {
          expr: Some(trace_expr),
          kind: EvalErrorKind::ExpectedFound(Type::Call, item.type_of()),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("set"),
    |program, trace_expr| {
      let key = program.pop(trace_expr)?;
      let val = program.pop(trace_expr)?;

      match key.val {
        ExprKind::Call(ref key) => match program.funcs.contains_key(key) {
          true => Err(EvalError {
            expr: Some(trace_expr),
            kind: EvalErrorKind::Message(
              "cannot shadow a native function".into(),
            ),
          }),
          false => {
            program.set_scope_item(trace_expr, interner().resolve(key), val);
            Ok(())
          }
        },
        key => Err(EvalError {
          expr: Some(trace_expr),
          kind: EvalErrorKind::ExpectedFound(
            Type::List(vec![Type::Any, Type::Call]),
            Type::List(vec![val.val.type_of(), key.type_of()]),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("get"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item.val {
        ExprKind::Call(ref key) => {
          if let Some(func) = program.funcs.get(key) {
            func(program, trace_expr)
          } else {
            let key_str = interner().resolve(key);

            // Always push something, otherwise it can get tricky to manage the
            // stack in-langauge.
            program.push(program.scope_item(key_str).unwrap_or(
              ExprKind::Nil.into_expr(DebugData::only_ingredients(vec![
                item,
                trace_expr.clone(),
              ])),
            ))
          }
        }
        item => Err(EvalError {
          expr: Some(trace_expr),
          kind: EvalErrorKind::ExpectedFound(Type::Call, item.type_of()),
        }),
      }
    },
  );

  Ok(())
}

#[cfg(test)]

mod tests {
  use crate::{FnSymbol, Scope};

  use super::*;

  #[test]
  fn storing_variables() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 'a def").unwrap();

    let a = program
      .scopes
      .last()
      .unwrap()
      .get_val(interner().get_or_intern("a"))
      .unwrap();

    assert_eq!(a, ExprKind::Integer(1));
  }

  #[test]
  fn retrieving_variables() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 'a def a").unwrap();
    assert_eq!(program.stack, vec![ExprKind::Integer(1)]);
  }

  #[test]
  fn evaluating_variables() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 'a def a 2 +").unwrap();
    assert_eq!(program.stack, vec![ExprKind::Integer(3)]);
  }

  #[test]
  fn removing_variables() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 'a def 'a undef").unwrap();
    assert!(!program
      .scopes
      .iter()
      .any(|scope| scope.has(interner().get_or_intern_static("a"))))
  }

  #[test]
  fn auto_calling_functions() {
    let mut program = Program::new().with_core().unwrap();
    program
      .eval_string("'(fn 1 2 +) 'is-three def is-three")
      .unwrap();
    assert_eq!(program.stack, vec![ExprKind::Integer(3)]);
  }

  #[test]
  fn only_auto_call_functions() {
    let mut program = Program::new().with_core().unwrap();
    program
      .eval_string("'(1 2 +) 'is-three def is-three")
      .unwrap();
    assert_eq!(
      program.stack,
      vec![ExprKind::List(vec![
        ExprKind::Integer(1),
        ExprKind::Integer(2),
        ExprKind::Call(interner().get_or_intern_static("+"))
      ])]
    );
  }

  #[test]
  fn getting_function_body() {
    let mut program = Program::new().with_core().unwrap();
    program
      .eval_string("'(fn 1 2 +) 'is-three def 'is-three get")
      .unwrap();
    assert_eq!(
      program.stack,
      vec![ExprKind::List(vec![
        ExprKind::Fn(FnSymbol {
          scoped: true,
          scope: Scope::new(),
        }),
        ExprKind::Integer(1),
        ExprKind::Integer(2),
        ExprKind::Call(interner().get_or_intern_static("+"))
      ])]
    );
  }

  #[test]
  fn assembling_functions_in_code() {
    let mut program = Program::new().with_core().unwrap();
    program
      .eval_string("'() 'fn tolist concat 1 tolist concat 2 tolist concat '+ tolist concat dup call")
      .unwrap();
    assert_eq!(
      program.stack,
      vec![
        ExprKind::List(vec![
          ExprKind::Fn(FnSymbol {
            scoped: true,
            scope: Scope::new(),
          }),
          ExprKind::Integer(1),
          ExprKind::Integer(2),
          ExprKind::Call(interner().get_or_intern_static("+"))
        ]),
        ExprKind::Integer(3)
      ]
    );
  }

  mod scope {
    use super::*;

    #[test]
    fn functions_are_isolated() {
      let mut program = Program::new().with_core().unwrap();
      program
        .eval_string(
          "0 'a def
          '(fn 5 'a def)

          '(fn 1 'a def call) call",
        )
        .unwrap();

      let a = program
        .scopes
        .last()
        .unwrap()
        .get_val(interner().get_or_intern("a"))
        .unwrap();

      assert_eq!(a, ExprKind::Integer(0));
    }

    #[test]
    fn functions_can_use_same_scope() {
      let mut program = Program::new().with_core().unwrap();
      program
        .eval_string(
          "0 'a def
          '(fn! 1 'a def) call",
        )
        .unwrap();

      let a = program
        .scopes
        .last()
        .unwrap()
        .get_val(interner().get_or_intern("a"))
        .unwrap();

      assert_eq!(a, ExprKind::Integer(1));
    }

    #[test]
    fn functions_can_shadow_vars() {
      let mut program = Program::new().with_core().unwrap();
      program
        .eval_string(
          "0 'a def
          '(fn 1 'a def a) call a",
        )
        .unwrap();

      let a = program
        .scopes
        .last()
        .unwrap()
        .get_val(interner().get_or_intern("a"))
        .unwrap();

      assert_eq!(a, ExprKind::Integer(0));
      assert_eq!(
        program.stack,
        vec![ExprKind::Integer(1), ExprKind::Integer(0)]
      )
    }
  }
}
