use core::{
  any::Any, cell::RefCell, cmp::Ordering, fmt, iter, num::FpCategory,
};
use std::{fmt::Debug, rc::Rc};

use lasso::Spur;
use termion::color;

use crate::{interner::interner, Scope, Span};

#[derive(Clone, PartialEq)]
pub struct FnSymbol {
  pub scoped: bool,
  pub scope: Scope,
}

impl fmt::Debug for FnSymbol {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("FnSymbol")
      .field("scoped", &self.scoped)
      .finish()
  }
}

#[derive(Debug, Clone)]
pub enum ExprKind {
  Nil,

  Boolean(bool),
  Integer(i64),
  Float(f64),

  String(Spur),
  List(Vec<Expr>),

  Lazy(Box<Expr>),
  Call(Spur),

  /// Boolean denotes whether to create a new scope.
  Fn(FnSymbol),

  UserData(Rc<RefCell<dyn Any>>),
}

impl ExprKind {
  pub fn unlazy(&self) -> &ExprKind {
    match self {
      ExprKind::Lazy(x) => x.val.unlazy(),
      x => x,
    }
  }

  pub fn unlazy_mut(&mut self) -> &mut ExprKind {
    match self {
      ExprKind::Lazy(x) => x.val.unlazy_mut(),
      x => x,
    }
  }

  pub fn is_truthy(&self) -> bool {
    match self.to_boolean() {
      Some(Self::Boolean(x)) => x,
      _ => false,
    }
  }

  pub const fn is_nil(&self) -> bool {
    matches!(*self, Self::Nil)
  }

  pub fn is_function(&self) -> bool {
    match self {
      Self::List(list) => list
        .first()
        .map(|x| {
          matches!(
            x,
            Expr {
              val: Self::Fn(_),
              ..
            }
          )
        })
        .unwrap_or(false),
      _ => false,
    }
  }

  pub fn fn_symbol(&self) -> Option<&FnSymbol> {
    match self {
      Self::List(list) => list.first().and_then(|x| match &x.val {
        Self::Fn(scope) => Some(scope),
        _ => None,
      }),
      _ => None,
    }
  }

  pub fn fn_body(&self) -> Option<&[Expr]> {
    match self {
      Self::List(list) => list.first().and_then(|x| match x.val {
        Self::Fn(_) => Some(&list[1..]),
        _ => None,
      }),
      _ => None,
    }
  }

  pub fn type_of(&self) -> Type {
    match self {
      Self::Nil => Type::Nil,

      Self::Boolean(_) => Type::Boolean,
      Self::Integer(_) => Type::Integer,
      Self::Float(_) => Type::Float,

      Self::String(_) => Type::String,

      Self::List(list) => Type::List(
        list
          .iter()
          .map(|expr| expr.val.type_of())
          .collect::<Vec<_>>(),
      ),

      Self::Lazy(x) => x.val.type_of(),
      Self::Call(_) => Type::Call,

      Self::Fn(_) => Type::FnScope,

      Self::UserData(_) => Type::UserData,
    }
  }

  pub fn coerce_same(&self, other: &Self) -> Option<(ExprKind, ExprKind)> {
    match self {
      x @ Self::Boolean(_) => Some(x.clone()).zip(other.to_boolean()),
      x @ Self::Integer(_) => Some(x.clone()).zip(other.to_integer()),
      x @ Self::Float(_) => Some(x.clone()).zip(other.to_float()),
      _ => None,
    }
  }

  pub fn coerce_same_float(
    &self,
    other: &Self,
  ) -> Option<(ExprKind, ExprKind)> {
    match (self, other) {
      (lhs @ Self::Float(_), rhs) => Some(lhs.clone()).zip(rhs.to_float()),
      (lhs, rhs @ Self::Float(_)) => lhs.to_float().zip(Some(rhs.clone())),
      _ => self.coerce_same(other),
    }
  }

  pub fn to_boolean(&self) -> Option<ExprKind> {
    match self {
      Self::Nil => Some(Self::Boolean(false)),

      x @ Self::Boolean(_) => Some(x.clone()),
      Self::Integer(x) => Some(Self::Boolean(*x != 0)),

      _ => None,
    }
  }

  pub fn to_integer(&self) -> Option<ExprKind> {
    match self {
      Self::Boolean(x) => Some(Self::Integer(if *x { 1 } else { 0 })),
      x @ Self::Integer(_) => Some(x.clone()),
      Self::Float(x) => {
        let x = x.floor();

        match x.classify() {
          FpCategory::Zero => Some(Self::Integer(0)),
          FpCategory::Normal => {
            if x >= i64::MIN as f64 && x <= i64::MAX as f64 {
              Some(Self::Integer(x as i64))
            } else {
              None
            }
          }
          _ => None,
        }
      }

      // Self::String(x) => x.parse().ok().map(Self::Integer),
      _ => None,
    }
  }

