use core::{cmp::Ordering, fmt, iter, num::FpCategory};

#[derive(Debug, Clone, Default)]
pub enum Expr {
  #[default]
  Nil,

  Boolean(bool),
  Integer(i64),
  Float(f64),

  Pointer(usize),

  List(Vec<Expr>),
  String(String),

  Lazy(Box<Expr>),
  Call(String),

  FnScope(Option<usize>),
  ScopePush,
  ScopePop,
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
    self.function_scope().is_some()
  }

  pub fn function_scope(&self) -> Option<usize> {
    match self {
      Expr::List(list) => list.first().and_then(|x| match x {
        Expr::FnScope(scope) => *scope,
        _ => None,
      }),
      _ => None,
    }
  }

  pub fn contains_block(&self) -> bool {
    match self {
      Expr::List(list) => list.get(1).is_some_and(|x| match x {
        Expr::ScopePush => true,
        _ => false,
      }),
      _ => false,
    }
  }

  pub fn type_of(&self) -> Type {
    match self {
      Self::Nil => Type::Nil,

      Self::Boolean(_) => Type::Boolean,
      Self::Integer(_) => Type::Integer,
      Self::Float(_) => Type::Float,

      Self::Pointer(_) => Type::Pointer,

      Self::List(_) => Type::List,
      Self::String(_) => Type::String,

      Self::Lazy(x) => x.type_of(),
      Self::Call(_) => Type::Call,

      Self::FnScope(_) => Type::FnScope,
      Self::ScopePush => Type::ScopePush,
      Self::ScopePop => Type::ScopePop,
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

      _ => None,
    }
  }

  pub fn to_float(&self) -> Option<Self> {
    match self {
      Self::Integer(x) => Some(Self::Float(*x as f64)),
      x @ Self::Float(_) => Some(x.clone()),

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

      (Self::FnScope(lhs), Self::FnScope(rhs)) => lhs == rhs,
      (Self::ScopePush, Self::ScopePush) => true,
      (Self::ScopePop, Self::ScopePop) => true,

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

      (Self::FnScope(lhs), Self::FnScope(rhs)) => lhs.partial_cmp(rhs),
      (Self::ScopePush, Self::ScopePush) => Some(Ordering::Equal),
      (Self::ScopePop, Self::ScopePop) => Some(Ordering::Equal),

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

impl fmt::Display for Expr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Nil => f.write_str("nil"),

      Self::Boolean(x) => fmt::Display::fmt(x, f),
      Self::Integer(x) => fmt::Display::fmt(x, f),
      Self::Float(x) => fmt::Display::fmt(x, f),

      Self::Pointer(x) => {
        f.write_str("*")?;
        fmt::Display::fmt(x, f)
      }

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
      Self::String(x) => fmt::Display::fmt(x, f),

      Self::Lazy(x) => {
        f.write_str("'")?;
        fmt::Display::fmt(x, f)
      }
      Self::Call(x) => fmt::Display::fmt(x, f),

      Self::FnScope(x) => {
        f.write_str("fn")?;

        match x {
          Some(x) => {
            f.write_str("(")?;
            fmt::Display::fmt(x, f)?;
            f.write_str(")")
          }
          None => Ok(()),
        }
      }
      Self::ScopePush => f.write_str("scope_push"),
      Self::ScopePop => f.write_str("scope_pop"),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
  Nil,

  Boolean,
  Integer,
  Float,

  Pointer,

  List,
  String,

  Call,

  FnScope,
  ScopePush,
  ScopePop,
}

impl Type {
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Nil => "nil",

      Self::Boolean => "boolean",
      Self::Integer => "integer",
      Self::Float => "float",

      Self::Pointer => "pointer",

      Self::List => "list",
      Self::String => "string",

      Self::Call => "call",

      Self::FnScope => "fn",
      Self::ScopePush => "scope_push",
      Self::ScopePop => "scope_pop",
    }
  }
}

impl fmt::Display for Type {
  #[inline]
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.as_str())
  }
}
