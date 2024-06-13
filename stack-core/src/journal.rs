use core::fmt;

use crate::expr::{Expr, ExprKind};

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct JournalEntry {
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
  pub fn new(ops: Vec<JournalOp>, scope: usize, scoped: bool) -> Self {
    Self { ops, scope, scoped }
  }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum JournalOp {
  Call(Expr),
  FnCall(Expr),
  Push(Expr),
  Pop(Expr),

  /// bool determines if the function is scoped or not
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
  scope: usize,
  scoped: Vec<bool>,

  entries: Vec<JournalEntry>,

  size: Option<usize>,
}

impl fmt::Display for Journal {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    use yansi::Paint;
    if !self.entries.is_empty() {
      writeln!(f, "Stack History (most recent first):")?;
    }

    for entry in self
      .entries
      .iter()
      .rev()
      .take(self.size.unwrap_or(self.entries.len()))
    {
      let mut line = String::new();
      for op in entry.ops.iter() {
        if !line.is_empty() {
          line.push(' ');
        }

        match op {
          JournalOp::Call(x) => {
            line.push_str(&format!(
              "{}",
              if f.alternate() { x.white() } else { x.new() }
            ));
          }
          JournalOp::FnCall(x) => {
            line.push_str(&format!(
              "{}",
              if f.alternate() { x.yellow() } else { x.new() }
            ));
          }
          JournalOp::Push(x) => {
            line.push_str(&format!(
              "{}",
              if f.alternate() { x.green() } else { x.new() }
            ));
          }
          JournalOp::Pop(x) => {
            line.push_str(&format!(
              "{}",
              if f.alternate() { x.red() } else { x.new() }
            ));
          }
          _ => {}
        }
      }

      let bullet_symbol = match entry.scoped {
        true => format!("{}*", "  ".repeat(entry.scope)),
        false => {
          format!("{}!", "  ".repeat(entry.scope))
        }
      };
      write!(f, " {} ", bullet_symbol)?;
      writeln!(f, "{}", line)?;
    }

    Ok(())
  }
}

impl Default for Journal {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl Journal {
  #[inline]
  pub fn new() -> Self {
    Self {
      ops: Vec::new(),
      scope: 0,
      scoped: vec![true],

      entries: Vec::new(),
      size: None,
    }
  }

  #[inline]
  pub const fn with_size(mut self, size: usize) -> Self {
    self.size = Some(size);
    self
  }

  pub fn ops(&self) -> &[JournalOp] {
    &self.ops
  }

  pub fn op(&mut self, op: JournalOp) {
    match op {
      JournalOp::Commit => {
        self.commit();
      }
      JournalOp::FnStart(is_scoped) => {
        self.scope += 1;
        self.scoped.push(is_scoped);
      }
      JournalOp::FnEnd => {
        self.scope = self.scope.saturating_sub(1);
        self.scoped.pop();
      }

      op => self.ops.push(op.clone()),
    }
  }

  pub fn commit(&mut self) {
    if !self.ops.is_empty() {
      self.entries.push(JournalEntry {
        ops: self.ops.drain(..).collect(),
        scope: self.scope,
        scoped: *self.scoped.last().unwrap_or(&true),
      });
    }
  }

  pub fn entries(&self) -> &Vec<JournalEntry> {
    &self.entries
  }

  pub fn len(&self) -> usize {
    self.ops.len()
  }

  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  /// Constructing from a higher to a lower index (backwards).
  pub fn construct_to_from(
    &self,
    stack: &mut Vec<Expr>,
    to: usize,
    from: usize,
  ) {
    for entry in self
      .entries
      .iter()
      .rev()
      .skip((self.entries.len() - 1) - from)
      .take(from - to)
    {
      for op in entry.ops.iter().rev() {
        match op {
          JournalOp::Push(_) => {
            stack.pop();
          }
          JournalOp::Pop(expr) => stack.push(expr.clone()),
          JournalOp::FnCall(expr) => {
            if let ExprKind::Symbol(symbol) = expr.kind {
              let len = stack.len();
              match symbol.as_str() {
                "swap" => stack.swap(len - 1, len - 2),
                "rot" => {
                  stack.swap(len - 2, len - 3);
                  stack.swap(len - 1, len - 3);
                }
                _ => {}
              }
            }
          }

          _ => {}
        };
      }
    }
  }

  /// Constructing from a lower to a higher index.
  pub fn construct_from_to(
    &self,
    stack: &mut Vec<Expr>,
    from: usize,
    to: usize,
  ) {
    for entry in self.entries.iter().skip(from + 1).take(to - from) {
      for op in entry.ops.iter() {
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
    }
  }

  pub fn construct_to(&self, index: usize) -> Vec<Expr> {
    let mut stack = Vec::new();
    self.construct_from_to(&mut stack, 0, index);

    stack
  }

  // pub fn trim_to(mut self, index: usize) -> Self {
  //   let total = self.ops.len();
  //   self.ops = self
  //     .ops
  //     .into_iter()
  //     .take(self.commits.get(index + 1).copied().unwrap_or(total))
  //     .collect();
  //   self.recount_commits();
  //   self
  // }
}