  pub fn to_float(&self) -> Option<ExprKind> {
    match self {
      Self::Integer(x) => Some(Self::Float(*x as f64)),
      x @ Self::Float(_) => Some(x.clone()),

      // Self::String(x) => x.parse().ok().map(Self::Float),
      _ => None,
    }
  }

  pub fn into_expr(self, debug_data: DebugData) -> Expr {
    Expr {
      val: self,
      debug_data,
    }
  }

  // TODO: These might make more sense as intrinsics, since they might be too
  //       complicated for coercions.

  // pub const fn to_list(&self) -> Option<Self> {
  //   match self {
  //     x @ Self::List(_) => Some(x.clone()),
  //     // TODO: Implement a way to convert a string into a list of its characters
  //     //       in the language itself.
  //     Self::String(x) => Some(Self::List(
  //       x.bytes()
  //         .map(|x| x as i64)
  //         .map(Self::Integer)
  //         .map(Expr::new)
  //         .collect_vec()
  //         .into(),
  //     )),

  //     x => Some(Self::List([Expr::new(x.clone())].into())),
  //   }
  // }

  // pub const fn to_string(&self) -> Option<Self> {
  //   match self {
  //     Self::List(x) => {
  //       x.iter()
  //         .map(|Expr(expr)| expr.borrow())
  //         .map(|expr| match *expr {
  //           Self::Integer(x) => if x >= u8::MIN as i64 && x <= u8::MAX as i64 {
  //             Ok(x as u8)
  //           } else {
  //             Err(())
  //           },
  //           _ => Err(()),
  //         })
  //         .try_collect::<_, Vec<_>, _>()
  //         .ok()
  //         .and_then(|bytes| core::str::from_utf8(&bytes).ok())
  //         .map(|x| ExprKind::String(x.into()))
  //     },

  //     _ => None,
  //   }
  // }
}

impl PartialEq for ExprKind {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      // Same types.
      (Self::Nil, Self::Nil) => true,

      (Self::Boolean(lhs), Self::Boolean(rhs)) => lhs == rhs,
      (Self::Integer(lhs), Self::Integer(rhs)) => lhs == rhs,
      (Self::Float(lhs), Self::Float(rhs)) => lhs == rhs,

      (Self::String(lhs), Self::String(rhs)) => lhs == rhs,

      (Self::List(lhs), Self::List(rhs)) => {
        if lhs.len() != rhs.len() {
          return false;
        }

        lhs.iter().zip(rhs).all(|(lhs, rhs)| lhs.val == rhs.val)
      }

      (Self::Lazy(lhs), Self::Lazy(rhs)) => lhs.val == rhs.val,
      (Self::Call(lhs), Self::Call(rhs)) => lhs == rhs,

      (Self::Fn(lhs), Self::Fn(rhs)) => lhs == rhs,

      (Self::UserData(lhs), Self::UserData(rhs)) => {
        core::ptr::addr_eq(Rc::as_ptr(lhs), Rc::as_ptr(rhs))
      }

      // Different types.
      (lhs @ Self::Boolean(_), rhs) => match rhs.to_boolean() {
        Some(rhs) => *lhs == rhs,
        None => false,
      },
      (lhs, rhs @ Self::Float(_)) => match lhs.to_float() {
        Some(lhs) => lhs == *rhs,
        None => false,
      },
      (lhs @ Self::Integer(_), rhs) => match rhs.to_integer() {
        Some(rhs) => *lhs == rhs,
        None => false,
      },
      (lhs @ Self::Float(_), rhs) => match rhs.to_float() {
        Some(rhs) => *lhs == rhs,
        None => false,
      },

      _ => false,
    }
  }
}

impl PartialOrd for ExprKind {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    match (self, other) {
      // Same types.
      (Self::Nil, Self::Nil) => Some(Ordering::Equal),
      (Self::Integer(lhs), Self::Integer(rhs)) => lhs.partial_cmp(rhs),
      (Self::Float(lhs), Self::Float(rhs)) => lhs.partial_cmp(rhs),

      (Self::List(lhs), Self::List(rhs)) => lhs.partial_cmp(rhs),
      (Self::String(lhs), Self::String(rhs)) => lhs.partial_cmp(rhs),

      (Self::Lazy(lhs), Self::Lazy(rhs)) => lhs.partial_cmp(rhs),
      (Self::Call(lhs), Self::Call(rhs)) => lhs.partial_cmp(rhs),

      (Self::Fn(lhs), Self::Fn(rhs)) => lhs.scoped.partial_cmp(&rhs.scoped),

