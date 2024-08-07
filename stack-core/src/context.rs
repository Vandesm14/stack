use std::{cell::RefCell, collections::HashMap, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{
  chain::Chain,
  engine::{RunError, RunErrorReason},
  expr::{Expr, ExprKind},
  journal::{Journal, JournalOp},
  scope::{Scanner, Scope},
  source::Source,
  symbol::Symbol,
  vec_one::VecOne,
};

// TODO: This API could be a lot nicer.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Context {
  stack: Vec<Expr>,
  scopes: VecOne<Scope>,
  journal: Option<Journal>,
  sources: HashMap<Symbol, Source>,
}

impl Context {
  #[inline]
  pub fn new() -> Self {
    Self {
      stack: Vec::new(),
      scopes: VecOne::new(Scope::new()),
      journal: None,
      sources: HashMap::new(),
    }
  }

  #[inline]
  pub fn with_stack_capacity(mut self, capacity: usize) -> Self {
    self.stack = Vec::with_capacity(capacity);
    self
  }

  #[inline]
  pub fn with_journal(mut self, size: Option<usize>) -> Self {
    self.journal = Some(
      size
        .map(|size| Journal::new().with_size(size))
        .unwrap_or_default(),
    );
    self
  }

  #[inline]
  pub fn add_source(&mut self, source: Source) {
    self.sources.insert(Symbol::from_ref(source.name()), source);
  }

  #[inline]
  pub fn source(&mut self, name: &Symbol) -> Option<&Source> {
    self.sources.get(name)
  }

  #[inline]
  pub fn sources(
    &self,
  ) -> std::collections::hash_map::Iter<'_, Symbol, Source> {
    self.sources.iter()
  }

  #[inline]
  pub fn remove_source(&mut self, name: &Symbol) {
    self.sources.remove(name);
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

  pub fn scan_expr(&mut self, expr: Expr) -> Result<Expr, RunError> {
    if expr.kind.is_function() {
      let mut duplicate = self.scopes.last().duplicate();
      let mut scanner = Scanner::new(&mut duplicate);

      match scanner.scan(expr) {
        Ok(expr) => Ok(expr),
        Err((expr, reason)) => Err(RunError {
          reason,
          context: self.clone(),
          expr,
        }),
      }
    } else {
      Ok(expr)
    }
  }

  pub fn stack_push(&mut self, expr: Expr) -> Result<(), RunError> {
    let expr = self.scan_expr(expr)?;

    if let Some(journal) = self.journal_mut() {
      journal.push_op(JournalOp::Push(expr.clone()));
    }

    self.stack.push(expr);

    Ok(())
  }

  pub fn stack_silent_push(&mut self, expr: Expr) -> Result<(), RunError> {
    let expr = self.scan_expr(expr)?;

    self.stack.push(expr);

    Ok(())
  }

  pub fn stack_pop(&mut self, expr: &Expr) -> Result<Expr, RunError> {
    match self.stack.pop() {
      Some(expr) => {
        if let Some(journal) = self.journal_mut() {
          journal.push_op(JournalOp::Pop(expr.clone()));
        }
        Ok(expr)
      }
      None => Err(RunError {
        reason: RunErrorReason::StackUnderflow,
        context: self.clone(),
        expr: expr.clone(),
      }),
    }
  }

  pub fn stack_silent_pop(&mut self, expr: &Expr) -> Result<Expr, RunError> {
    match self.stack.pop() {
      Some(expr) => Ok(expr),
      None => Err(RunError {
        reason: RunErrorReason::StackUnderflow,
        context: self.clone(),
        expr: expr.clone(),
      }),
    }
  }

  #[inline]
  pub fn scope_item(&self, symbol: Symbol) -> Option<Expr> {
    self.scopes.last().get_val(symbol)
  }

  #[inline]
  pub fn scope_items(
    &self,
  ) -> impl Iterator<Item = (&Symbol, &Rc<RefCell<Chain<Option<Expr>>>>)> {
    self.scopes.last().items.iter()
  }

  #[inline]
  pub fn scope(&self) -> &Scope {
    self.scopes.last()
  }

  #[inline]
  pub fn scope_mut(&mut self) -> &mut Scope {
    self.scopes.last_mut()
  }

  #[inline]
  pub fn def_scope_item(&mut self, symbol: Symbol, value: Expr) {
    let layer = self.scopes.last_mut();
    let val = layer.define(symbol, value);

    if let Some(journal) = self.journal_mut() {
      journal.push_op(JournalOp::ScopeDef(
        symbol,
        val.borrow().val().unwrap_or(ExprKind::Nil.into()),
      ));
    }
  }

  pub fn set_scope_item(
    &mut self,
    symbol: Symbol,
    expr: Expr,
  ) -> Result<(), RunError> {
    let layer = self.scopes.last_mut();
    let old = layer.get_val(symbol);
    match layer.set(symbol, expr.clone()) {
      Ok(val) => {
        if let Some((journal, old)) = self.journal_mut().as_mut().zip(old) {
          journal.push_op(JournalOp::ScopeSet(
            symbol,
            old,
            val.borrow().val().unwrap_or(ExprKind::Nil.into()),
          ));
        }

        Ok(())
      }
      Err(reason) => Err(RunError {
        reason,
        context: self.clone(),
        expr,
      }),
    }
  }

  #[inline]
  pub fn remove_scope_item(&mut self, symbol: Symbol) {
    self.scopes.last_mut().remove(symbol);
  }

  #[inline]
  pub fn push_scope(&mut self, scope: Scope) {
    // if let Some(journal) = self.journal_mut() {
    //   for (key, val) in scope.items.iter() {
    //     journal.push_op(JournalOp::ScopeSet(
    //       *key,
    //       val.borrow().val().unwrap_or_else(|| ExprKind::Nil.into()),
    //     ))
    //   }
    // }
    self.scopes.push(scope);
  }

  #[inline]
  pub fn pop_scope(&mut self) {
    self.scopes.try_pop();
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_ser_and_de() {
    let mut context = Context::new();
    context.stack_push(ExprKind::Integer(2).into()).unwrap();
    context.def_scope_item(
      Symbol::from_ref("foo"),
      ExprKind::Symbol(Symbol::from_ref("bar")).into(),
    );

    let json = serde_json::to_string(&context).unwrap();
    let ser_context: Context = serde_json::from_str(json.as_str()).unwrap();

    assert_eq!(context, ser_context);
  }
}
