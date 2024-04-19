use std::collections::HashMap;

use crate::{
  engine::{RunError, RunErrorReason},
  expr::{Expr, Symbol},
  journal::Journal,
  scope::{Scanner, Scope},
};

// TODO: This API could be a lot nicer.

#[derive(Debug, Clone, PartialEq)]
pub struct Context {
  stack: Vec<Expr>,
  lets: Vec<HashMap<Symbol, Expr>>,
  scopes: Vec<Scope>,
  journal: Option<Journal>,
}

impl Default for Context {
  fn default() -> Self {
    Self::new()
  }
}

impl Context {
  #[inline]
  pub fn new() -> Self {
    Self {
      stack: Vec::new(),
      lets: Vec::new(),
      scopes: vec![Scope::new()],
      journal: None,
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
  pub fn journal(&self) -> &Option<Journal> {
    &self.journal
  }

  #[inline]
  pub fn journal_mut(&mut self) -> &mut Option<Journal> {
    &mut self.journal
  }

  #[inline]
  pub fn stack_push(&mut self, expr: Expr) -> Result<(), RunError> {
    let expr = if expr.kind.is_function() {
      let mut scanner = Scanner::new(self.scopes.last().unwrap().duplicate());

      match scanner.scan(expr.clone()) {
        Ok(expr) => expr,
        Err(reason) => {
          return Err(RunError {
            reason,
            context: self.clone(),
            expr,
          })
        }
      }
    } else {
      expr
    };

    // TODO: implement
    // if self.debug {
    //   self.journal.op(JournalOp::Push(expr.clone()));
    // }

    self.stack.push(expr);
    // TODO: I don't think we need to commit after each push.
    // self.journal.commit();

    Ok(())
  }

  pub fn scope_item(&self, symbol: Symbol) -> Option<Expr> {
    self.scopes.last().and_then(|layer| layer.get_val(symbol))
  }

  pub fn def_scope_item(&mut self, symbol: Symbol, value: Expr) {
    if let Some(layer) = self.scopes.last_mut() {
      layer.define(symbol, value).unwrap();
    }
  }

  pub fn set_scope_item(
    &mut self,
    symbol: Symbol,
    expr: Expr,
  ) -> Result<(), RunError> {
    if let Some(layer) = self.scopes.last_mut() {
      match layer.set(symbol, expr.clone()) {
        Ok(_) => Ok(()),
        Err(reason) => Err(RunError {
          reason,
          context: self.clone(),
          expr,
        }),
      }
    } else {
      panic!("context has no scopes!");
    }
  }

  pub fn remove_scope_item(&mut self, symbol: Symbol) {
    if let Some(layer) = self.scopes.last_mut() {
      layer.remove(symbol);
    }
  }

  pub fn push_scope(&mut self, scope: Scope) {
    self.scopes.push(scope);
  }

  pub fn pop_scope(&mut self) {
    self.scopes.pop();

    debug_assert!(!self.scopes.is_empty());
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
