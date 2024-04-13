use core::{any::Any, cmp::Ordering, fmt, ops};
use std::rc::Rc;

use internment::Intern;

use crate::{intrinsic::Intrinsic, lexer::Span, source::Source};

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

#[derive(Default)]
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

  UserData(Box<dyn UserData>),

  Intrinsic(Intrinsic),
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
  pub const fn is_falsy(&self) -> bool {
    !self.is_truthy()
  }
}

impl Clone for ExprKind {
  fn clone(&self) -> Self {
    match self {
      Self::Nil => Self::Nil,
      Self::Error(x) => Self::Error(x.clone()),

      Self::Boolean(x) => Self::Boolean(*x),
      Self::Integer(x) => Self::Integer(*x),
      Self::Float(x) => Self::Float(*x),
      Self::String(x) => Self::String(x.clone()),

      Self::Symbol(x) => Self::Symbol(*x),

      Self::Lazy(x) => Self::Lazy(x.clone()),
      Self::List(x) => Self::List(x.clone()),

      Self::UserData(x) => Self::UserData(UserData::clone(&**x)),

      Self::Intrinsic(x) => Self::Intrinsic(*x),
    }
  }
}

impl PartialEq for ExprKind {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Nil, Self::Nil) => true,

      (Self::Boolean(lhs), Self::Boolean(rhs)) => lhs == rhs,
      (Self::Integer(lhs), Self::Integer(rhs)) => lhs == rhs,
      (Self::Float(lhs), Self::Float(rhs)) => lhs == rhs,
      (Self::String(lhs), Self::String(rhs)) => lhs == rhs,

      (Self::Symbol(lhs), Self::Symbol(rhs)) => lhs == rhs,

      (Self::Lazy(lhs), Self::Lazy(rhs)) => lhs == rhs,
      (Self::List(lhs), Self::List(rhs)) => *lhs == *rhs,

      (Self::UserData(lhs), Self::UserData(rhs)) => lhs.eq(&**rhs),

      (Self::Intrinsic(lhs), Self::Intrinsic(rhs)) => lhs == rhs,

      _ => false,
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

      (Self::UserData(lhs), Self::UserData(rhs)) => lhs.partial_cmp(&**rhs),

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

impl fmt::Debug for ExprKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Nil => write!(f, "Nil"),
      Self::Error(x) => f.debug_tuple("Error").field(x).finish(),

      Self::Boolean(x) => f.debug_tuple("Boolean").field(x).finish(),
      Self::Integer(x) => f.debug_tuple("Integer").field(x).finish(),
      Self::Float(x) => f.debug_tuple("Float").field(x).finish(),
      Self::String(x) => f.debug_tuple("String").field(x).finish(),

      Self::Symbol(x) => f.debug_tuple("Symbol").field(x).finish(),

      Self::Lazy(x) => f.debug_tuple("Lazy").field(x).finish(),
      Self::List(x) => f.debug_tuple("List").field(x).finish(),

      Self::UserData(_) => f.debug_tuple("UserData").field(&"..").finish(),

      Self::Intrinsic(x) => f.debug_tuple("Intrinsic").field(x).finish(),
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

      Self::UserData(x) => x.fmt(f),

      Self::Intrinsic(x) => write!(f, "{x}"),
    }
  }
}

/// Implemented by types that are provided by embedders.
pub trait UserData: Any {
  /// See [`Clone::clone`].
  #[must_use]
  fn clone(&self) -> Box<dyn UserData>;

  /// See [`PartialEq::eq`].
  #[must_use]
  fn eq(&self, other: &dyn UserData) -> bool;

  /// See [`PartialOrd::partial_cmp`].
  #[must_use]
  fn partial_cmp(&self, other: &dyn UserData) -> Option<Ordering>;

  /// See [`Display::fmt`].
  ///
  /// [`Display`]: fmt::Display
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
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
