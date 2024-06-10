use core::fmt;
use std::mem;

use crate::expr::{Expr, ExprKind};

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct JournalEntry {
  pub index: usize,
  pub ops: Vec<JournalOp>,
  pub scope: usize,
  pub scoped: bool,
}

impl fmt::Display for JournalEntry {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    core::iter::once("")
      .chain(core::iter::repeat(", "))
      .zip(self.ops.iter())
      .try_for_each(|(sep, op)| {
        if f.alternate() {
          write!(f, "{sep}{op:#}")
        } else {
          write!(f, "{sep}{op}")
        }
      })
  }
}

impl JournalEntry {
  pub fn new(
    index: usize,
    ops: Vec<JournalOp>,
    scope: usize,
    scoped: bool,
  ) -> Self {
    Self {
      index,
      ops,
      scope,
      scoped,
    }
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

impl fmt::Display for JournalOp {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if f.alternate() {
      match self {
        Self::Call(call) => write!(f, "call({call})"),
        Self::FnCall(fn_call) => write!(f, "fn({fn_call})"),
        Self::Push(push) => write!(f, "push({push})"),
        Self::Pop(pop) => write!(f, "pop({pop})"),
        _ => write!(f, ""),
      }
    } else {
      match self {
        Self::Call(call) => write!(f, "{call}"),
        Self::FnCall(fn_call) => write!(f, "{fn_call}"),
        Self::Push(push) => write!(f, "{push}"),
        Self::Pop(pop) => write!(f, "{pop}"),
        _ => write!(f, ""),
      }
    }
  }
}

impl JournalOp {
  pub fn is_stack_based(&self) -> bool {
    matches!(self, Self::Push(_) | Self::Pop(_))
  }

  pub fn expr(&self) -> Option<&Expr> {
    match self {
      Self::Call(expr) => Some(expr),
      Self::FnCall(expr) => Some(expr),
      Self::Push(expr) => Some(expr),
      Self::Pop(expr) => Some(expr),

      _ => None,
    }
  }
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
      for op in self.ops.iter() {
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

  pub fn ops(&self) -> &[JournalOp] {
    &self.ops
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

  pub fn all_entries(&self) -> Vec<JournalEntry> {
    self.entries(self.len())
  }

  pub fn entries(&self, max_commits: usize) -> Vec<JournalEntry> {
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

    for (i, op) in self.ops.iter().enumerate().skip(start) {
      match op {
        JournalOp::Commit => {
          entries.push(JournalEntry::new(
            i,
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

    entries.into_iter().rev().collect::<Vec<_>>()
  }

  pub fn len(&self) -> usize {
    self.ops.len()
  }

  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  pub fn construct_to(&self, index: usize) -> Vec<Expr> {
    let mut stack: Vec<Expr> = Vec::new();
    for (i, op) in self.ops.iter().enumerate() {
      if i == index {
        break;
      }

      match op {
        JournalOp::Push(expr) => stack.push(expr.clone()),
        JournalOp::Pop(_) => {
          stack.pop();
        }
        JournalOp::FnCall(expr) => {
          if let ExprKind::Symbol(symbol) = expr.kind {
            let len = stack.len();
            match symbol.as_str() {
              "swap" => stack.swap(len - 1, len - 2),
              "rot" => {
                stack.swap(len - 1, len - 3);
                stack.swap(len - 2, len - 3);
              }
              _ => {}
            }
          }
        }

        _ => {}
      };
    }

    stack
  }
}
