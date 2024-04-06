use itertools::Itertools as _;
use lasso::Spur;

use crate::{
  interner::interner, module, DebugData, Expr, ExprKind, Func, Lexer, Module,
  Parser, Scanner, Scope,
};
use core::{fmt, iter};
use std::{collections::HashMap, time::SystemTime};

#[derive(Debug, Clone)]
pub struct SourceFile {
  pub contents: Spur,
  pub mtime: SystemTime,
}

#[derive(Debug, Clone)]
pub struct Program {
  pub stack: Vec<Expr>,
  pub scopes: Vec<Scope>,
  pub funcs: HashMap<Spur, Func>,
  pub sources: HashMap<String, SourceFile>,
  pub debug: bool,
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EvalErrorKind {
  Push,
  StackUnderflow,
  UnknownCall,
  ParseError,
  Message(String),
}

impl fmt::Display for EvalErrorKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Push => write!(f, "failed to push to the stack"),
      Self::StackUnderflow => write!(f, "stack underflow"),
      Self::UnknownCall => write!(f, "unknown call"),
      Self::ParseError => write!(f, "parse error"),
      Self::Message(message) => write!(f, "{}", message),
    }
  }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct EvalError<'a> {
  kind: EvalErrorKind,
  expr: Option<&'a Expr>,
}

impl<'a> fmt::Display for EvalError<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // writeln!(f, "Error: {}", self.message)?;
    // writeln!(f, "Expr: {}", self.expr)?;
    // writeln!(f,)?;
    // write!(f, "{}", self.program)
    todo!()
  }
}

impl Program {
  #[inline]
  pub fn new() -> Self {
    Self {
      stack: vec![],
      scopes: vec![Scope::new()],
      funcs: HashMap::new(),
      sources: HashMap::new(),
      debug: false,
    }
  }

  pub fn with_core(&mut self) -> Result<&mut Self, EvalError> {
    module::core::Core::default().link(self)?;
    Ok(self)
  }

  pub fn with_module<M>(&mut self, module: M) -> Result<&mut Self, EvalError>
  where
    M: Module,
  {
    module.link(self)?;
    Ok(self)
  }

  pub fn with_debug(mut self) -> Self {
    self.debug = true;
    self
  }

  pub fn loaded_files(&self) -> impl Iterator<Item = &str> {
    self.sources.keys().map(|s| s.as_str())
  }

  pub fn pop(&mut self, trace_expr: &Expr) -> Result<Expr, EvalError> {
    self.stack.pop().ok_or_else(|| EvalError {
      expr: Some(trace_expr),
      kind: EvalErrorKind::StackUnderflow,
    })
  }

  pub fn push(&mut self, expr: Expr) -> Result<(), EvalError> {
    let expr = if expr.val.is_function() {
      let mut scanner =
        Scanner::new(self.scopes.last().unwrap().duplicate(), &self.funcs);

      match scanner.scan(expr.clone()) {
        Ok(expr) => expr,
        Err(message) => {
          return Err(EvalError {
            expr: Some(&expr),
            kind: EvalErrorKind::Message(message),
          })
        }
      }
    } else {
      expr
    };

    self.stack.push(expr);

    Ok(())
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
  ) {
    if let Some(layer) = self.scopes.last_mut() {
      layer
        .define(interner().get_or_intern(symbol), value)
        .unwrap();
    }
  }

  pub fn set_scope_item(
    &mut self,
    trace_expr: &Expr,
    symbol: &str,
    value: Expr,
  ) {
    if let Some(layer) = self.scopes.last_mut() {
      layer.set(interner().get_or_intern(symbol), value);
    }
  }

  pub fn remove_scope_item(&mut self, symbol: &str) {
    if let Some(layer) = self.scopes.last_mut() {
      layer.remove(interner().get_or_intern(symbol));
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
      if value.val.is_function() {
        self.eval_expr(
          ExprKind::Lazy(Box::<Expr>::new(ExprKind::Call(call).into()).into())
            .into(),
        )?;
        self.eval_call(trace_expr, interner().get_or_intern_static("get"))?;
        self.eval_call(trace_expr, interner().get_or_intern_static("call"))?;

        Ok(())
      } else {
        self.push(self.scope_item(call_str).unwrap_or(ExprKind::Nil.into()))
      }
    } else {
      Err(EvalError {
        kind: EvalErrorKind::UnknownCall,
        expr: Some(trace_expr),
      })
    }
  }

  pub fn eval_expr(&mut self, expr: Expr) -> Result<(), EvalError> {
    match expr.clone().val {
      ExprKind::Call(call) => self.eval_call(&expr, call),
      ExprKind::Lazy(block) => self.push(*block),
      ExprKind::List(list) => {
        let stack_len = self.stack.len();

        self.eval(list)?;

        let list_len = self.stack.len() - stack_len;

        let mut list = iter::repeat_with(|| self.pop(&expr).unwrap())
          .take(list_len)
          .collect::<Vec<_>>();
        list.reverse();

        self.push(Expr {
          val: ExprKind::List(list),
          debug_data: DebugData {
            ingredients: Some(vec![expr]),
            source_file: expr.debug_data.source_file,
            span: expr.debug_data.span,
          },
        })
      }
      ExprKind::Fn(_) => Ok(()),
      _ => self.push(expr),
    }
  }

  pub fn eval_string(&mut self, line: &str) -> Result<(), EvalError> {
    let lexer = Lexer::new(line);
    let parser = Parser::new(lexer, interner().get_or_intern("internal"));
    // TODO: It might be time to add a proper EvalError enum.
    let exprs = parser.parse().map_err(|e| EvalError {
      expr: None,
      kind: EvalErrorKind::ParseError,
    })?;

    self.eval(exprs)
  }

  pub fn eval(&mut self, exprs: Vec<Expr>) -> Result<(), EvalError> {
    let mut clone = self.clone();
    let result = exprs.into_iter().try_for_each(|expr| clone.eval_expr(expr));

    self.sources = clone.sources;

    // TODO: Why tf are we doing this again? This is so dumb and I hate that I made it work like this.
    // We need to figure out a better way of reverting the state after an error instead of conditionally
    // assigning if there isn't an error
    match result {
      Ok(x) => {
        self.stack = clone.stack;
        self.scopes = clone.scopes;
        self.debug = clone.debug;

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
