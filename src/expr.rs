use core::{cmp::Ordering, fmt, iter, num::FpCategory};

use lasso::{Resolver, Spur};

use crate::interner::interner;

#[derive(Debug, Clone)]
pub enum Expr {
  Nil,

  Boolean(bool),
  Integer(i64),
  Float(f64),

  Pointer(usize),

  List(Vec<Expr>),
  String(Spur),

  Lazy(Box<Expr>),
  Call(Spur),

  /// Boolean denotes whether to create a new scope.
  Fn(bool),
}

impl Expr {
  pub fn is_truthy(&self) -> bool {
    match self.to_boolean() {
      Some(Self::Boolean(x)) => x,
      _ => false,
    }
  }

  pub const fn is_nil(&self) -> bool {
    matches!(*self, Expr::Nil)
  }

  pub fn is_function(&self) -> bool {
    self.create_fn_scope().is_some()
  }

  pub fn create_fn_scope(&self) -> Option<bool> {
    match self {
      Expr::List(list) => list.first().and_then(|x| match x {
        Expr::Fn(scope) => Some(*scope),
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

      Self::Pointer(_) => Type::Pointer,

      Self::List(list) => {
        Type::List(list.iter().map(|expr| expr.type_of()).collect::<Vec<_>>())
      }
      Self::String(_) => Type::String,

      Self::Lazy(x) => x.type_of(),
      Self::Call(_) => Type::Call,

      Self::Fn(_) => Type::FnScope,
    }
  }

  pub fn coerce_same(&self, other: &Self) -> Option<(Self, Self)> {
    match self {
      x @ Self::Boolean(_) => Some(x.clone()).zip(other.to_boolean()),
      x @ Self::Integer(_) => Some(x.clone()).zip(other.to_integer()),
      x @ Self::Float(_) => Some(x.clone()).zip(other.to_float()),

      x @ Self::Pointer(_) => Some(x.clone()).zip(other.to_pointer()),

      _ => None,
    }
  }

  pub fn coerce_same_float(&self, other: &Self) -> Option<(Self, Self)> {
    match (self, other) {
      (lhs @ Self::Float(_), rhs) => Some(lhs.clone()).zip(rhs.to_float()),
      (lhs, rhs @ Self::Float(_)) => lhs.to_float().zip(Some(rhs.clone())),
      _ => self.coerce_same(other),
    }
  }

  pub fn to_boolean(&self) -> Option<Self> {
    match self {
      Self::Nil => Some(Self::Boolean(false)),

      x @ Self::Boolean(_) => Some(x.clone()),
      Self::Integer(x) => Some(Self::Boolean(*x != 0)),

      _ => None,
    }
  }

  pub fn to_integer(&self) -> Option<Self> {
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

      Self::Pointer(x) => {
        if *x < i64::MAX as usize {
          Some(Self::Integer(*x as i64))
        } else {
          None
        }
      }

      // Self::String(x) => x.parse().ok().map(Self::Integer),
      _ => None,
    }
  }

  pub fn to_float(&self) -> Option<Self> {
    match self {
      Self::Integer(x) => Some(Self::Float(*x as f64)),
      x @ Self::Float(_) => Some(x.clone()),

      // Self::String(x) => x.parse().ok().map(Self::Float),
      _ => None,
    }
  }

  pub fn to_pointer(&self) -> Option<Self> {
    match self {
      // TODO: Should nil be usable as a null pointer?
      Self::Nil => Some(Self::Pointer(0)),
      Self::Integer(x) => {
        if *x >= 0 {
          Some(Self::Pointer(*x as usize))
        } else {
          None
        }
      }

      x @ Self::Pointer(_) => Some(x.clone()),

      _ => None,
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

impl PartialEq for Expr {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      // Same types.
      (Self::Nil, Self::Nil) => true,

      (Self::Boolean(lhs), Self::Boolean(rhs)) => lhs == rhs,
      (Self::Integer(lhs), Self::Integer(rhs)) => lhs == rhs,
      (Self::Float(lhs), Self::Float(rhs)) => lhs == rhs,

      (Self::Pointer(lhs), Self::Pointer(rhs)) => lhs == rhs,

      (Self::List(lhs), Self::List(rhs)) => lhs == rhs,
      (Self::String(lhs), Self::String(rhs)) => lhs == rhs,

      (Self::Lazy(lhs), Self::Lazy(rhs)) => lhs == rhs,
      (Self::Call(lhs), Self::Call(rhs)) => lhs == rhs,

      (Self::Fn(lhs), Self::Fn(rhs)) => lhs == rhs,

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

      (lhs @ Self::Pointer(_), rhs) => match rhs.to_pointer() {
        Some(rhs) => *lhs == rhs,
        None => false,
      },

      _ => false,
    }
  }
}

impl PartialOrd for Expr {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    match (self, other) {
      // Same types.
      (Self::Nil, Self::Nil) => Some(Ordering::Equal),
      (Self::Integer(lhs), Self::Integer(rhs)) => lhs.partial_cmp(rhs),
      (Self::Float(lhs), Self::Float(rhs)) => lhs.partial_cmp(rhs),

      (Self::Pointer(lhs), Self::Pointer(rhs)) => lhs.partial_cmp(rhs),

      (Self::List(lhs), Self::List(rhs)) => lhs.partial_cmp(rhs),
      (Self::String(lhs), Self::String(rhs)) => lhs.partial_cmp(rhs),

      (Self::Lazy(lhs), Self::Lazy(rhs)) => lhs.partial_cmp(rhs),
      (Self::Call(lhs), Self::Call(rhs)) => lhs.partial_cmp(rhs),

      (Self::Fn(lhs), Self::Fn(rhs)) => lhs.partial_cmp(rhs),

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

      (lhs @ Self::Pointer(_), rhs) => match rhs.to_pointer() {
        Some(rhs) => lhs.partial_cmp(&rhs),
        None => None,
      },

      _ => None,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
  Nil,

  Boolean,
  Integer,
  Float,

  Pointer,

  List(Vec<Self>),
  String,

  Call,

  FnScope,
  ScopePush,
  ScopePop,

  Any,
  Set(Vec<Self>),
}

impl fmt::Display for Type {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Nil => f.write_str("nil"),

      Self::Boolean => f.write_str("boolean"),
      Self::Integer => f.write_str("integer"),
      Self::Float => f.write_str("float"),

      Self::Pointer => f.write_str("pointer"),

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
      Self::String => f.write_str("string"),

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
    }
  }
}

#[doc(hidden)]
pub struct Display<'a> {
  resolver: &'a dyn Resolver<Spur>,
  expr: &'a Expr,
}

