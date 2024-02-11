use itertools::Itertools as _;
use lasso::Spur;

use crate::{
  interner::interner, module, Expr, Func, Lexer, Module, Parser, Scanner, Scope,
};
use core::{fmt, iter};
use std::{collections::HashMap, time::SystemTime};

#[derive(Debug, Clone)]
pub struct LoadedFile {
  pub contents: Spur,
  pub mtime: SystemTime,
}

#[derive(Debug, Clone)]
pub struct Program {
  pub stack: Vec<Expr>,
  pub scopes: Vec<Scope>,
  pub funcs: HashMap<Spur, Func>,
  pub loaded_files: HashMap<String, LoadedFile>,
  pub debug_trace: Option<Vec<Expr>>,
}

impl Default for Program {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for Program {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Stack: [")?;

    self.stack.iter().enumerate().try_for_each(|(i, expr)| {
      if i == self.stack.len() - 1 {
        write!(f, "{}", expr)
      } else {
        write!(f, "{}, ", expr)
      }
    })?;
    write!(f, "]")?;

    writeln!(f,)?;

    if let Some(trace) = &self.debug_trace {
      writeln!(f, "Trace:\n  {}", trace.iter().rev().take(20).join("\n  "))?;
    }

    if !self.scopes.is_empty() {
      writeln!(f, "Scope:")?;

      let layer = self.scopes.last().unwrap();
      let items = layer.items.len();
      for (item_i, (key, value)) in
        layer.items.iter().sorted_by_key(|(s, _)| *s).enumerate()
      {
        if item_i == items - 1 {
          write!(
            f,
            " + {}: {}",
            interner().resolve(key),
            match value.borrow().val() {
              Some(expr) => expr.to_string(),
              None => "None".to_owned(),
            }
          )?;
        } else {
          writeln!(
            f,
            " + {}: {}",
            interner().resolve(key),
            match value.borrow().val() {
              Some(expr) => expr.to_string(),
              None => "None".to_owned(),
            }
          )?;
        }
      }
    }

    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct EvalError {
  pub program: Program,
  pub message: String,
  pub expr: Expr,
}

impl fmt::Display for EvalError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "Error: {}", self.message)?;
    writeln!(f, "Expr: {}", self.expr)?;
    writeln!(f,)?;
    write!(f, "{}", self.program)
  }
}

impl Program {
  #[inline]
  pub fn new() -> Self {
    Self {
      stack: vec![],
      scopes: vec![Scope::new()],
      funcs: HashMap::new(),
      loaded_files: HashMap::new(),
      debug_trace: None,
    }
  }

  pub fn with_core(mut self) -> Result<Self, EvalError> {
    module::core::Core::default().link(&mut self)?;
    Ok(self)
  }

  pub fn with_module<M>(mut self, module: M) -> Result<Self, EvalError>
  where
    M: Module,
  {
    module.link(&mut self)?;
    Ok(self)
  }

  pub fn with_debug(mut self) -> Self {
    self.debug_trace = Some(vec![]);
    self
  }

  pub fn loaded_files(&self) -> impl Iterator<Item = &str> {
    self.loaded_files.keys().map(|s| s.as_str())
  }

  pub fn pop(&mut self, trace_expr: &Expr) -> Result<Expr, EvalError> {
    self.stack.pop().ok_or_else(|| EvalError {
      expr: trace_expr.clone(),
      program: self.clone(),
      message: "Stack underflow".into(),
    })
  }

  pub fn push(&mut self, expr: Expr) {
    let expr = if expr.is_function() {
      let mut scanner =
        Scanner::new(self.scopes.last().unwrap().duplicate(), &self.funcs);

      // TODO: Don't silently fail here
      scanner.scan(expr.clone()).unwrap_or(expr)
    } else {
      expr
    };

    self.stack.push(expr)
  }

  pub fn scope_item(&self, symbol: &str) -> Option<Expr> {
    self
      .scopes
      .last()
      .and_then(|layer| layer.get_val(interner().get_or_intern(symbol)))
  }

  pub fn def_scope_item(
    &mut self,
    trace_expr: &Expr,
    symbol: &str,
    value: Expr,
  ) -> Result<(), EvalError> {
    if let Some(layer) = self.scopes.last_mut() {
      match layer.define(interner().get_or_intern(symbol), value) {
        Ok(_) => Ok(()),
        Err(message) => Err(EvalError {
          expr: trace_expr.clone(),
          program: self.clone(),
          message,
        }),
      }
    } else {
      Err(EvalError {
        expr: trace_expr.clone(),
        program: self.clone(),
        message: format!("no scope to define {symbol}"),
      })
    }
  }

