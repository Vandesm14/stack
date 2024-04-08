use core::fmt;
use itertools::Itertools;
use std::mem;
use termion::color;

use crate::Expr;

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
struct JournalEntry {
  ops: Vec<JournalOp>,
  scope: usize,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum JournalOp {
  Call(Expr),
  FnCall(Expr),
  Push(Expr),
  Pop(Expr),
  Commit,
  ScopeChange(usize),
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct Journal {
  ops: Vec<JournalOp>,
  current: Vec<JournalOp>,
  commits: Vec<usize>,
}

impl fmt::Display for Journal {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let entries = self.entries(10);
    if !entries.is_empty() {
      write!(f, "\n\nStack History:\n")?;
    }

    for entry in entries {
      let mut line = String::new();
      let mut should_print = false;
      for op in entry.ops {
        if !line.is_empty() {
          line.push(' ');
        }

        match op {
          JournalOp::Call(call) => {
            line.push_str(&format!("{}", color::Fg(color::White)));
            line.push_str(&format!("{}", call));

            line.push_str(&format!("{}", color::Fg(color::White)));
            line.push_str(" |");

            should_print = true;
          }
          JournalOp::FnCall(fn_call) => {
            line.push_str(&format!("{}", color::Fg(color::Yellow)));
            line.push_str(&format!("{}", fn_call));

            line.push_str(&format!("{}", color::Fg(color::White)));
            line.push_str(" |");

            should_print = true;
          }
          JournalOp::Push(push) => {
            line.push_str(&format!("{}", color::Fg(color::Green)));
            line.push_str(&format!("{}", push));

            should_print = true;
          }
          JournalOp::Pop(pop) => {
            line.push_str(&format!("{}", color::Fg(color::Red)));
            line.push_str(&format!("{}", pop));

            should_print = true;
          }
          _ => {}
        }
      }

      if should_print {
        line.push_str(&format!("{}", color::Fg(color::Reset)));
        write!(f, " {}* ", "  ".repeat(entry.scope))?;
        writeln!(f, "{}", line)?;
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

  pub fn clear(&mut self) {
    self.ops.clear();
    self.current.clear();
    self.commits.clear();
  }

  fn entries(&self, max_commits: usize) -> Vec<JournalEntry> {
    let mut entries: Vec<JournalEntry> = Vec::new();
    let mut entry = JournalEntry::default();

    let start = match max_commits >= self.commits.len() {
      false => self
        .commits
        .get(self.commits.len() - max_commits - 1)
        .map(|index| index + 1)
        .unwrap_or(0),
      true => 0,
    };

    let mut scope = 0;
    for op in self.ops.iter().skip(start) {
      match op {
        JournalOp::Commit => {
          entries.push(entry.clone());
          entry = JournalEntry::default();
          entry.scope = scope;
        }
        JournalOp::ScopeChange(scope_op) => {
          scope = *scope_op;
          entry.scope = scope;
        }
        op => entry.ops.push(op.clone()),
      }
    }

    entries.into_iter().rev().collect_vec()
  }
}
