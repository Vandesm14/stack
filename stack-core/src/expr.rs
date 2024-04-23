use core::{cmp::Ordering, fmt, hash::Hash, ops};
use std::collections::HashMap;

use compact_str::CompactString;
use internment::Intern;
use yansi::Paint;

use crate::{lexer::Span, scope::Scope, source::Source, symbol::Symbol};

#[derive(Debug, Clone)]
pub struct Expr {
  pub kind: ExprKind,
  pub info: Option<ExprInfo>,
}

impl PartialEq for Expr {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.kind == other.kind
  }
}

impl PartialOrd for Expr {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    self.kind.partial_cmp(&other.kind)
  }
}

impl fmt::Display for Expr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if f.alternate() {
      write!(f, "{:#}", self.kind)
    } else {
      write!(f, "{}", self.kind)
    }
  }
}

#[derive(Debug, Clone)]
pub enum ExprKind {
  Nil,
  Error(Error),

  Boolean(bool),
  Integer(i64),
  Float(f64),
  String(CompactString),

  Symbol(Symbol),

  Lazy(Box<Expr>),
  List(Vec<Expr>),
  Record(HashMap<Symbol, Expr>),

  Fn(FnIdent),
}

impl ExprKind {
  #[inline]
  pub const fn is_nil(&self) -> bool {
    matches!(self, Self::Nil)
  }

  #[inline]
  pub const fn is_truthy(&self) -> bool {
    matches!(self, Self::Boolean(true))
  }

  pub fn is_function(&self) -> bool {
    match self {
      Self::List(list) => vec_is_function(list),
      _ => false,
    }
  }

  pub fn fn_symbol(&self) -> Option<&FnIdent> {
    match self {
      Self::List(list) => vec_fn_symbol(list),
      _ => None,
    }
  }

  pub fn fn_symbol_mut(&mut self) -> Option<&mut FnIdent> {
    match self {
      Self::List(list) => vec_fn_symbol_mut(list),
      _ => None,
    }
  }

  pub fn fn_body(&self) -> Option<&[Expr]> {
    match self {
      Self::List(list) => vec_fn_body(list),
      _ => None,
    }
  }

  pub const fn unlazy(&self) -> &ExprKind {
    match self {
      ExprKind::Lazy(x) => x.kind.unlazy(),
      x => x,
    }
  }

  pub fn unlazy_mut(&mut self) -> &mut ExprKind {
    match self {
      ExprKind::Lazy(x) => x.kind.unlazy_mut(),
      x => x,
    }
  }
}

impl PartialEq for ExprKind {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Nil, Self::Nil) => true,
      (Self::Error(lhs), Self::Error(rhs)) => lhs == rhs,

      (Self::Boolean(lhs), Self::Boolean(rhs)) => lhs == rhs,
      (Self::Integer(lhs), Self::Integer(rhs)) => lhs == rhs,
      (Self::Float(lhs), Self::Float(rhs)) => lhs == rhs,
      (Self::String(lhs), Self::String(rhs)) => lhs == rhs,

      (Self::Symbol(lhs), Self::Symbol(rhs)) => lhs == rhs,

      (Self::Lazy(lhs), Self::Lazy(rhs)) => lhs == rhs,
      (Self::List(lhs), Self::List(rhs)) => lhs == rhs,

      _ => false,
    }
  }
}

impl PartialOrd for ExprKind {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    match (self, other) {
      (Self::Nil, Self::Nil) => Some(Ordering::Equal),
      (Self::Error(lhs), Self::Error(rhs)) => {
        lhs.eq(rhs).then_some(Ordering::Equal)
      }

      (Self::Boolean(lhs), Self::Boolean(rhs)) => {
        lhs.eq(rhs).then_some(Ordering::Equal)
      }
      (Self::Integer(lhs), Self::Integer(rhs)) => lhs.partial_cmp(rhs),
      (Self::Float(lhs), Self::Float(rhs)) => lhs.partial_cmp(rhs),
      (Self::String(lhs), Self::String(rhs)) => {
        lhs.eq(rhs).then_some(Ordering::Equal)
      }

