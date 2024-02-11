use crate::{interner::interner, Chain, Expr, FnSymbol, Func};
use core::fmt;
use lasso::Spur;
use std::{cell::RefCell, collections::HashMap, fmt::Formatter, rc::Rc};

pub type Val = Rc<RefCell<Chain<Expr>>>;

#[derive(Default, PartialEq)]
pub struct Scope {
  pub items: HashMap<Spur, Val>,
}

impl fmt::Debug for Scope {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let iter = self
      .items
      .iter()
      .map(|(name, item)| (interner().resolve(name), item));
    write!(f, "{:?}", HashMap::<&str, &Val>::from_iter(iter))
  }
}

impl Clone for Scope {
  fn clone(&self) -> Self {
    let mut items = HashMap::new();

    for (name, item) in self.items.iter() {
      items.insert(*name, item.clone());
    }

    Self { items }
  }
}

impl Scope {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn from(items: HashMap<Spur, Val>) -> Self {
    Self { items }
  }

  pub fn define(&mut self, name: Spur, item: Expr) -> Result<(), String> {
    if let Some(chain) = self.items.get(&name) {
      let mut chain = RefCell::borrow_mut(chain);
      match chain.is_root() {
        // Chain::Link(_) => {
        //   val.unlink_with(|_| item);
        // }
        // Chain::Root(root) => {
        //   let mut root = root.borrow_mut();
        //   *root = item;
        // }
        true => {
          chain.set(item);
        }
        false => {
          chain.unlink_with(item);
        }
      }
    } else {
      let val = Rc::new(RefCell::new(Chain::new(item)));
      self.items.insert(name, val);
    }

    Ok(())
  }

  pub fn set(&mut self, name: Spur, item: Expr) -> Result<(), String> {
    if let Some(chain) = self.items.get_mut(&name) {
      let mut chain = RefCell::borrow_mut(chain);
      chain.set(item);
      Ok(())
    } else {
      Err("Cannot set to a nonexistent variable".to_owned())
    }
  }

  pub fn remove(&mut self, name: Spur) -> Result<(), String> {
    if self.items.get(&name).is_none() {
      return Err("Cannot remove a nonexistent variable".to_owned());
    }

    self.items.remove(&name);

    Ok(())
  }

  pub fn has(&self, name: Spur) -> bool {
    self.items.contains_key(&name)
  }

  pub fn get_val(&self, name: Spur) -> Option<Expr> {
    self.items.get(&name).map(|item| item.borrow().val())
  }

  pub fn get_ref(&self, name: Spur) -> Option<&Val> {
    self.items.get(&name)
  }

  /// Merges another scope into this one, not overwriting any existing variables
  pub fn merge(&mut self, other: Scope) {
    for (name, item) in other.items {
      if !self.has(name) {
        self.items.insert(name, item);
      }
    }
  }

  pub fn duplicate(&self) -> Self {
    let mut items = HashMap::new();

    for (name, item) in self.items.iter() {
      let mut item = RefCell::borrow_mut(item);
      items.insert(*name, item.link());
    }

    Self { items }
  }
}

#[derive(Debug)]
pub struct Scanner<'a> {
  pub scope: Scope,
  pub funcs: &'a HashMap<Spur, Func>,
}

impl<'a> Scanner<'a> {
  pub fn new(scope: Scope, funcs: &'a HashMap<Spur, Func>) -> Self {
    Self { scope, funcs }
  }

  pub fn scan(&mut self, expr: Expr) -> Result<Expr, String> {
    if expr.is_function() {
      let expr = expr;
      // We can unwrap here because we know the expression is a function
      let fn_symbol = expr.fn_symbol().unwrap();
      let mut fn_body = expr.fn_body().unwrap().to_vec();

      for item in fn_body.iter_mut() {
        if let Expr::Call(call) = item.unlazy() {
          if !self.funcs.contains_key(call) && !self.scope.has(*call) {
            self.scope.define(*call, Expr::Nil).unwrap();
          }
        } else if item.unlazy().is_function() {
          let mut scanner = Scanner::new(self.scope.clone(), self.funcs);
          let unlazied_mut = item.unlazy_mut();
          *unlazied_mut = scanner.scan(unlazied_mut.clone()).unwrap();
        }
      }

      let mut fn_scope = fn_symbol.scope.clone();
      fn_scope.merge(self.scope.clone());

      let fn_symbol = Expr::Fn(FnSymbol {
        scope: fn_scope,
        scoped: fn_symbol.scoped,
      });

      let mut list_items = vec![fn_symbol];
      list_items.extend(fn_body);

      let expr = Expr::List(list_items);

      Ok(expr)
    } else {
      // If the expression is not a function, we just return it
      Ok(expr)
    }
  }
}

#[cfg(test)]
mod tests {
  // TODO: Write tests
}
