use core::{
  any::Any, cell::RefCell, cmp::Ordering, fmt, iter, num::FpCategory,
};
use std::rc::Rc;

use lasso::Spur;

use crate::{interner::interner, FnSymbol, Type};

#[derive(Debug, Clone)]
pub enum ExprTree {
  Nil,

  Boolean(bool),
  Integer(i64),
  Float(f64),

  String(Spur),
  List(Vec<ExprTree>),

  Lazy(Box<ExprTree>),
  Call(Spur),

  /// Boolean denotes whether to create a new scope.
  Fn(FnSymbol),

  UserData(Rc<RefCell<dyn Any>>),
}

impl ExprTree {
  pub fn is_truthy(&self) -> bool {
    match self.to_boolean() {
      Some(Self::Boolean(x)) => x,
      _ => false,
    }
  }

  pub const fn is_nil(&self) -> bool {
    matches!(*self, ExprTree::Nil)
  }

  pub fn is_function(&self) -> bool {
    match self {
      ExprTree::List(list) => list
        .first()
        .map(|x| matches!(x, ExprTree::Fn(_)))
        .unwrap_or(false),
      _ => false,
    }
  }

  pub fn fn_symbol(&self) -> Option<&FnSymbol> {
    match self {
      ExprTree::List(list) => list.first().and_then(|x| match x {
        ExprTree::Fn(scope) => Some(scope),
        _ => None,
      }),
      _ => None,
    }
  }

  pub fn fn_body(&self) -> Option<&[ExprTree]> {
    match self {
      ExprTree::List(list) => list.first().and_then(|x| match x {
        ExprTree::Fn(_) => Some(&list[1..]),
        _ => None,
      }),
      _ => None,
    }
  }

  pub fn unlazy(&self) -> &Self {
    match self {
      Self::Lazy(x) => x.unlazy(),
      x => x,
    }
  }

  pub fn unlazy_mut(&mut self) -> &mut Self {
    match self {
      Self::Lazy(x) => x.unlazy_mut(),
      x => x,
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
          .map(|expr_tree| expr_tree.type_of())
          .collect::<Vec<_>>(),
      ),

      Self::Lazy(x) => x.type_of(),
      Self::Call(_) => Type::Call,

      Self::Fn(_) => Type::FnScope,

      Self::UserData(_) => Type::UserData,
    }
  }

  pub fn coerce_same(&self, other: &Self) -> Option<(Self, Self)> {
    match self {
      x @ Self::Boolean(_) => Some(x.clone()).zip(other.to_boolean()),
      x @ Self::Integer(_) => Some(x.clone()).zip(other.to_integer()),
      x @ Self::Float(_) => Some(x.clone()).zip(other.to_float()),
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
  //         .map(ExprTree::new)
  //         .collect_vec()
  //         .into(),
  //     )),

  //     x => Some(Self::List([ExprTree::new(x.clone())].into())),
  //   }
  // }

  // pub const fn to_string(&self) -> Option<Self> {
  //   match self {
  //     Self::List(x) => {
  //       x.iter()
  //         .map(|ExprTree(exprTree)| exprTree.borrow())
  //         .map(|exprTree| match *exprTree {
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
  //         .map(|x| ExprTreeKind::String(x.into()))
  //     },

  //     _ => None,
  //   }
  // }
}

impl PartialEq for ExprTree {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      // Same types.
      (Self::Nil, Self::Nil) => true,

      (Self::Boolean(lhs), Self::Boolean(rhs)) => lhs == rhs,
      (Self::Integer(lhs), Self::Integer(rhs)) => lhs == rhs,
      (Self::Float(lhs), Self::Float(rhs)) => lhs == rhs,

      (Self::String(lhs), Self::String(rhs)) => lhs == rhs,

      (Self::List(lhs), Self::List(rhs)) => lhs == rhs,

      (Self::Lazy(lhs), Self::Lazy(rhs)) => lhs == rhs,
      (Self::Call(lhs), Self::Call(rhs)) => lhs == rhs,

      // TODO: I removed `lhs.scope == rhs.scope &&` since it made asserting
      // equality impossible in tests (without filling out the entire scope).
      // Though, I think there's a better solution than removing comparability.
      (Self::Fn(lhs), Self::Fn(rhs)) => lhs.scoped == rhs.scoped,

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

impl PartialOrd for ExprTree {
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

impl fmt::Display for ExprTree {
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
