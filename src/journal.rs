use core::fmt;
use itertools::Itertools;
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
  commits: Vec<usize>,
}

impl fmt::Display for Journal {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut do_space = false;

    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();

    let max_commits = 20;
    let start = match max_commits >= self.commits.len() {
      false => self
        .commits
        .get(self.commits.len() - max_commits - 1)
        .map(|index| index + 1)
        .unwrap_or(0),
      true => 0,
    };

    for op in self.ops.iter().skip(start) {
      if !do_space {
        line.push_str(" * ");
      }

      do_space |= true;

      match op {
        JournalOp::Call(call) => {
          line.push_str(&format!("{}", color::Fg(color::Yellow)));
          line.push_str(&format!("{}", call));
          line.push_str(" |");
        }
        JournalOp::Push(push) => {
          line.push_str(&format!("{}", color::Fg(color::Green)));
          line.push_str(&format!("{}", push));
        }
        JournalOp::Pop(pop) => {
          line.push_str(&format!("{}", color::Fg(color::Red)));
          line.push_str(&format!("{}", pop));
        }
        JournalOp::Commit => {
          do_space = false;
          lines.push(line.clone());
          line = String::new();
        }
      }
      line.push_str(&format!("{}", color::Fg(color::Reset)));

      if do_space {
        line.push(' ');
      }
    }

    lines = lines.into_iter().rev().collect_vec();

    if !lines.is_empty() {
      write!(f, "\n\nStack History:\n")?;
    }

    // dbg!(self.ops.clone());

    write!(f, "{}", lines.join("\n"))?;

    Ok(())
  }
}

impl Journal {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn op(&mut self, op: JournalOp) {
    if matches!(op, JournalOp::Commit) {
      return self.commit();
    }

    self.current.push(op);
  }

  pub fn commit(&mut self) {
    if !self.current.is_empty() {
      self.ops.extend(mem::take(&mut self.current));
      self.commits.push(self.ops.len());
      self.ops.push(JournalOp::Commit);
    }
  }
}
