use core::fmt;
use itertools::Itertools;
use std::mem;
use termion::color;

use crate::Expr;

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
struct JournalEntry {
  ops: Vec<JournalOp>,
  scope: usize,
  scoped: bool,
}

impl JournalEntry {
  fn new(ops: Vec<JournalOp>, scope: usize, scoped: bool) -> Self {
    Self { ops, scope, scoped }
  }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum JournalOp {
  Call(Expr),
  FnCall(Expr),
  Push(Expr),
  Pop(Expr),

  FnStart(bool),
  FnEnd,

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
    let entries = self.entries(20);
    if !entries.is_empty() {
      write!(f, "\n\nStack History (most recent first):\n")?;
    }

    let len = entries.len();
    for (i, entry) in entries.iter().enumerate() {
      let mut line = String::new();
      let mut should_print = false;
      for op in entry.ops.iter() {
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

        let bullet_symbol = match entry.scoped {
          true => format!("{}*", "  ".repeat(entry.scope)),
          false => {
            format!("{}!", "  ".repeat(entry.scope))
          }
        };
        write!(f, " {} ", bullet_symbol)?;

        if i != len - 1 {
          writeln!(f, "{}", line)?;
        } else {
          write!(f, "{}", line)?;
        }
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

    let start = match max_commits >= self.commits.len() {
      false => self
        .commits
        .get(self.commits.len() - max_commits - 1)
        .map(|index| index + 1)
        .unwrap_or(0),
      true => 0,
    };

    let mut scope = 0;
    let mut ops: Vec<JournalOp> = Vec::new();
    let mut scoped: Vec<bool> = vec![true];
    for op in self.ops.iter().skip(start) {
      match op {
        JournalOp::Commit => {
          entries.push(JournalEntry::new(
            ops,
            scope,
            *scoped.last().unwrap_or(&true),
          ));
          ops = Vec::new();
        }
        JournalOp::FnStart(is_scoped) => {
          scope += 1;
          scoped.push(*is_scoped);
        }
        JournalOp::FnEnd => {
          scope = scope.saturating_sub(1);
          scoped.pop();
        }
        op => ops.push(op.clone()),
      }
    }

    entries.into_iter().rev().collect_vec()
  }
}