      // Different types.
      (lhs @ Self::Boolean(_), rhs) => match rhs.to_boolean() {
        Some(rhs) => lhs.partial_cmp(&rhs),
        None => None,
      },
      (lhs, rhs @ Self::Float(_)) => match lhs.to_float() {
        Some(lhs) => lhs.partial_cmp(rhs),
        None => None,
      },
      (lhs @ Self::Integer(_), rhs) => match rhs.to_integer() {
        Some(rhs) => lhs.partial_cmp(&rhs),
        None => None,
      },
      (lhs @ Self::Float(_), rhs) => match rhs.to_float() {
        Some(rhs) => lhs.partial_cmp(&rhs),
        None => None,
      },

      _ => None,
    }
  }
}

impl fmt::Display for ExprKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Nil => f.write_str("nil"),

      Self::Boolean(x) => fmt::Display::fmt(x, f),
      Self::Integer(x) => fmt::Display::fmt(x, f),
      Self::Float(x) => fmt::Display::fmt(x, f),

      Self::String(x) => write!(f, "\"{}\"", interner().resolve(x)),

      Self::List(x) => {
        f.write_str("(")?;

        iter::once("")
          .chain(iter::repeat(" "))
          .zip(x.iter())
          .try_for_each(|(s, x)| {
            f.write_str(s)?;
            fmt::Display::fmt(x, f)
          })?;

        f.write_str(")")
      }

      Self::Lazy(x) => {
        f.write_str("'")?;
        fmt::Display::fmt(x, f)
      }
      Self::Call(x) => f.write_str(interner().resolve(x)),

      Self::Fn(_) => f.write_str("fn"),

      Self::UserData(_) => f.write_str("userdata"),
    }
  }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct DebugData {
  pub source_file: Option<Spur>,
  pub span: Option<Span>,
}

impl DebugData {
  pub fn new(source_file: Option<Spur>, span: Option<Span>) -> Self {
    Self { source_file, span }
  }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Expr {
  pub val: ExprKind,
  pub debug_data: DebugData,
}

impl fmt::Display for Expr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.val)
  }
}

impl From<Expr> for ExprKind {
  fn from(value: Expr) -> Self {
    value.val
  }
}

impl Expr {
  pub fn into_expr_kind(self) -> ExprKind {
    self.into()
  }

  pub fn to_pretty_string(&self) -> String {
    let reset = color::Fg(color::Reset);
    let result = match &self.val {
      ExprKind::Nil => format!("{}nil", color::Fg(color::White)),

      ExprKind::Boolean(x) => {
        format!("{}{}", color::Fg(color::Yellow), x)
      }
      ExprKind::Integer(x) => format!("{}{}", color::Fg(color::Yellow), x),
      ExprKind::Float(x) => format!("{}{}", color::Fg(color::Yellow), x),

      ExprKind::String(x) => {
        format!("{}\"{}\"", color::Fg(color::Green), interner().resolve(x))
      }

      ExprKind::List(x) => {
        let mut string = String::new();
        string.push('(');

        iter::once("")
          .chain(iter::repeat(" "))
          .zip(x.iter())
          .for_each(|(s, x)| {
            string.push_str(s);
            string.push_str(&x.to_pretty_string())
          });

        string.push(')');

        string
      }

      ExprKind::Lazy(x) => {
        format!("'{}", x)
      }
      ExprKind::Call(x) => {
        format!("{}{}", color::Fg(color::Blue), interner().resolve(&x))
      }

      ExprKind::Fn(_) => format!("{}fn", color::Fg(color::Blue)),

      ExprKind::UserData(_) => format!("{}userdata", color::Fg(color::Blue)),
    };

    format!("{}{}", result, reset)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
  Nil,

  Boolean,
  Integer,
  Float,

  Pointer,

  String,

  List(Vec<Self>),

  Call,

  FnScope,
  ScopePush,
  ScopePop,

  Any,
  Set(Vec<Self>),

  UserData,
}

impl fmt::Display for Type {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Nil => f.write_str("nil"),

      Self::Boolean => f.write_str("boolean"),
      Self::Integer => f.write_str("integer"),
      Self::Float => f.write_str("float"),

      Self::Pointer => f.write_str("pointer"),

      Self::String => f.write_str("string"),

      Self::List(list) => {
        f.write_str("(")?;

        iter::once("")
          .chain(iter::repeat(" "))
          .zip(list.iter())
          .try_for_each(|(sep, ty)| {
            f.write_str(sep)?;
            fmt::Display::fmt(ty, f)
          })?;

        f.write_str(")")
      }

      Self::Call => f.write_str("call"),

      Self::FnScope => f.write_str("fn"),
      Self::ScopePush => f.write_str("scope_push"),
      Self::ScopePop => f.write_str("scope_pop"),

