use core::{cmp::Ordering, fmt, ops};
use std::rc::Rc;

use internment::Intern;

use crate::{intrinsic::Intrinsic, lexer::Span, scope::Scope, source::Source};

pub type Symbol = Intern<String>;

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
    write!(f, "{}", self.kind)
  }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum ExprKind {
  #[default]
  Nil,
  Error(Box<Expr>),

  Boolean(bool),
  Integer(i64),
  Float(f64),
  String(String),

  Symbol(Symbol),

  Lazy(Box<Expr>),
  List(Vec<Expr>),

  Intrinsic(Intrinsic),

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

  #[inline]
  pub fn is_function(&self) -> bool {
    match self {
      Self::List(list) => list
        .first()
        .map(|x| {
          matches!(
            x,
            Expr {
              kind: Self::Fn(_),
              ..
            }
          )
        })
        .unwrap_or(false),
      _ => false,
    }
  }

  #[inline]
  pub fn fn_symbol(&self) -> Option<&FnIdent> {
    match self {
      Self::List(list) => list.first().and_then(|x| match &x.kind {
        Self::Fn(scope) => Some(scope),
        _ => None,
      }),
      _ => None,
    }
  }

  #[inline]
  pub fn fn_body(&self) -> Option<&[Expr]> {
    match self {
      Self::List(list) => list.first().and_then(|x| match x.kind {
        Self::Fn(_) => Some(&list[1..]),
        _ => None,
      }),
      _ => None,
    }
  }

  #[inline]
  pub const fn unlazy(&self) -> &ExprKind {
    match self {
      ExprKind::Lazy(x) => x.kind.unlazy(),
      x => x,
    }
  }

  #[inline]
  pub fn unlazy_mut(&mut self) -> &mut ExprKind {
    match self {
      ExprKind::Lazy(x) => x.kind.unlazy_mut(),
      x => x,
    }
  }
}

impl PartialOrd for ExprKind {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    match (self, other) {
      (Self::Nil, Self::Nil) => Some(Ordering::Equal),

      (Self::Boolean(lhs), Self::Boolean(rhs)) => lhs.partial_cmp(rhs),
      (Self::Integer(lhs), Self::Integer(rhs)) => lhs.partial_cmp(rhs),
      (Self::Float(lhs), Self::Float(rhs)) => lhs.partial_cmp(rhs),
      (Self::String(lhs), Self::String(rhs)) => lhs.partial_cmp(rhs),

      (Self::Symbol(lhs), Self::Symbol(rhs)) => {
        if lhs == rhs {
          Some(Ordering::Equal)
        } else {
          None
        }
      }

      (Self::Lazy(lhs), Self::Lazy(rhs)) => lhs.partial_cmp(rhs),
      (Self::List(_), Self::List(_)) => None,

      (Self::Intrinsic(_), Self::Intrinsic(_)) => None,

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
    match self {
      Self::Nil => write!(f, "nil"),
      Self::Error(x) => write!(f, "error({x})"),

      Self::Boolean(x) => write!(f, "{x}"),
      Self::Integer(x) => write!(f, "{x}"),
      Self::Float(x) => write!(f, "{x}"),
      Self::String(x) => {
        if f.alternate() {
          write!(f, "{x}")
        } else {
          write!(f, "\"{x}\"")
        }
      }

      Self::Symbol(x) => write!(f, "{}", x.as_ref()),

      Self::Lazy(x) => write!(f, "'{}", x.kind),
      Self::List(x) => {
        write!(f, "(")?;

        core::iter::once("")
          .chain(core::iter::repeat(" "))
          .zip(x.iter())
          .try_for_each(|(sep, x)| write!(f, "{sep}{}", x.kind))?;

        write!(f, ")")
      }

      Self::Intrinsic(x) => write!(f, "{x}"),

      Self::Fn(x) => write!(f, "{x}"),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FnIdent {
  pub scoped: bool,
  pub scope: Scope,
}

impl fmt::Display for FnIdent {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "fn(..)")
  }
}

#[derive(Clone)]
pub enum ExprInfo {
  /// Comes from a [`Source`].
  Source { source: Rc<dyn Source>, span: Span },
  /// Comes from evaluation.
  Runtime { components: Vec<Expr> },
}

impl fmt::Debug for ExprInfo {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Source { source, span } => f
        .debug_struct("Source")
        .field("source", &source.path())
        .field("span", span)
        .finish(),
      Self::Runtime { components } => f
        .debug_struct("Runtime")
        .field("components", components)
        .finish(),
    }
  }
}

impl fmt::Display for ExprInfo {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      // TODO: This should output line and column numbers instead of the span.
      Self::Source { source, span } => write!(
        f,
        "'{}' at {}:{}",
        &source.content()[span.start..span.end],
        source.path().display(),
        span
      ),
      Self::Runtime { components } => core::iter::once("")
        .chain(core::iter::repeat(" "))
        .zip(components.iter())
        .filter_map(|(sep, x)| x.info.as_ref().map(|x| (sep, x)))
        .try_for_each(|(sep, x)| write!(f, "{sep}{}", x)),
    }
  }
}
