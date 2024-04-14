use std::collections::HashMap;

use crate::{
  engine::{RunError, RunErrorReason},
  expr::{Expr, Symbol},
};

// TODO: This API could be a lot nicer.

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Context {
  stack: Vec<Expr>,
  lets: Vec<HashMap<Symbol, Expr>>,
}

impl Context {
  #[inline]
  pub const fn new() -> Self {
    Self {
      stack: Vec::new(),
      lets: Vec::new(),
    }
  }

  #[inline]
  pub fn with_stack_capacity(mut self, capacity: usize) -> Self {
    self.stack = Vec::with_capacity(capacity);
    self
  }

  #[inline]
  pub fn stack(&self) -> &[Expr] {
    &self.stack
  }

  #[inline]
  pub fn stack_mut(&mut self) -> &mut Vec<Expr> {
    &mut self.stack
  }

  #[inline]
  pub fn stack_push(&mut self, expr: Expr) {
    self.stack.push(expr);
  }

  #[inline]
  pub fn stack_pop(&mut self, expr: &Expr) -> Result<Expr, RunError> {
    self.stack.pop().ok_or_else(|| RunError {
      reason: RunErrorReason::StackUnderflow,
      context: self.clone(),
      expr: expr.clone(),
    })
  }

  #[inline]
  pub fn let_push(&mut self) {
    self.lets.push(HashMap::new());
  }

  #[inline]
  pub fn let_pop(&mut self) -> Option<HashMap<Symbol, Expr>> {
    self.lets.pop()
  }

  #[inline]
  pub fn let_get(&self, name: Symbol) -> Option<&Expr> {
    self.lets.iter().rev().find_map(|x| x.get(&name))
  }

  #[inline]
  pub fn let_set(&mut self, name: Symbol, expr: Expr) -> Option<Expr> {
    self.lets.last_mut().and_then(|x| x.insert(name, expr))
  }
}