impl fmt::Display for Display<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self.expr {
      Expr::Nil => f.write_str("nil"),

      Expr::Boolean(x) => fmt::Display::fmt(x, f),
      Expr::Integer(x) => fmt::Display::fmt(x, f),
      Expr::Float(x) => fmt::Display::fmt(x, f),

      Expr::Pointer(x) => {
        f.write_str("*")?;
        fmt::Display::fmt(x, f)
      }

      Expr::List(x) => {
        f.write_str("(")?;

        iter::once("")
          .chain(iter::repeat(" "))
          .zip(x.iter())
          .try_for_each(|(s, x)| {
            f.write_str(s)?;
            fmt::Display::fmt(&x.display(self.resolver), f)
          })?;

        f.write_str(")")
      }
      Expr::String(x) => write!(f, "\"{}\"", self.resolver.resolve(x)),

      Expr::Lazy(x) => {
        f.write_str("'")?;
        fmt::Display::fmt(&x.display(self.resolver), f)
      }
      Expr::Call(x) => f.write_str(self.resolver.resolve(x)),

      Expr::Fn(x) => {
        f.write_str("fn")?;

        f.write_str("(")?;
        fmt::Display::fmt(x, f)?;
        f.write_str(")")
      }
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  use test_case::test_case;

  #[test_case(Expr::Nil => Some(Expr::Boolean(false)))]
  #[test_case(Expr::Boolean(false) => Some(Expr::Boolean(false)))]
  #[test_case(Expr::Boolean(true) => Some(Expr::Boolean(true)))]
  #[test_case(Expr::Integer(0) => Some(Expr::Boolean(false)))]
  #[test_case(Expr::Integer(1) => Some(Expr::Boolean(true)))]
  #[test_case(Expr::Integer(i64::MIN) => Some(Expr::Boolean(true)))]
  #[test_case(Expr::Integer(i64::MAX) => Some(Expr::Boolean(true)))]
  #[test_case(Expr::Float(0.0) => None)]
  #[test_case(Expr::Float(1.0) => None)]
  #[test_case(Expr::Float(f64::MIN) => None)]
  #[test_case(Expr::Float(f64::MAX) => None)]
  #[test_case(Expr::Float(f64::NEG_INFINITY) => None)]
  #[test_case(Expr::Float(f64::INFINITY) => None)]
  #[test_case(Expr::Float(f64::NAN) => None)]
  #[test_case(Expr::Pointer(0) => None)]
  #[test_case(Expr::Pointer(1) => None)]
  fn to_boolean(expr: Expr) -> Option<Expr> {
    expr.to_boolean()
  }