      (Self::Symbol(lhs), Self::Symbol(rhs)) => {
        lhs.eq(rhs).then_some(Ordering::Equal)
      }

      (Self::Lazy(lhs), Self::Lazy(rhs)) => lhs.partial_cmp(rhs),
      (Self::List(lhs), Self::List(rhs)) => {
        lhs.eq(rhs).then_some(Ordering::Equal)
      }

      _ => None,
    }
  }
}

impl ops::Add for ExprKind {
  type Output = Result<Self, (Self, Self)>;

  fn add(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (Self::Integer(lhs), Self::Integer(rhs)) => {
        Ok(Self::Integer(lhs.saturating_add(rhs)))
      }
      (Self::Float(lhs), Self::Float(rhs)) => Ok(Self::Float(lhs + rhs)),

      (lhs, rhs) => Err((lhs, rhs)),
    }
  }
}

impl ops::Sub for ExprKind {
  type Output = Result<Self, (Self, Self)>;

  fn sub(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (Self::Integer(lhs), Self::Integer(rhs)) => {
        Ok(Self::Integer(lhs.saturating_sub(rhs)))
      }
      (Self::Float(lhs), Self::Float(rhs)) => Ok(Self::Float(lhs - rhs)),

      (lhs, rhs) => Err((lhs, rhs)),
    }
  }
}

impl ops::Mul for ExprKind {
  type Output = Result<Self, (Self, Self)>;

  fn mul(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (Self::Integer(lhs), Self::Integer(rhs)) => {
        Ok(Self::Integer(lhs.saturating_mul(rhs)))
      }
      (Self::Float(lhs), Self::Float(rhs)) => Ok(Self::Float(lhs * rhs)),

      (lhs, rhs) => Err((lhs, rhs)),
    }
  }
}

impl ops::Div for ExprKind {
  type Output = Result<Self, (Self, Self)>;

  fn div(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (Self::Integer(lhs), Self::Integer(rhs)) => {
        Ok(Self::Integer(lhs.saturating_div(rhs)))
      }
      (Self::Float(lhs), Self::Float(rhs)) => Ok(Self::Float(lhs / rhs)),

      (lhs, rhs) => Err((lhs, rhs)),
    }
  }
}

impl ops::Rem for ExprKind {
  type Output = Result<Self, (Self, Self)>;

  fn rem(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (Self::Integer(lhs), Self::Integer(rhs)) => Ok(Self::Integer(lhs % rhs)),
      (Self::Float(lhs), Self::Float(rhs)) => Ok(Self::Float(lhs % rhs)),

      (lhs, rhs) => Err((lhs, rhs)),
    }
  }
}

