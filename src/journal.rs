use std::mem;

use crate::Expr;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum JournalOp {
  Call(Expr),
  Push(Expr),
  Pop(Expr),
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct Journal {
  states: Vec<Vec<JournalOp>>,
  current: Vec<JournalOp>,
}

impl Journal {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn new_op(&mut self, op: JournalOp) {
    self.current.push(op);
  }

  pub fn submit(&mut self) -> usize {
    self.states.push(mem::take(&mut self.current));
    self.states.len() - 1
  }
}