  #[test_case(Expr::Nil => None)]
  #[test_case(Expr::Boolean(false) => Some(Expr::Integer(0)))]
  #[test_case(Expr::Boolean(true) => Some(Expr::Integer(1)))]
  #[test_case(Expr::Integer(0) => Some(Expr::Integer(0)))]
  #[test_case(Expr::Integer(1) => Some(Expr::Integer(1)))]
  #[test_case(Expr::Integer(i64::MIN) => Some(Expr::Integer(i64::MIN)))]
  #[test_case(Expr::Integer(i64::MAX) => Some(Expr::Integer(i64::MAX)))]
  #[test_case(Expr::Float(f64::MIN) => None)]
  #[test_case(Expr::Float(f64::MAX) => None)]
  #[test_case(Expr::Float(f64::NEG_INFINITY) => None)]
  #[test_case(Expr::Float(f64::INFINITY) => None)]
  #[test_case(Expr::Float(f64::NAN) => None)]
  #[test_case(Expr::Float(0.0) => Some(Expr::Integer(0)))]
  #[test_case(Expr::Float(1.0) => Some(Expr::Integer(1)))]
  #[test_case(Expr::Pointer(0) => Some(Expr::Integer(0)))]
  #[test_case(Expr::Pointer(1) => Some(Expr::Integer(1)))]
  fn to_integer(expr: Expr) -> Option<Expr> {
    expr.to_integer()
  }

  #[test_case(Expr::Nil => None)]
  #[test_case(Expr::Boolean(false) => None)]
  #[test_case(Expr::Boolean(true) => None)]
  #[test_case(Expr::Integer(0) => Some(Expr::Float(0.0)))]
  #[test_case(Expr::Integer(1) => Some(Expr::Float(1.0)))]
  #[test_case(Expr::Integer(i64::MIN) => Some(Expr::Float(i64::MIN as f64)))]
  #[test_case(Expr::Integer(i64::MAX) => Some(Expr::Float(i64::MAX as f64)))]
  #[test_case(Expr::Float(f64::MIN) => Some(Expr::Float(f64::MIN)))]
  #[test_case(Expr::Float(f64::MAX) => Some(Expr::Float(f64::MAX)))]
  #[test_case(Expr::Float(f64::NEG_INFINITY) => Some(Expr::Float(f64::NEG_INFINITY)))]
  #[test_case(Expr::Float(f64::INFINITY) => Some(Expr::Float(f64::INFINITY)))]
  // NOTE: NaN cannot be equality checked.
  // #[test_case(Expr::Float(f64::NAN) => Some(Expr::Float(f64::NAN)))]
  #[test_case(Expr::Float(0.0) => Some(Expr::Float(0.0)))]
  #[test_case(Expr::Float(1.0) => Some(Expr::Float(1.0)))]
  #[test_case(Expr::Pointer(0) => None)]
  #[test_case(Expr::Pointer(1) => None)]
  fn to_float(expr: Expr) -> Option<Expr> {
    expr.to_float()
  }

  #[test_case(Expr::Nil => Some(Expr::Pointer(0)))]
  #[test_case(Expr::Boolean(false) => None)]
  #[test_case(Expr::Boolean(true) => None)]
  #[test_case(Expr::Integer(0) => Some(Expr::Pointer(0)))]
  #[test_case(Expr::Integer(1) => Some(Expr::Pointer(1)))]
  #[test_case(Expr::Integer(i64::MIN) => None)]
  #[test_case(Expr::Integer(i64::MAX) => Some(Expr::Pointer(i64::MAX as usize)))]
  #[test_case(Expr::Float(f64::MIN) => None)]
  #[test_case(Expr::Float(f64::MAX) => None)]
  #[test_case(Expr::Float(f64::NEG_INFINITY) => None)]
  #[test_case(Expr::Float(f64::INFINITY) => None)]
  #[test_case(Expr::Float(f64::NAN) => None)]
  #[test_case(Expr::Float(0.0) => None)]
  #[test_case(Expr::Float(1.0) => None)]
  #[test_case(Expr::Pointer(0) => Some(Expr::Pointer(0)))]
  #[test_case(Expr::Pointer(1) => Some(Expr::Pointer(1)))]
  fn to_pointer(expr: Expr) -> Option<Expr> {
    expr.to_pointer()
  }
}
