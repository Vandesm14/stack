use core::fmt;
use std::collections::HashMap;

use crate::{
  expr::{Expr, ExprKind},
  scope::Scope,
  symbol::Symbol,
};

#[derive(Debug, Clone, PartialEq)]
pub struct JournalEntry {
  pub ops: Vec<JournalOp>,
  pub scope_level: usize,
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
  pub fn new(ops: Vec<JournalOp>, scope_level: usize, scoped: bool) -> Self {
    Self {
      ops,
      scope_level,
      scoped,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum JournalOp {
  Call(Expr),
  SCall(Expr),

  FnCall(Expr),
  Push(Expr),
  Pop(Expr),

  ScopedFnStart(JournalScope),
  ScopelessFnStart,
  FnEnd(JournalScope),

  ScopeDef(Symbol, Expr),
  ScopeSet(Symbol, Expr, Expr),
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

pub type JournalScope = HashMap<Symbol, Expr>;

impl From<Scope> for JournalScope {
  fn from(value: Scope) -> Self {
    let iter = value.items.into_iter().map(|(key, value)| {
      (
        key,
        value.borrow().val().clone().unwrap_or(ExprKind::Nil.into()),
      )
    });

    JournalScope::from_iter(iter)
  }
}

#[derive(Debug, Clone, PartialEq)]
// TODO: implement this as a ring buffer with max_commits so we never go over
pub struct Journal {
  ops: Vec<JournalOp>,

  last_pop: Option<ExprKind>,
  last_push: Option<ExprKind>,

  entries: Vec<JournalEntry>,
  scope_levels: Vec<bool>,

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
        true => format!("{}*", "  ".repeat(entry.scope_level)),
        false => {
          format!("{}!", "  ".repeat(entry.scope_level))
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

      last_pop: None,
      last_push: None,

      entries: Vec::new(),
      scope_levels: vec![false],

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

  pub fn push_op(&mut self, op: JournalOp) {
    match &op {
      JournalOp::ScopedFnStart(..) => {
        self.scope_levels.push(true);
      }
      JournalOp::ScopelessFnStart => {
        self.scope_levels.push(false);
      }
      JournalOp::FnEnd(..) => {
        self.scope_levels.pop();
      }

      JournalOp::Push(expr) => {
        if let Some(last_pop) = &self.last_pop {
          if &expr.kind == last_pop {
            self.ops.pop();
            return;
          }
        }

        self.last_push = Some(expr.kind.clone());
      }
      JournalOp::Pop(expr) => {
        if let Some(last_push) = &self.last_push {
          if &expr.kind == last_push {
            self.ops.pop();
            return;
          }
        }

        self.last_pop = Some(expr.kind.clone());
      }

      _ => {}
    }

    self.ops.push(op)
  }

  pub fn commit(&mut self) {
    if !self.ops.is_empty() {
      self.entries.push(JournalEntry {
        ops: self.ops.drain(..).collect(),
        scope_level: self.scope_levels.len(),
        scoped: self.scope_levels.last().copied().unwrap_or_default(),
      });

      self.last_pop = None;
      self.last_push = None;
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

  fn construct_entry(
    &self,
    entry: &JournalEntry,
    stack: &mut Vec<Expr>,
    scopes: &mut Vec<JournalScope>,
  ) {
    for op in entry.ops.iter() {
      match op {
        JournalOp::ScopedFnStart(scope) => {
          scopes.push(scope.clone());
        }
        JournalOp::FnEnd(..) => {
          scopes.pop();
        }
        JournalOp::ScopeDef(key, value) => {
          let scope = scopes.last_mut();
          if let Some(scope) = scope {
            scope.insert(*key, value.clone());
          }
        }
        JournalOp::ScopeSet(key, _, value) => {
          let scope = scopes.last_mut();
          if let Some(scope) = scope {
            scope.insert(*key, value.clone());
          }
        }
        JournalOp::Push(expr) => stack.push(expr.clone()),
        JournalOp::Pop(_) => {
          stack.pop();
        }
        JournalOp::FnCall(expr) => {
          if let ExprKind::Symbol(symbol) = expr.kind {
            let len = stack.len();
            match symbol.as_str() {
              "swap" => {
                stack.swap(len - 1, len - 2);
              }
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

  fn unconstruct_entry(
    &self,
    entry: &JournalEntry,
    stack: &mut Vec<Expr>,
    scopes: &mut Vec<JournalScope>,
  ) {
    for op in entry.ops.iter().rev() {
      match op {
        JournalOp::ScopedFnStart(..) => {
          scopes.pop();
        }
        JournalOp::FnEnd(scope) => {
          scopes.push(scope.clone());
        }
        JournalOp::ScopeDef(key, ..) => {
          let scope = scopes.last_mut();
          if let Some(scope) = scope {
            scope.remove(key);
          }
        }
        JournalOp::ScopeSet(key, old_value, _) => {
          let scope = scopes.last_mut();
          if let Some(scope) = scope {
            scope.insert(*key, old_value.clone());
          }
        }
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

  /// Constructing from a higher to a lower index (backwards).
  pub fn construct_to_from(
    &self,
    stack: &mut Vec<Expr>,
    scopes: &mut Vec<JournalScope>,
    to: usize,
    from: usize,
  ) {
    let skip = (self.entries.len() - 1) - from;
    let take = from - to;
    for entry in self.entries.iter().rev().skip(skip).take(take) {
      self.unconstruct_entry(entry, stack, scopes);
    }
  }

  /// Constructing from a lower to a higher index.
  pub fn construct_from_to(
    &self,
    stack: &mut Vec<Expr>,
    scopes: &mut Vec<JournalScope>,
    from: usize,
    to: usize,
  ) {
    let skip = from + 1;
    let take = to - from;

    for entry in self.entries.iter().skip(skip).take(take) {
      self.construct_entry(entry, stack, scopes);
    }
  }

  pub fn construct_at_zero(
    &self,
    stack: &mut Vec<Expr>,
    scopes: &mut Vec<JournalScope>,
  ) {
    if let Some(entry) = self.entries.first() {
      self.construct_entry(entry, stack, scopes);
    }
  }

  pub fn construct_to(&self, index: usize) -> (Vec<Expr>, Vec<JournalScope>) {
    let mut stack = Vec::new();
    let mut scopes = vec![JournalScope::new()];
    self.construct_at_zero(&mut stack, &mut scopes);
    self.construct_from_to(&mut stack, &mut scopes, 0, index);

    (stack, scopes)
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