impl fmt::Display for ExprKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // TODO: Is there a nicer way to do this that avoids the duplication?
    if f.alternate() {
      match self {
        Self::Nil => write!(f, "{}", "nil".green()),
        Self::Error(x) => write!(f, "error({})", x.to_string().red()),

        Self::Boolean(x) => write!(f, "{}", x.to_string().green()),
        Self::Integer(x) => write!(f, "{}", x.to_string().blue()),
        Self::Float(x) => write!(f, "{}", x.to_string().blue()),
        Self::String(x) => {
          write!(f, "{}{}{}", "\"".green(), x.green(), "\"".green(),)
        }

        Self::Symbol(x) => write!(f, "{}", x.as_str().blue()),

        Self::Lazy(x) => write!(f, "{}{x:#}", "'".yellow()),
        Self::List(x) => {
          write!(f, "{}", "(".yellow())?;

          core::iter::once("")
            .chain(core::iter::repeat(" "))
            .zip(x.iter())
            .try_for_each(|(sep, x)| write!(f, "{sep}{x:#}"))?;

          write!(f, "{}", ")".yellow())
        }
        Self::Record(x) => {
          write!(f, "{{")?;

          core::iter::once("")
            .chain(core::iter::repeat(" "))
            .zip(x.iter())
            .try_for_each(|(sep, (key, value))| {
              let key = Expr {
                info: None,
                kind: ExprKind::Symbol(*key),
              };
              write!(f, "{sep}{key:#}: {value:#}")
            })?;

          write!(f, "}}")
        }

        Self::Fn(x) => write!(f, "{}", x.to_string().yellow()),
      }
    } else {
      match self {
        Self::Nil => write!(f, "nil"),
        Self::Error(x) => write!(f, "error({x})"),

        Self::Boolean(x) => write!(f, "{x}"),
        Self::Integer(x) => write!(f, "{x}"),
        Self::Float(x) => write!(f, "{x}"),
        Self::String(x) => write!(f, "\"{x}\""),

        Self::Symbol(x) => write!(f, "{}", x.as_str()),

        Self::Lazy(x) => write!(f, "{x}"),
        Self::List(x) => {
          write!(f, "(")?;

          core::iter::once("")
            .chain(core::iter::repeat(" "))
            .zip(x.iter())
            .try_for_each(|(sep, x)| write!(f, "{sep}{x}"))?;

          write!(f, ")")
        }
        Self::Record(x) => {
          write!(f, "{{")?;

          core::iter::once("")
            .chain(core::iter::repeat(" "))
            .zip(x.iter())
            .try_for_each(|(sep, (key, value))| {
              write!(f, "{sep}{key}: {value}")
            })?;

          write!(f, "}}")
        }

        Self::Fn(x) => write!(f, "{x}"),
      }
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Error(Intern<ErrorInner>);

impl Error {
  /// Creates a [`Error`].
  #[inline]
  pub fn new(value: String) -> Self {
    Self(Intern::new(ErrorInner(value)))
  }

  /// Returns the <code>&[str]</code> for this [`Error`].
  #[inline]
  pub fn as_str(&self) -> &str {
    self.0 .0.as_str()
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "\"{}\"", self.as_str())
  }
}

#[derive(Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
struct ErrorInner(String);

impl fmt::Debug for ErrorInner {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "\"{}\"", self.0)
  }
}

impl fmt::Display for ErrorInner {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "\"{}\"", self.0)
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnIdent {
  pub scoped: bool,
  pub scope: Scope,
}

impl fmt::Display for FnIdent {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if self.scoped {
      write!(f, "fn")
    } else {
      write!(f, "fn!")
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprInfo {
  pub source: Source,
  pub span: Span,
}

impl fmt::Display for ExprInfo {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{}:{}",
      self.source.name(),
      self
        .source
        .location(self.span.start)
        .map(|x| x.to_string())
        .unwrap_or_else(|| "?:?".into())
    )
  }
}

pub fn vec_is_function(list: &[Expr]) -> bool {
  list
    .first()
    .map(|x| {
      matches!(
        x,
        Expr {
          kind: ExprKind::Fn(_),
          ..
        }
      )
    })
    .unwrap_or(false)
}

pub fn vec_fn_symbol(list: &[Expr]) -> Option<&FnIdent> {
  list.first().and_then(|x| match &x.kind {
    ExprKind::Fn(scope) => Some(scope),
    _ => None,
  })
}

pub fn vec_fn_symbol_mut(list: &mut [Expr]) -> Option<&mut FnIdent> {
  list.first_mut().and_then(|x| match &mut x.kind {
    ExprKind::Fn(scope) => Some(scope),
    _ => None,
  })
}

pub fn vec_fn_body(list: &[Expr]) -> Option<&[Expr]> {
  list.first().and_then(|x| match x.kind {
    ExprKind::Fn(_) => Some(&list[1..]),
    _ => None,
  })
}
