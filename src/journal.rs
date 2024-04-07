use core::fmt;
use std::mem;
use termion::color;

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

impl fmt::Display for Journal {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for (i, state) in self.states.iter().enumerate() {
      if i > 0 {
        writeln!(f)?;
      }

      for (op_i, op) in state.iter().enumerate() {
        if op_i > 0 {
          write!(f, " ")?;
        }

        match op {
          JournalOp::Call(call) => {
            write!(f, "{}", color::Fg(color::Yellow))?;
            write!(f, "{}", call)?;
            write!(f, " |")?;
          }
          JournalOp::Push(push) => {
            write!(f, "{}", color::Fg(color::Green))?;
            write!(f, "{}", push)?;
          }
          JournalOp::Pop(pop) => {
            write!(f, "{}", color::Fg(color::Red))?;
            write!(f, "{}", pop)?;
          }
        }
        write!(f, "{}", color::Fg(color::Reset))?;
      }
    }

    Ok(())
  }
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
