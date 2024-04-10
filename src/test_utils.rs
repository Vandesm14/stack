use crate::{Expr, ExprKind, FnSymbol};
use core::{any::Any, cell::RefCell};
use internment::Intern;
use itertools::Itertools;
use std::{fmt::Debug, ops::Deref, rc::Rc};

#[derive(Debug, Clone)]
pub enum TestExpr {
  Nil,

  Boolean(bool),
  Integer(i64),
  Float(f64),

  String(String),
  List(Vec<TestExpr>),

  Lazy(Box<TestExpr>),
  Call(Intern<String>),

  /// Boolean denotes whether to create a new scope.
  Fn(FnSymbol),

  UserData(Rc<RefCell<dyn Any>>),
}

impl PartialEq for TestExpr {
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

      _ => false,
    }
  }
}

impl From<Expr> for TestExpr {
  fn from(value: Expr) -> Self {
    let val = value.val;
    match val {
      ExprKind::Nil => TestExpr::Nil,

      ExprKind::Boolean(bool) => TestExpr::Boolean(bool),
      ExprKind::Integer(int) => TestExpr::Integer(int),
      ExprKind::Float(float) => TestExpr::Float(float),

      ExprKind::String(string) => TestExpr::String(string),
      ExprKind::List(list) => {
        let mut items: Vec<TestExpr> = Vec::new();
        for item in list {
          items.push(item.into());
        }

        TestExpr::List(items)
      }

      ExprKind::Lazy(lazy) => {
        let inner: TestExpr = lazy.deref().clone().into();
        TestExpr::Lazy(Box::new(inner))
      }
      ExprKind::Call(call) => TestExpr::Call(call),

      ExprKind::Fn(fn_symbol) => TestExpr::Fn(fn_symbol),

      ExprKind::UserData(data) => TestExpr::UserData(data),
    }
  }
}

pub fn simple_expr(expr: Expr) -> TestExpr {
  expr.into()
}

pub fn simple_exprs(exprs: Vec<Expr>) -> Vec<TestExpr> {
  exprs.into_iter().map(simple_expr).collect_vec()
}
