use core::fmt;
use std::mem;
use termion::color;

use crate::Expr;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum JournalOp {
  Call(Expr),
  Push(Expr),
  Pop(Expr),
  Commit,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct Journal {
  ops: Vec<JournalOp>,
  current: Vec<JournalOp>,
}

impl fmt::Display for Journal {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut do_space = false;
    for (i, op) in self.ops.iter().enumerate() {
      if !do_space {
        if i != 0 {
          writeln!(f)?;
        }
        write!(f, " * ")?;
      }

      do_space |= true;

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
        JournalOp::Commit => {
          do_space = false;
        }
      }
      write!(f, "{}", color::Fg(color::Reset))?;

      if do_space {
        write!(f, " ")?;
      }
    }

    Ok(())
  }
}

impl Journal {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn op(&mut self, op: JournalOp) {
    self.current.push(op);
  }

  pub fn commit(&mut self) {
    self.ops.extend(mem::take(&mut self.current));
    self.ops.push(JournalOp::Commit);
  }
}
