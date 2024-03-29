use crate::{interner::interner, Chain, Expr, FnSymbol, Func};
use core::fmt;
use lasso::Spur;
use std::{cell::RefCell, collections::HashMap, fmt::Formatter, rc::Rc};

pub type Val = Rc<RefCell<Chain<Option<Expr>>>>;

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
        true => {
          chain.set(Some(item));
        }
        false => {
          chain.unlink_with(Some(item));
        }
      }
    } else {
      let val = Rc::new(RefCell::new(Chain::new(Some(item))));
      self.items.insert(name, val);
    }

    Ok(())
  }

  pub fn reserve(&mut self, name: Spur) -> Result<(), String> {
    if self.items.get(&name).is_none() {
      let val = Rc::new(RefCell::new(Chain::new(None)));
      self.items.insert(name, val);
      Ok(())
    } else {
      Err("Cannot reserve an already existing variable".to_owned())
    }
  }

  pub fn set(&mut self, name: Spur, item: Expr) -> Result<(), String> {
    if let Some(chain) = self.items.get_mut(&name) {
      let mut chain = RefCell::borrow_mut(chain);
      chain.set(Some(item));
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
    self.items.get(&name).and_then(|item| item.borrow().val())
  }

  pub fn get_ref(&self, name: Spur) -> Option<&Val> {
    self.items.get(&name)
  }

  /// Merges another scope into this one, not overwriting any existing variables
  pub fn merge(&mut self, other: Scope) {
    for (name, item) in other.items {
      if !self.has(name)
        || (self.get_val(name).is_none() && item.borrow().val().is_some())
      {
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
      let fn_symbol = match expr.fn_symbol() {
        Some(fn_symbol) => fn_symbol,
        None => return Err("Invalid function".to_owned()),
      };
      let mut fn_body = match expr.fn_body() {
        Some(fn_body) => fn_body.to_vec(),
        None => return Err("Invalid function".to_owned()),
      };

      for item in fn_body.iter_mut() {
        if let Expr::Call(call) = item.unlazy() {
          if !self.funcs.contains_key(call) && !self.scope.has(*call) {
            self.scope.reserve(*call).unwrap();
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
  use crate::{interner::interner, Expr, Program};

  #[test]
  fn top_level_scopes() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("0 'a def").unwrap();

    assert_eq!(
      program
        .scopes
        .last()
        .unwrap()
        .get_val(interner().get_or_intern("a")),
      Some(Expr::Integer(0))
    );
  }

  #[test]
  fn function_scopes_are_isolated() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("'(fn 0 'a def) call").unwrap();

    assert_eq!(
      program
        .scopes
        .last()
        .unwrap()
        .get_val(interner().get_or_intern("a")),
      None
    );
  }

  #[test]
  fn functions_can_set_to_outer() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("0 'a def '(fn 1 'a set) call").unwrap();

    assert_eq!(
      program
        .scopes
        .last()
        .unwrap()
        .get_val(interner().get_or_intern("a")),
      Some(Expr::Integer(1))
    );
  }

  #[test]
  fn functions_can_shadow_outer() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("0 'a def '(fn 1 'a def) call").unwrap();

    assert_eq!(
      program
        .scopes
        .last()
        .unwrap()
        .get_val(interner().get_or_intern("a")),
      Some(Expr::Integer(0))
    );
  }

  #[test]
  fn closures_can_access_vars() {
    let mut program = Program::new().with_core().unwrap();
    program
      .eval_string("0 'a def '(fn 1 'a def '(fn a)) call call")
      .unwrap();

    assert_eq!(program.stack, vec![Expr::Integer(1)]);
  }

  #[test]
  fn closures_can_mutate_vars() {
    let mut program = Program::new().with_core().unwrap();
    program
      .eval_string("0 'a def '(fn 1 'a def '(fn 2 'a set a)) call call")
      .unwrap();

    assert_eq!(program.stack, vec![Expr::Integer(2)],);
  }

  #[test]
  fn scopeless_functions_can_def_outer() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("'(fn! 0 'a def) call").unwrap();

    assert_eq!(
      program
        .scopes
        .last()
        .unwrap()
        .get_val(interner().get_or_intern("a")),
      Some(Expr::Integer(0))
    );
  }

  #[test]
  fn scopeless_function_macro_test() {
    let mut program = Program::new().with_core().unwrap();
    program
      .eval_string("'(fn! def) 'define def 0 'a define")
      .unwrap();

    assert_eq!(
      program
        .scopes
        .last()
        .unwrap()
        .get_val(interner().get_or_intern("a")),
      Some(Expr::Integer(0))
    );
  }

  #[test]
  fn should_fail_on_invalid_symbol() {
    let mut program = Program::new().with_core().unwrap();
    let result = program.eval_string("a").unwrap_err();

    assert_eq!(result.message, "unknown call a");
  }

  #[test]
  fn should_fail_on_invalid_symbol_in_fn() {
    let mut program = Program::new().with_core().unwrap();
    let result = program.eval_string("'(fn a) call").unwrap_err();

    assert_eq!(result.message, "unknown call a");
  }

  #[test]
  fn variables_defined_from_scopeless_should_be_usable() {
    let mut program = Program::new().with_core().unwrap();
    program
      .eval_string("'(fn! 0 'a def) '(fn call '(fn a)) call call")
      .unwrap();
  }
}
