use crate::{
  interner::interner, Expr
};
use core::fmt;
use lasso::Spur;
use std::{cell::RefCell, collections::HashMap, fmt::Formatter, rc::Rc};

type Val = Rc<RefCell<Expr>>;

#[derive(Default, Clone, PartialEq)]
pub struct Scope {
  pub items: HashMap<Spur, Val>,
}

impl fmt::Debug for Scope {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let iter = self
      .items
      .iter()
      .map(|(name, item)| (interner().resolve(name), item));
    write!(
      f,
      "{:?}",
      HashMap::<&str, &Val>::from_iter(iter)
    )
  }
}

impl Scope {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn define(&mut self, name: Spur, item: Expr) -> Result<(), String> {
    let item = Rc::new(RefCell::new(item));
    self.items.insert(name, item);

    Ok(())
  }

  pub fn set(&mut self, name: Spur, item: Expr) -> Result<(), String> {
    if let None = self.items.get(&name) {
      return Err("Cannot set to a nonexistent variable".to_owned());
    }

    let item = Rc::new(RefCell::new(item));
    self.items.insert(name, item);

    Ok(())
  }

  pub fn remove(&mut self, name: Spur) -> Result<(), String> {
    if let None = self.items.get(&name) {
      return Err("Cannot remove a nonexistent variable".to_owned());
    }

    self.items.remove(&name);

    Ok(())
  }

  pub fn get(&self, name: Spur) -> Option<Expr> {
    self.items.get(&name).map(|item| item.borrow().clone())
  }

  pub fn get_ref(&self, name: Spur) -> Option<Val> {
    self.items.get(&name).cloned()
  }
}

#[derive(Default, Debug, Clone)]
pub struct Scanner {
  pub scope: Scope,
}

impl Scanner {
  pub fn new(scope: Scope) -> Self {
    Self { scope }
  }

  pub fn scan(&mut self, expr: Expr) -> Result<Expr, String> {
    if expr.is_function() {
      let mut expr = expr;
      // We can unwrap here because we know the expression is a function
      let fn_symbol = expr.fn_symbol().unwrap();
      let fn_body = expr.fn_body().unwrap();

      for item in fn_body {
        if let Expr::Call(call) = item {
          println!("Call: {}", item);
        } else {
          println!("Not a call: {}", item);
        }
      }

      Ok(expr)
    } else {
      // If the expression is not a function, we just return it
      Ok(expr)
    }
  }
}
