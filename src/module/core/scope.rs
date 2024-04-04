use crate::{interner::interner, Ast, EvalError, Expr, Program, Type};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program.funcs.insert(
    interner().get_or_intern_static("def"),
    |program, trace_expr| {
      let key = program.pop_expr(trace_expr)?;
      let (val, val_index) = program.pop_with_index(trace_expr)?;

      match key {
        Expr::Call(ref key) => match program.funcs.contains_key(key) {
          true => Err(EvalError {
            expr: trace_expr,
            program: program.clone(),
            message: format!(
              "cannot shadow a native function {}",
              interner().resolve(key)
            ),
          }),
          false => program.def_scope_item(
            trace_expr,
            interner().resolve(key),
            val_index,
          ),
        },
        key => Err(EvalError {
          expr: trace_expr,
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::List(vec![
              // TODO: A type to represent functions.
              Type::Any,
              Type::Call,
            ]),
            Type::List(vec![
              val.type_of(&program.ast),
              key.type_of(&program.ast),
            ]),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("undef"),
    |program, trace_expr| {
      let item = program.pop_expr(trace_expr)?;

      match item {
        Expr::Call(key) => {
          let key_str = interner().resolve(&key).to_owned();
          match program.remove_scope_item(&key_str) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
          }
        }
        item => Err(EvalError {
          expr: trace_expr,
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::Call,
            item.type_of(&program.ast),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("set"),
    |program, trace_expr| {
      let key = program.pop_expr(trace_expr)?;
      let (val, val_index) = program.pop_with_index(trace_expr)?;

      match key {
        Expr::Call(ref key) => match program.funcs.contains_key(key) {
          true => Err(EvalError {
            expr: trace_expr,
            program: program.clone(),
            message: format!(
              "cannot shadow a native function {}",
              interner().resolve(key)
            ),
          }),
          false => program.set_scope_item(
            trace_expr,
            interner().resolve(key),
            val_index,
          ),
        },
        key => Err(EvalError {
          expr: trace_expr,
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::List(vec![
              // TODO: A type to represent functions.
              Type::Any,
              Type::Call,
            ]),
            Type::List(vec![
              val.type_of(&program.ast),
              key.type_of(&program.ast),
            ]),
          ),
        }),
      }
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("get"),
    |program, trace_expr| {
      let item = program.pop_expr(trace_expr)?;

      match item {
        Expr::Call(ref key) => {
          if let Some(func) = program.funcs.get(key) {
            func(program, trace_expr)
          } else {
            let key_str = interner().resolve(key);

            // Always push something, otherwise it can get tricky to manage the
            // stack in-langauge.
            program.push(program.scope_item(key_str).unwrap_or(Ast::NIL))
          }
        }
        item => Err(EvalError {
          expr: trace_expr,
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::Call,
            item.type_of(&program.ast),
          ),
        }),
      }
    },
  );

  Ok(())
}

#[cfg(test)]

mod tests {
  use crate::{ExprTree, FnSymbol, Scope};

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
    let a = program.ast.expr(a).unwrap().into_expr_tree(&program.ast);

    assert_eq!(a, ExprTree::Integer(1));
  }

  #[test]
  fn retrieving_variables() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 'a def a").unwrap();
    assert_eq!(program.stack_exprs(), vec![ExprTree::Integer(1)]);
  }

  #[test]
  fn evaluating_variables() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 'a def a 2 +").unwrap();
    assert_eq!(program.stack_exprs(), vec![ExprTree::Integer(3)]);
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
    assert_eq!(program.stack_exprs(), vec![ExprTree::Integer(3)]);
  }

  #[test]
  fn only_auto_call_functions() {
    let mut program = Program::new().with_core().unwrap();
    program
      .eval_string("'(1 2 +) 'is-three def is-three")
      .unwrap();
    assert_eq!(
      program.stack_exprs(),
      vec![ExprTree::List(vec![
        ExprTree::Integer(1),
        ExprTree::Integer(2),
        ExprTree::Call(interner().get_or_intern_static("+"))
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
      program.stack_exprs(),
      vec![ExprTree::List(vec![
        ExprTree::Fn(FnSymbol {
          scoped: true,
          scope: Scope::new(),
        }),
        ExprTree::Integer(1),
        ExprTree::Integer(2),
        ExprTree::Call(interner().get_or_intern_static("+"))
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
      program.stack_exprs(),
      vec![
        ExprTree::List(vec![
          ExprTree::Fn(FnSymbol {
            scoped: true,
            scope: Scope::new(),
          }),
          ExprTree::Integer(1),
          ExprTree::Integer(2),
          ExprTree::Call(interner().get_or_intern_static("+"))
        ]),
        ExprTree::Integer(3)
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
      let a = program.ast.expr(a).unwrap().into_expr_tree(&program.ast);

      assert_eq!(a, ExprTree::Integer(0));
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
      let a = program.ast.expr(a).unwrap().into_expr_tree(&program.ast);

      assert_eq!(a, ExprTree::Integer(1));
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
      let a = program.ast.expr(a).unwrap().into_expr_tree(&program.ast);

      assert_eq!(a, ExprTree::Integer(0));
      assert_eq!(
        program.stack_exprs(),
        vec![ExprTree::Integer(1), ExprTree::Integer(0)]
      )
    }
  }
}
