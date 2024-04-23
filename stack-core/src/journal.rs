use core::fmt;
use std::mem;

use crate::expr::Expr;

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

#[derive(Debug, Clone, PartialEq, PartialOrd)]
// TODO: implement this as a ring buffer with max_commits so we never go over
pub struct Journal {
  ops: Vec<JournalOp>,
  current: Vec<JournalOp>,
  commits: Vec<usize>,

  size: usize,
}

impl fmt::Display for Journal {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    use yansi::Paint;

    let entries = self.entries(self.size);
    if !entries.is_empty() {
      writeln!(f, "Stack History (most recent first):")?;
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
            line.push_str(&format!("{}", call.white()));

            should_print = true;
          }
          JournalOp::FnCall(fn_call) => {
            line.push_str(&format!("{}", fn_call.yellow()));

            should_print = true;
          }
          JournalOp::Push(push) => {
            line.push_str(&format!("{}", push.green()));

            should_print = true;
          }
          JournalOp::Pop(pop) => {
            line.push_str(&format!("{}", pop.red()));

            should_print = true;
          }
          _ => {}
        }
      }

      if should_print {
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

impl Default for Journal {
  #[inline]
  fn default() -> Self {
    Self::new(20)
  }
}

impl Journal {
  #[inline]
  pub const fn new(size: usize) -> Self {
    Self {
      commits: Vec::new(),
      current: Vec::new(),
      ops: Vec::new(),
      size,
    }
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
          // TODO: There was a bug where we were underflowing, meaning multiple FnEnd's existed.
          // I'm not sure why that could be the case, so I'm using `saturating_sub` to ignore
          // that case and prevent underflow.
          scope = scope.saturating_sub(1);
          scoped.pop();
        }
        op => ops.push(op.clone()),
      }
    }

    entries.into_iter().rev().collect::<Vec<_>>()
  }
}