  pub fn set_scope_item(
    &mut self,
    trace_expr: &Expr,
    symbol: &str,
    value: Expr,
  ) -> Result<(), EvalError> {
    if let Some(layer) = self.scopes.last_mut() {
      match layer.set(interner().get_or_intern(symbol), value) {
        Ok(_) => Ok(()),
        Err(message) => Err(EvalError {
          expr: trace_expr.clone(),
          program: self.clone(),
          message,
        }),
      }
    } else {
      Err(EvalError {
        expr: trace_expr.clone(),
        program: self.clone(),
        message: format!("no scope to set {symbol}"),
      })
    }
  }

  // TODO: Make this return a result
  pub fn remove_scope_item(&mut self, symbol: &str) {
    if let Some(layer) = self.scopes.last_mut() {
      layer.remove(interner().get_or_intern(symbol)).unwrap();
    }
  }

  pub fn push_scope(&mut self, scope: Scope) {
    self.scopes.push(scope);
  }

  pub fn pop_scope(&mut self) {
    self.scopes.pop();
  }

  fn eval_call(
    &mut self,
    trace_expr: &Expr,
    call: Spur,
  ) -> Result<(), EvalError> {
    let call_str = interner().resolve(&call);

    if let Some(func) = self.funcs.get(&call) {
      return func(self, trace_expr);
    }

    if let Some(value) = self.scope_item(call_str) {
      if value.is_function() {
        self.eval_expr(Expr::Lazy(Box::new(Expr::Call(call))))?;
        self.eval_call(trace_expr, interner().get_or_intern_static("get"))?;
        self.eval_call(trace_expr, interner().get_or_intern_static("call"))?;

        Ok(())
      } else {
        self.push(self.scope_item(call_str).unwrap_or(Expr::Nil));
        Ok(())
      }
    } else {
      Err(EvalError {
        expr: trace_expr.clone(),
        program: self.clone(),
        message: format!("unknown call {call_str}"),
      })
    }
  }

  pub fn eval_expr(&mut self, expr: Expr) -> Result<(), EvalError> {
    if let Some(trace) = &mut self.debug_trace {
      trace.push(expr.clone());
    }

    match expr.clone() {
      Expr::Call(call) => self.eval_call(&expr, call),
      Expr::Lazy(block) => {
        self.push(*block);
        Ok(())
      }
      Expr::List(list) => {
        let stack_len = self.stack.len();

        self.eval(list)?;

        let list_len = self.stack.len() - stack_len;

        let mut list = iter::repeat_with(|| self.pop(&expr).unwrap())
          .take(list_len)
          .collect::<Vec<_>>();
        list.reverse();

        self.push(Expr::List(list));

        Ok(())
      }
      Expr::Fn(_) => Ok(()),
      expr => {
        self.push(expr);
        Ok(())
      }
    }
  }

  pub fn eval_string(&mut self, line: &str) -> Result<(), EvalError> {
    let lexer = Lexer::new(line);
    let parser = Parser::new(lexer);
    // TODO: It might be time to add a proper EvalError enum.
    let exprs = parser.parse().map_err(|e| EvalError {
      program: self.clone(),
      message: e.to_string(),
      expr: Expr::Nil,
    })?;

    self.eval(exprs)
  }

