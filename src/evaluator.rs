use itertools::Itertools as _;
use lasso::Spur;

use crate::{
  interner::interner, module, Ast, AstIndex, Expr, Func, Lexer, Module, Parser,
  Scanner, Scope,
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
  pub stack: Vec<AstIndex>,
  pub ast: Ast,
  pub scopes: Vec<Scope>,
  pub funcs: HashMap<Spur, Func>,
  pub loaded_files: HashMap<String, LoadedFile>,
  pub debug_trace: Option<Vec<AstIndex>>,
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
  pub expr: AstIndex,
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
      ast: Ast::new(),
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

  pub fn ast_expr(
    &self,
    trace_expr: AstIndex,
    index: AstIndex,
  ) -> Result<&Expr, EvalError> {
    match self.ast.expr(index) {
      Some(expr) => Ok(expr),
      None => Err(EvalError {
        expr: trace_expr,
        program: self.clone(),
        message: "Failed to find expr in AST".into(),
      }),
    }
  }

  pub fn pop(&mut self, trace_expr: AstIndex) -> Result<AstIndex, EvalError> {
    self.stack.pop().ok_or_else(|| EvalError {
      expr: trace_expr,
      program: self.clone(),
      message: "Stack underflow".into(),
    })
  }

  pub fn pop_expr(&mut self, trace_expr: AstIndex) -> Result<&Expr, EvalError> {
    let expr = self.pop(trace_expr)?;

    self.ast_expr(trace_expr, expr)
  }

  pub fn pop_with_index(
    &mut self,
    trace_expr: AstIndex,
  ) -> Result<(&Expr, AstIndex), EvalError> {
    let index = self.pop(trace_expr)?;
    let expr = self.ast_expr(trace_expr, index)?;

    Ok((expr, index))
  }

  pub fn push(&mut self, expr: AstIndex) -> Result<(), EvalError> {
    let expr = if self.ast_expr(expr, expr).unwrap().is_function(&self.ast) {
      let mut scanner =
        Scanner::new(self.scopes.last().unwrap().duplicate(), &self.funcs);

      match scanner.scan(self.ast, expr) {
        Ok(expr) => expr,
        Err(message) => {
          return Err(EvalError {
            expr: Ast::NIL,
            program: self.clone(),
            message,
          })
        }
      }
    } else {
      expr
    };

    self.stack.push(expr);

    Ok(())
  }

  pub fn push_expr(&mut self, expr: Expr) -> Result<AstIndex, EvalError> {
    let index = self.ast.push_expr(expr);

    self.push(index)?;

    Ok(index)
  }

  pub fn scope_item(&self, symbol: &str) -> Option<AstIndex> {
    self
      .scopes
      .last()
      .and_then(|layer| layer.get_val(interner().get_or_intern(symbol)))
  }

  pub fn def_scope_item(
    &mut self,
    trace_expr: AstIndex,
    symbol: &str,
    value: AstIndex,
  ) -> Result<(), EvalError> {
    if let Some(layer) = self.scopes.last_mut() {
      match layer.define(interner().get_or_intern(symbol), value) {
        Ok(_) => Ok(()),
        Err(message) => Err(EvalError {
          expr: trace_expr,
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
    trace_expr: AstIndex,
    symbol: &str,
    value: AstIndex,
  ) -> Result<(), EvalError> {
    if let Some(layer) = self.scopes.last_mut() {
      match layer.set(interner().get_or_intern(symbol), value) {
        Ok(_) => Ok(()),
        Err(message) => Err(EvalError {
          expr: trace_expr,
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

  pub fn remove_scope_item(&mut self, symbol: &str) -> Result<(), EvalError> {
    if let Some(layer) = self.scopes.last_mut() {
      match layer.remove(interner().get_or_intern(symbol)) {
        Ok(_) => {}
        Err(message) => {
          return Err(EvalError {
            expr: Ast::NIL,
            program: self.clone(),
            message,
          })
        }
      }
    }

    Ok(())
  }

  pub fn push_scope(&mut self, scope: Scope) {
    self.scopes.push(scope);
  }

  pub fn pop_scope(&mut self) {
    self.scopes.pop();
  }

  fn eval_call(
    &mut self,
    trace_expr: AstIndex,
    call: Spur,
  ) -> Result<(), EvalError> {
    let call_str = interner().resolve(&call);

    // Intrinsics; Calls native Rust functions
    if let Some(func) = self.funcs.get(&call) {
      return func(self, trace_expr);
    }

    // Calls runtime values from scope
    if let Some(value) = self.scope_item(call_str) {
      if self
        .ast_expr(trace_expr, value)
        .unwrap()
        .is_function(&self.ast)
      {
        self.eval_expr(Expr::Lazy(trace_expr))?;
        self.eval_call(trace_expr, interner().get_or_intern_static("get"))?;
        self.eval_call(trace_expr, interner().get_or_intern_static("call"))?;

        Ok(())
      } else {
        self.push(self.scope_item(call_str).unwrap_or(Ast::NIL))
      }
    } else {
      Err(EvalError {
        expr: trace_expr,
        program: self.clone(),
        message: format!("unknown call {call_str}"),
      })
    }
  }

  pub fn eval_expr(&mut self, expr: Expr) -> Result<(), EvalError> {
    let index = self.ast.push_expr(expr);
    self.eval_index(index)
  }

  pub fn eval_index(&mut self, index: AstIndex) -> Result<(), EvalError> {
    if let Some(trace) = &mut self.debug_trace {
      trace.push(index);
    }

    let expr = self.ast_expr(index, index)?;
    match *expr {
      Expr::Call(call) => self.eval_call(index, call),
      Expr::Lazy(block) => self.push(block),
      Expr::List(list) => {
        let stack_len = self.stack.len();

        self.eval(self.ast.expr_many(list))?;

        let list_len = self.stack.len() - stack_len;

        let mut list = iter::repeat_with(|| self.pop(index).unwrap())
          .take(list_len)
          .collect::<Vec<_>>();
        list.reverse();

        let list = self.ast.push_expr(Expr::List(list));

        self.push(list)
      }
      Expr::Fn(_) => Ok(()),
      expr => self.push(index),
    }
  }

  pub fn eval_string(&mut self, line: &str) -> Result<(), EvalError> {
    let lexer = Lexer::new(line);
    let parser = Parser::new(lexer, &mut self.ast);

    let old_ast_size = self.ast.len();

    // TODO: It might be time to add a proper EvalError enum.
    let exprs = parser.parse().map_err(|e| EvalError {
      program: self.clone(),
      message: e.to_string(),
      expr: Ast::NIL,
    })?;

    let new_exprs = old_ast_size..self.ast.len();

    if let Some(new_exprs) = self.ast.expr_range(new_exprs) {
      self.eval(new_exprs.to_vec())
    } else {
      Err(EvalError {
        program: self.clone(),
        message: "Failed to find parsed exprs".into(),
        expr: Ast::NIL,
      })
    }
  }

  pub fn eval(&mut self, exprs: Vec<Expr>) -> Result<(), EvalError> {
    let mut clone = self.clone();
    let result = exprs.into_iter().try_for_each(|expr| clone.eval_expr(expr));

    self.loaded_files = clone.loaded_files;

    match result {
      Ok(x) => {
        self.stack = clone.stack;
        self.scopes = clone.scopes;
        self.debug_trace = clone.debug_trace;

        Ok(x)
      }
      Err(e) => Err(e),
    }
  }

  pub fn eval_indicies<I>(&mut self, indicies: I) -> Result<(), EvalError>
  where
    I: IntoIterator<Item = AstIndex>,
  {
    let mut clone = self.clone();
    let result = indicies
      .into_iter()
      .try_for_each(|expr| clone.eval_index(expr));

    self.loaded_files = clone.loaded_files;

    match result {
      Ok(x) => {
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
