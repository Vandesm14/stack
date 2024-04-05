use core::{
  any::Any, cell::RefCell, cmp::Ordering, fmt, iter, num::FpCategory,
};
use std::{ops::Range, rc::Rc};

use itertools::Itertools;
use lasso::Spur;

use crate::{interner::interner, ExprTree, Scope};

#[derive(Clone)]
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

pub type AstIndex = usize;

#[derive(Debug, Clone)]
pub enum Expr {
  Nil,

  Boolean(bool),
  Integer(i64),
  Float(f64),

  String(Spur),
  List(Vec<AstIndex>),

  Lazy(AstIndex),
  Call(Spur),

  /// Boolean denotes whether to create a new scope.
  Fn(FnSymbol),

  UserData(Rc<RefCell<dyn Any>>),
}

impl Expr {
  pub fn into_expr_tree(&self, ast: &Ast) -> ExprTree {
    match self {
      Self::Nil => ExprTree::Nil,

      Self::Boolean(bool) => ExprTree::Boolean(*bool),
      Self::Integer(int) => ExprTree::Integer(*int),
      Self::Float(f64) => ExprTree::Float(*f64),

      Self::String(spur) => ExprTree::String(*spur),
      Self::List(indicies) => ExprTree::List(
        ast
          .expr_many(indicies.clone())
          .into_iter()
          .map(|expr| expr.into_expr_tree(ast))
          .collect_vec(),
      ),

      Self::Lazy(index) => {
        ExprTree::Lazy(Box::new(ast.expr(*index).unwrap().into_expr_tree(ast)))
      }
      Self::Call(spur) => ExprTree::Call(*spur),

      Self::Fn(symbol) => ExprTree::Fn(symbol.clone()),

      Self::UserData(data) => ExprTree::UserData(data.clone()),
    }
  }

  pub fn is_truthy(&self) -> bool {
    match self.to_boolean() {
      Some(Self::Boolean(x)) => x,
      _ => false,
    }
  }

  pub const fn is_nil(&self) -> bool {
    matches!(*self, Expr::Nil)
  }

  pub fn is_function(&self, ast: &Ast) -> bool {
    match self {
      Expr::List(list) => list
        .first()
        .map(|x| matches!(ast.expr(*x), Some(Expr::Fn(_))))
        .unwrap_or(false),
      _ => false,
    }
  }

  pub fn fn_symbol(&self) -> Option<&AstIndex> {
    match self {
      Expr::List(list) => list.first(),
      _ => None,
    }
  }

  pub fn fn_body(&self, ast: &Ast) -> Option<&[AstIndex]> {
    match self {
      Expr::List(list) => list.first().and_then(|x| match ast.expr(*x) {
        Some(Expr::Fn(_)) => Some(&list[1..]),
        _ => None,
      }),
      _ => None,
    }
  }

  pub fn type_of(&self, ast: &Ast) -> Type {
    match self {
      Self::Nil => Type::Nil,

      Self::Boolean(_) => Type::Boolean,
      Self::Integer(_) => Type::Integer,
      Self::Float(_) => Type::Float,

      Self::String(_) => Type::String,

      Self::List(list) => Type::List(
        list
          .iter()
          .map(|expr| ast.expr(*expr).unwrap().type_of(ast))
          .collect::<Vec<_>>(),
      ),

      Self::Lazy(x) => ast.expr(*x).unwrap().type_of(ast),
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

  pub fn recursive_partial_eq(&self, other: &Self, ast: &Ast) -> bool {
    match (self, other) {
      // Same types.
      (Self::List(lhs), Self::List(rhs)) => {
        if lhs.len() == rhs.len() {
          lhs.iter().zip(rhs.iter()).all(|(a, b)| {
            if let (Some(a), Some(b)) = (ast.expr(*a), ast.expr(*b)) {
              a.recursive_partial_eq(b, ast)
            } else {
              false
            }
          })
        } else {
          false
        }
      }

      (Self::Lazy(lhs), Self::Lazy(rhs)) => {
        if let (Some(lhs), Some(rhs)) = (ast.expr(*lhs), ast.expr(*rhs)) {
          lhs.recursive_partial_eq(rhs, ast)
        } else {
          false
        }
      }

      (a, b) => a == b,
    }
  }

  pub fn recursive_partial_ord(
    &self,
    other: &Self,
    ast: &Ast,
  ) -> Option<Ordering> {
    match (self, other) {
      // Same types.
      (Self::Lazy(lhs), Self::Lazy(rhs)) => {
        if let (Some(lhs), Some(rhs)) = (ast.expr(*lhs), ast.expr(*rhs)) {
          lhs.recursive_partial_ord(rhs, ast)
        } else {
          None
        }
      }

      (a, b) => a.partial_cmp(b),
    }
  }
}

#[derive(Debug, Clone)]
pub struct Ast {
  pub exprs: Vec<Expr>,
}

impl Default for Ast {
  fn default() -> Self {
    Self::new()
  }
}

impl Ast {
  pub const NIL: AstIndex = 0;

  pub fn new() -> Self {
    Self {
      exprs: vec![Expr::Nil],
    }
  }

  pub fn len(&self) -> usize {
    self.exprs.len()
  }

  pub fn is_truthy(&self, index: AstIndex) -> Option<bool> {
    match self.expr(index)?.to_boolean() {
      Some(Expr::Boolean(x)) => Some(x),
      _ => Some(false),
    }
  }

  pub fn expr(&self, index: AstIndex) -> Option<&Expr> {
    self.exprs.get(index)
  }

  pub fn expr_mut(&mut self, index: AstIndex) -> Option<&mut Expr> {
    self.exprs.get_mut(index)
  }

  pub fn expr_many<I>(&self, indices: I) -> Vec<Expr>
  where
    I: IntoIterator<Item = AstIndex>,
  {
    indices
      .into_iter()
      .filter_map(|index| self.expr(index).cloned())
      .collect()
  }

  pub fn expr_range(&self, range: Range<AstIndex>) -> Option<&[Expr]> {
    self.exprs.get(range)
  }

  pub fn push_expr(&mut self, expr: Expr) -> AstIndex {
    let last_index = self.exprs.len();
    self.exprs.push(expr);

    last_index
  }

  pub fn push_many(&mut self, exprs: Vec<Expr>) -> Vec<AstIndex> {
    exprs
      .into_iter()
      .map(|expr| self.push_expr(expr))
      .collect_vec()
  }

  pub fn set_expr(&mut self, index: AstIndex, expr: Expr) -> bool {
    if let Some(mut stored) = self.expr_mut(index) {
      *stored = expr;
      true
    } else {
      false
    }
  }

  pub fn unlazy(&self, index: AstIndex) -> Option<AstIndex> {
    match self.expr(index) {
      Some(Expr::Lazy(x)) => self.unlazy(*x),
      Some(_) => Some(index),
      None => None,
    }
  }

  // TODO: reimplement... if we need this
  // pub fn unlazy_mut(&mut self, index: AstIndex) -> Option<&mut Expr> {
  //   match self.expr(index) {
  //     Some(Expr::Lazy(x)) => self.unlazy_mut(*x),
  //     Some(_) => Some(index),
  //     None => None,
  //   }
  // }

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

impl PartialOrd for Expr {
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

impl fmt::Display for Expr {
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
  fn to_float(expr: Expr) -> Option<Expr> {
    expr.to_float()
  }
}