  pub fn eval(&mut self, exprs: Vec<Expr>) -> Result<(), EvalError> {
    let mut clone = self.clone();
    let result = exprs.into_iter().try_for_each(|expr| clone.eval_expr(expr));

    self.loaded_files = clone.loaded_files;

    match result {
      Ok(x) => {
        // TODO: Store each operation in an append-only operations list, and
        //       rollback if there is an error.
        self.stack = clone.stack;
        self.scopes = clone.scopes;
        self.debug_trace = clone.debug_trace;

        Ok(x)
      }
      Err(e) => Err(e),
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::FnSymbol;

  use super::*;

  mod eval {
    use super::*;

    #[test]
    fn implicitly_adds_to_stack() {
      let mut program = Program::new().with_core().unwrap();
      program.eval_string("1 2").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(1), Expr::Integer(2)]);
    }

    #[test]
    fn add_two_numbers() {
      let mut program = Program::new().with_core().unwrap();
      program.eval_string("1 2 +").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(3)]);
    }

    #[test]
    fn subtract_two_numbers() {
      let mut program = Program::new().with_core().unwrap();
      program.eval_string("1 2 -").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(-1)]);
    }

    #[test]
    fn multiply_two_numbers() {
      let mut program = Program::new().with_core().unwrap();
      program.eval_string("1 2 *").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(2)]);
    }

    #[test]
    fn divide_two_numbers() {
      let mut program = Program::new().with_core().unwrap();
      program.eval_string("1 2 /").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(0)]);

      let mut program = Program::new().with_core().unwrap();
      program.eval_string("1.0 2.0 /").unwrap();
      assert_eq!(program.stack, vec![Expr::Float(0.5)]);
    }

    #[test]
    fn modulo_two_numbers() {
      let mut program = Program::new().with_core().unwrap();
      program.eval_string("10 5 %").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(0)]);

      let mut program = Program::new().with_core().unwrap();
      program.eval_string("11 5 %").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(1)]);
    }

    #[test]
    fn complex_operations() {
      let mut program = Program::new().with_core().unwrap();
      program.eval_string("1 2 + 3 *").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(9)]);
    }

    #[test]
    fn eval_from_stack() {
      let mut program = Program::new().with_core().unwrap();
      program.eval_string("'(1 2 +) unwrap call").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(3)]);
    }

    #[test]
    fn dont_eval_skips() {
      let mut program = Program::new().with_core().unwrap();
      program.eval_string("6 'var def 'var").unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::Call(interner().get_or_intern_static("var"))]
      );
    }

    #[test]
    fn eval_lists() {
      let mut program = Program::new().with_core().unwrap();
      program.eval_string("(1 2 3)").unwrap();
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
    fn eval_lists_eagerly() {
      let mut program = Program::new().with_core().unwrap();
      program.eval_string("6 'var def (var)").unwrap();
      assert_eq!(program.stack, vec![Expr::List(vec![Expr::Integer(6)])]);
    }
  }

  mod comparison {
    use super::*;

    mod greater_than {
      use super::*;

      #[test]
      fn greater_than_int() {
        let mut program = Program::new().with_core().unwrap();
        program.eval_string("1 1 >").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("1 2 >").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("2 1 >").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);
      }

      #[test]
      fn greater_than_float() {
        let mut program = Program::new().with_core().unwrap();
        program.eval_string("1.0 1.0 >").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("1.0 1.1 >").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("1.1 1.0 >").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);
      }

      #[test]
      fn greater_than_int_and_float() {
        // Int first
        let mut program = Program::new().with_core().unwrap();
        program.eval_string("1 1.0 >").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("1 1.1 >").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("2 1.0 >").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);

        // Float first
        let mut program = Program::new().with_core().unwrap();
        program.eval_string("1.0 1 >").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("1.0 1 >").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("1.1 1 >").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);
      }
    }

    mod less_than {
      use super::*;

      #[test]
      fn less_than_int() {
        let mut program = Program::new().with_core().unwrap();
        program.eval_string("1 1 <").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("1 2 <").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("2 1 <").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);
      }

      #[test]
      fn less_than_float() {
        let mut program = Program::new().with_core().unwrap();
        program.eval_string("1.0 1.0 <").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("1.0 1.1 <").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("1.1 1.0 <").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);
      }

      #[test]
      fn less_than_int_and_float() {
        // Int first
        let mut program = Program::new().with_core().unwrap();
        program.eval_string("1 1.0 <").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("1 1.1 <").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("2 1.0 <").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        // Float first
        let mut program = Program::new().with_core().unwrap();
        program.eval_string("1.0 1 <").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("0.9 1 <").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("1.1 1 <").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);
      }
    }

    mod bitwise {
      use super::*;

      #[test]
      fn and_int() {
        let mut program = Program::new().with_core().unwrap();
        program.eval_string("1 1 and").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("1 0 and").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("0 1 and").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("0 0 and").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);
      }

      #[test]
      fn and_bool() {
        let mut program = Program::new().with_core().unwrap();
        program.eval_string("true true and").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("true false and").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("false true and").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("false false and").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);
      }

      #[test]
      fn or_int() {
        let mut program = Program::new().with_core().unwrap();
        program.eval_string("1 1 or").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("1 0 or").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("0 1 or").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("0 0 or").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);
      }

      #[test]
      fn or_bool() {
        let mut program = Program::new().with_core().unwrap();
        program.eval_string("true true or").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("true false or").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("false true or").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);

        let mut program = Program::new().with_core().unwrap();
        program.eval_string("false false or").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);
      }
    }
  }

  mod variables {
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

      assert_eq!(a, Expr::Integer(1));
    }

    #[test]
    fn retrieving_variables() {
      let mut program = Program::new().with_core().unwrap();
      program.eval_string("1 'a def a").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(1)]);
    }

    #[test]
    fn evaluating_variables() {
      let mut program = Program::new().with_core().unwrap();
      program.eval_string("1 'a def a 2 +").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(3)]);
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
      assert_eq!(program.stack, vec![Expr::Integer(3)]);
    }

    #[test]
    fn only_auto_call_functions() {
      let mut program = Program::new().with_core().unwrap();
      program
        .eval_string("'(1 2 +) 'is-three def is-three")
        .unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::List(vec![
          Expr::Integer(1),
          Expr::Integer(2),
          Expr::Call(interner().get_or_intern_static("+"))
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
        vec![Expr::List(vec![
          Expr::Fn(FnSymbol {
            scoped: true,
            scope: Scope::new(),
          }),
          Expr::Integer(1),
          Expr::Integer(2),
          Expr::Call(interner().get_or_intern_static("+"))
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
          Expr::List(vec![
            Expr::Fn(FnSymbol {
              scoped: true,
              scope: Scope::new(),
            }),
            Expr::Integer(1),
            Expr::Integer(2),
            Expr::Call(interner().get_or_intern_static("+"))
          ]),
          Expr::Integer(3)
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

        assert_eq!(a, Expr::Integer(0));
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

        assert_eq!(a, Expr::Integer(1));
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

        assert_eq!(a, Expr::Integer(0));
        assert_eq!(program.stack, vec![Expr::Integer(1), Expr::Integer(0)])
      }
    }
  }

  mod stack_ops {
    use super::*;

    #[test]
    fn clearing_stack() {
      let mut program = Program::new().with_core().unwrap();
      program.eval_string("1 2 clear").unwrap();
      assert_eq!(program.stack, vec![]);
    }

    #[test]
    fn dropping_from_stack() {
      let mut program = Program::new().with_core().unwrap();
      program.eval_string("1 2 drop").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(1)]);
    }

    #[test]
    fn duplicating() {
      let mut program = Program::new().with_core().unwrap();
      program.eval_string("1 dup").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(1), Expr::Integer(1)]);
    }

    #[test]
    fn swapping() {
      let mut program = Program::new().with_core().unwrap();
      program.eval_string("1 2 swap").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(2), Expr::Integer(1)]);
    }

    #[test]
    fn rotating() {
      let mut program = Program::new().with_core().unwrap();
      program.eval_string("1 2 3 rot").unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::Integer(3), Expr::Integer(1), Expr::Integer(2)]
      );
    }

    #[test]
    fn collect() {
      let mut program = Program::new().with_core().unwrap();
      program.eval_string("1 2 3 collect").unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::List(vec![
          Expr::Integer(1),
          Expr::Integer(2),
          Expr::Integer(3)
        ])]
      );
    }

    // TODO: wtf is this? do we still need this test??
    // #[test]
    // fn collect_and_unwrap() {
    //   let mut program = Program::new().with_core().unwrap();
    //   program
    //     .eval_string("1 2 3 collect 'a set 'a get unwrap")
    //     .unwrap();
    //   assert_eq!(
    //     program.stack,
    //     vec![Expr::Integer(1), Expr::Integer(2), Expr::Integer(3)]
    //   );
    //   assert_eq!(
    //     program.scopes,
    //     vec![HashMap::from_iter(vec![(
    //       "a".to_string(),
    //       Expr::List(vec![
    //         Expr::Integer(1),
    //         Expr::Integer(2),
    //         Expr::Integer(3)
    //       ])
    //     )])]
    //   );
    // }
  }

  mod list_ops {
    use super::*;

    #[test]
    fn concatenating_lists() {
      let mut program = Program::new().with_core().unwrap();
      program.eval_string("(1 2) (3 \"4\") concat").unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::List(vec![
          Expr::Integer(1),
          Expr::Integer(2),
          Expr::Integer(3),
          Expr::String(interner().get_or_intern_static("4"))
        ])]
      );
    }

    #[test]
    fn concatenating_blocks() {
      let mut program = Program::new().with_core().unwrap();
      program.eval_string("(1 2) ('+) concat").unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::List(vec![
          Expr::Integer(1),
          Expr::Integer(2),
          Expr::Call(interner().get_or_intern_static("+"))
        ])]
      );
    }

    #[test]
    fn getting_length_of_list() {
      let mut program = Program::new().with_core().unwrap();
      program.eval_string("(1 2 3) len").unwrap();
      assert_eq!(
        program.stack,
        vec![
          Expr::List(vec![
            Expr::Integer(1),
            Expr::Integer(2),
            Expr::Integer(3)
          ]),
          Expr::Integer(3)
        ]
      );
    }

    #[test]
    fn getting_indexed_item_of_list() {
      let mut program = Program::new().with_core().unwrap();
      program.eval_string("(1 2 3) 1 index").unwrap();
      assert_eq!(
        program.stack,
        vec![
          Expr::List(vec![
            Expr::Integer(1),
            Expr::Integer(2),
            Expr::Integer(3)
          ]),
          Expr::Integer(2)
        ]
      );
    }
  }

  // TODO: Make this a test again
  // mod string_ops {
  //   use super::*;

  //   // #[test]
  //   // fn exploding_string() {
  //   //   let mut program = Program::new().with_core().unwrap();
  //   //   program.eval_string("\"abc\" explode").unwrap();
  //   //   assert_eq!(
  //   //     program.stack,
  //   //     vec![Expr::List(vec![
  //   //       Expr::String(interner().get_or_intern_static("a")),
  //   //       Expr::String(interner().get_or_intern_static("b")),
  //   //       Expr::String(interner().get_or_intern_static("c"))
  //   //     ])]
  //   //   );
  //   // }

  //   // #[test]
  //   // fn joining_to_string() {
  //   //   let mut program = Program::new().with_core().unwrap();
  //   //   program
  //   //     .eval_string("(\"a\" 3 \"hello\" 1.2) \"\" join")
  //   //     .unwrap();

  //   //   assert_eq!(
  //   //     program.stack,
  //   //     vec![Expr::String(interner().get_or_intern_static("a3hello1.2"))]
  //   //   );
  //   // }
  // }

  mod control_flow {
    use super::*;

    #[test]
    fn if_true() {
      let mut program = Program::new().with_core().unwrap();
      program
        .eval_string("1 2 + '(\"correct\") '(3 =) if")
        .unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::String(interner().get_or_intern_static("correct"))]
      );
    }

    #[test]
    fn if_empty_condition() {
      let mut program = Program::new().with_core().unwrap();
      program
        .eval_string("1 2 + 3 = '(\"correct\") '() if")
        .unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::String(interner().get_or_intern_static("correct"))]
      );
    }

    #[test]
    fn if_else_true() {
      let mut program = Program::new().with_core().unwrap();
      program
        .eval_string("1 2 + 3 = '(\"incorrect\") '(\"correct\") '() ifelse")
        .unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::String(interner().get_or_intern_static("correct"))]
      );
    }

    #[test]
    fn if_else_false() {
      let mut program = Program::new().with_core().unwrap();
      program
        .eval_string("1 2 + 2 = '(\"incorrect\") '(\"correct\") '() ifelse")
        .unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::String(interner().get_or_intern_static("incorrect"))]
      );
    }
  }

  mod loops {
    use super::*;

    #[test]
    fn while_loop() {
      let mut program = Program::new().with_core().unwrap();
      program
        .eval_string(
          ";; Set i to 3
           3 'i def

           '(
             ;; Decrement i by 1
             i 1 -
             ;; Set i
             'i set

             i
           ) '(
             ;; If i is 0, break
             i 0 !=
           ) while",
        )
        .unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::Integer(2), Expr::Integer(1), Expr::Integer(0)]
      );
    }
  }

  mod type_ops {
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
          Expr::List(vec![
            Expr::Integer(1),
            Expr::Integer(2),
            Expr::Integer(3)
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
        vec![Expr::Lazy(
          Expr::Call(interner().get_or_intern_static("set")).into()
        )]
      );
    }
  }
}
