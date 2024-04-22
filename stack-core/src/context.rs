use core::fmt;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
  chain::Chain,
  engine::{RunError, RunErrorReason},
  expr::Expr,
  journal::{Journal, JournalOp},
  scope::{Scanner, Scope},
  source::Source,
  symbol::Symbol,
  vec_one::VecOne,
};

// TODO: This API could be a lot nicer.

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Context {
  stack: Vec<Expr>,
  lets: Vec<HashMap<Symbol, Expr>>,
  scopes: VecOne<Scope>,
  journal: Option<Journal>,
  sources: HashMap<Symbol, Rc<Source>>,
}

impl fmt::Display for Context {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Stack: [")?;

    // self.stack.iter().enumerate().try_for_each(|(i, expr)| {
    //   if i == self.stack.len() - 1 {
    //     write!(f, "{expr:#}")
    //   } else {
    //     write!(f, "{expr:#}, ")
    //   }
    // })?;

    core::iter::once("")
      .chain(core::iter::repeat(" "))
      .zip(self.stack.iter())
      .try_for_each(|(sep, x)| write!(f, "{sep}{x:#}"))?;

    write!(f, "]")?;

    // if !self.scopes.is_empty() {
    //   writeln!(f, "Scope:")?;

    //   let layer = self.scopes.last().unwrap();
    //   let items = layer.items.len();
    //   for (item_i, (key, value)) in
    //     layer.items.iter().sorted_by_key(|(s, _)| *s).enumerate()
    //   {
    //     if item_i == items - 1 {
    //       write!(
    //         f,
    //         " + {}: {}",
    //         interner().resolve(key),
    //         match value.borrow().val() {
    //           Some(expr) => expr.to_string(),
    //           None => "None".to_owned(),
    //         }
    //       )?;
    //     } else {
    //       writeln!(
    //         f,
    //         " + {}: {}",
    //         interner().resolve(key),
    //         match value.borrow().val() {
    //           Some(expr) => expr.to_string(),
    //           None => "None".to_owned(),
    //         }
    //       )?;
    //     }
    //   }
    // }

    if let Some(journal) = self.journal() {
      let journal = journal.to_string();

      if !journal.is_empty() {
        write!(f, "\n\n")?;
        write!(f, "{}", journal)?;
      }
    }

    Ok(())
  }
}

impl Context {
  #[inline]
  pub fn new() -> Self {
    Self {
      stack: Vec::new(),
      lets: Vec::new(),
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
  pub fn with_journal(mut self, journal: bool) -> Self {
    self.journal = if journal { Some(Journal::new()) } else { None };
    self
  }

  #[inline]
  pub fn add_source(&mut self, source: Rc<Source>) {
    self.sources.insert(*source.name(), source);
  }

  #[inline]
  pub fn source(&mut self, name: &Symbol) -> Option<&Rc<Source>> {
    self.sources.get(name)
  }

  #[inline]
  pub fn sources(
    &self,
  ) -> std::collections::hash_map::Iter<'_, Symbol, Rc<Source>> {
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

  pub fn stack_push(&mut self, expr: Expr) -> Result<(), RunError> {
    let expr = if expr.kind.is_function() {
      let mut scanner = Scanner::new(self.scopes.last().duplicate());

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

    if let Some(journal) = self.journal_mut() {
      journal.op(JournalOp::Push(expr.clone()));
    }

    self.stack.push(expr);
    // TODO: I don't think we need to commit after each push.
    // self.journal.commit();

    Ok(())
  }

  pub fn stack_pop(&mut self, expr: &Expr) -> Result<Expr, RunError> {
    match self.stack.pop() {
      Some(expr) => {
        if let Some(journal) = self.journal_mut() {
          journal.op(JournalOp::Pop(expr.clone()));
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
  pub fn def_scope_item(&mut self, symbol: Symbol, value: Expr) {
    let layer = self.scopes.last_mut();

    layer.define(symbol, value);
    // Remove the item from our let block since we've defined our own
    self.let_remove(symbol);
  }

  pub fn set_scope_item(
    &mut self,
    symbol: Symbol,
    expr: Expr,
  ) -> Result<(), RunError> {
    let layer = self.scopes.last_mut();

    match layer.set(symbol, expr.clone()) {
      Ok(_) => Ok(()),
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
    self.scopes.push(scope);
  }

  #[inline]
  pub fn pop_scope(&mut self) {
    self.scopes.try_pop();
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

  #[inline]
  pub fn let_remove(&mut self, name: Symbol) -> Option<Expr> {
    self.lets.last_mut().and_then(|x| x.remove(&name))
  }
}