      Self::Any => f.write_str("any"),
      Self::Set(set) => {
        f.write_str("[")?;

        iter::once("")
          .chain(iter::repeat(" "))
          .zip(set.iter())
          .try_for_each(|(sep, ty)| {
            f.write_str(sep)?;
            fmt::Display::fmt(ty, f)
          })?;

        f.write_str("]")
      }

      Self::UserData => f.write_str("userdata"),
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  use test_case::test_case;

  #[test_case(ExprKind::Nil => Some(ExprKind::Boolean(false)))]
  #[test_case(ExprKind::Boolean(false) => Some(ExprKind::Boolean(false)))]
  #[test_case(ExprKind::Boolean(true) => Some(ExprKind::Boolean(true)))]
  #[test_case(ExprKind::Integer(0) => Some(ExprKind::Boolean(false)))]
  #[test_case(ExprKind::Integer(1) => Some(ExprKind::Boolean(true)))]
  #[test_case(ExprKind::Integer(i64::MIN) => Some(ExprKind::Boolean(true)))]
  #[test_case(ExprKind::Integer(i64::MAX) => Some(ExprKind::Boolean(true)))]
  #[test_case(ExprKind::Float(0.0) => None)]
  #[test_case(ExprKind::Float(1.0) => None)]
  #[test_case(ExprKind::Float(f64::MIN) => None)]
  #[test_case(ExprKind::Float(f64::MAX) => None)]
  #[test_case(ExprKind::Float(f64::NEG_INFINITY) => None)]
  #[test_case(ExprKind::Float(f64::INFINITY) => None)]
  #[test_case(ExprKind::Float(f64::NAN) => None)]
  fn to_boolean(expr: ExprKind) -> Option<ExprKind> {
    expr.to_boolean()
  }

  #[test_case(ExprKind::Nil => None)]
  #[test_case(ExprKind::Boolean(false) => Some(ExprKind::Integer(0)))]
  #[test_case(ExprKind::Boolean(true) => Some(ExprKind::Integer(1)))]
  #[test_case(ExprKind::Integer(0) => Some(ExprKind::Integer(0)))]
  #[test_case(ExprKind::Integer(1) => Some(ExprKind::Integer(1)))]
  #[test_case(ExprKind::Integer(i64::MIN) => Some(ExprKind::Integer(i64::MIN)))]
  #[test_case(ExprKind::Integer(i64::MAX) => Some(ExprKind::Integer(i64::MAX)))]
  #[test_case(ExprKind::Float(f64::MIN) => None)]
  #[test_case(ExprKind::Float(f64::MAX) => None)]
  #[test_case(ExprKind::Float(f64::NEG_INFINITY) => None)]
  #[test_case(ExprKind::Float(f64::INFINITY) => None)]
  #[test_case(ExprKind::Float(f64::NAN) => None)]
  #[test_case(ExprKind::Float(0.0) => Some(ExprKind::Integer(0)))]
  #[test_case(ExprKind::Float(1.0) => Some(ExprKind::Integer(1)))]
  fn to_integer(expr: ExprKind) -> Option<ExprKind> {
    expr.to_integer()
  }

  #[test_case(ExprKind::Nil => None)]
  #[test_case(ExprKind::Boolean(false) => None)]
  #[test_case(ExprKind::Boolean(true) => None)]
  #[test_case(ExprKind::Integer(0) => Some(ExprKind::Float(0.0)))]
  #[test_case(ExprKind::Integer(1) => Some(ExprKind::Float(1.0)))]
  #[test_case(ExprKind::Integer(i64::MIN) => Some(ExprKind::Float(i64::MIN as f64)))]
  #[test_case(ExprKind::Integer(i64::MAX) => Some(ExprKind::Float(i64::MAX as f64)))]
  #[test_case(ExprKind::Float(f64::MIN) => Some(ExprKind::Float(f64::MIN)))]
  #[test_case(ExprKind::Float(f64::MAX) => Some(ExprKind::Float(f64::MAX)))]
  #[test_case(ExprKind::Float(f64::NEG_INFINITY) => Some(ExprKind::Float(f64::NEG_INFINITY)))]
  #[test_case(ExprKind::Float(f64::INFINITY) => Some(ExprKind::Float(f64::INFINITY)))]
  // NOTE: NaN cannot be equality checked.
  // #[test_case(ExprKind::Float(f64::NAN) => Some(ExprKind::Float(f64::NAN)))]
  #[test_case(ExprKind::Float(0.0) => Some(ExprKind::Float(0.0)))]
  #[test_case(ExprKind::Float(1.0) => Some(ExprKind::Float(1.0)))]
  fn to_float(expr: ExprKind) -> Option<ExprKind> {
    expr.to_float()
  }
}
