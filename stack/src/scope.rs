use internment::Intern;

use core::fmt;
use std::{cell::RefCell, collections::HashMap, fmt::Formatter, rc::Rc};

use crate::{chain::Chain, expr::FnIdent, module::Func, prelude::*};

pub type Val = Rc<RefCell<Chain<Option<Expr>>>>;

#[derive(Default, PartialEq)]
pub struct Scope {
  pub items: HashMap<Intern<String>, Val>,
}

impl fmt::Debug for Scope {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let iter = self
      .items
      .iter()
      .map(|(name, item)| (name.as_ref().as_str(), item));
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

  pub fn from(items: HashMap<Intern<String>, Val>) -> Self {
    Self { items }
  }

  pub fn define(
    &mut self,
    name: Intern<String>,
    item: Expr,
  ) -> Result<(), String> {
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

  pub fn reserve(&mut self, name: Intern<String>) -> Result<(), String> {
    if self.items.get(&name).is_none() {
      let val = Rc::new(RefCell::new(Chain::new(None)));
      self.items.insert(name, val);
      Ok(())
    } else {
      Err("Cannot reserve an already existing variable".to_owned())
    }
  }

  pub fn set(
    &mut self,
    name: Intern<String>,
    item: Expr,
  ) -> Result<(), String> {
    if let Some(chain) = self.items.get_mut(&name) {
      let mut chain = RefCell::borrow_mut(chain);
      chain.set(Some(item));
      Ok(())
    } else {
      Err("Cannot set to a nonexistent variable".to_owned())
    }
  }

  pub fn remove(&mut self, name: Intern<String>) {
    self.items.remove(&name);
  }

  pub fn has(&self, name: Intern<String>) -> bool {
    self.items.contains_key(&name)
  }

  pub fn get_val(&self, name: Intern<String>) -> Option<Expr> {
    self.items.get(&name).and_then(|item| item.borrow().val())
  }

  pub fn get_ref(&self, name: Intern<String>) -> Option<&Val> {
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
  pub funcs: &'a HashMap<Intern<String>, Func>,
}

impl<'a> Scanner<'a> {
  pub fn new(scope: Scope, funcs: &'a HashMap<Intern<String>, Func>) -> Self {
    Self { scope, funcs }
  }

  pub fn scan(&mut self, expr: Expr) -> Result<Expr, String> {
    if expr.kind.is_function() {
      let expr = expr;
      // We can unwrap here because we know the expression is a function
      let fn_symbol = match expr.kind.fn_symbol() {
        Some(fn_symbol) => fn_symbol,
        None => return Err("Invalid function".to_owned()),
      };
      let mut fn_body = match expr.kind.fn_body() {
        Some(fn_body) => fn_body.to_vec(),
        None => return Err("Invalid function".to_owned()),
      };

      for item in fn_body.iter_mut() {
        if let ExprKind::Symbol(call) = item.kind.unlazy() {
          if !self.funcs.contains_key(call) && !self.scope.has(*call) {
            self.scope.reserve(*call).unwrap();
          }
        } else if item.kind.unlazy().is_function() {
          let mut scanner = Scanner::new(self.scope.clone(), self.funcs);
          let unlazied_mut = item.kind.unlazy_mut();
          *unlazied_mut = scanner
            .scan(Expr {
              kind: unlazied_mut.clone(),
              info: item.info.clone(),
            })
            .unwrap()
            .kind
        }
      }

      let mut fn_scope = fn_symbol.scope.clone();
      fn_scope.merge(self.scope.clone());

      let fn_ident = ExprKind::Fn(FnIdent {
        scope: fn_scope,
        scoped: fn_symbol.scoped,
      });

      let mut list_items = vec![Expr {
        kind: fn_ident,
        info: expr.info.clone(),
      }];
      list_items.extend(fn_body);

      let new_expr = ExprKind::List(list_items);

      Ok(Expr {
        kind: new_expr,
        info: expr.info,
      })
    } else {
      // If the expression is not a function, we just return it
      Ok(expr)
    }
  }
}

// #[cfg(test)]
// mod tests {
//   use crate::{interner::interner, ExprKind, Program};

//   #[test]
//   fn top_level_scopes() {
//     let mut program = Program::new().with_core().unwrap();
//     program.eval_string("0 'a def").unwrap();

//     assert_eq!(
//       program
//         .scopes
//         .last()
//         .unwrap()
//         .get_val(interner().get_or_intern("a")),
//       Some(ExprKind::Integer(0))
//     );
//   }

//   #[test]
//   fn function_scopes_are_isolated() {
//     let mut program = Program::new().with_core().unwrap();
//     program.eval_string("'(fn 0 'a def) call").unwrap();

//     assert_eq!(
//       program
//         .scopes
//         .last()
//         .unwrap()
//         .get_val(interner().get_or_intern("a")),
//       None
//     );
//   }

//   #[test]
//   fn functions_can_set_to_outer() {
//     let mut program = Program::new().with_core().unwrap();
//     program.eval_string("0 'a def '(fn 1 'a set) call").unwrap();

//     assert_eq!(
//       program
//         .scopes
//         .last()
//         .unwrap()
//         .get_val(interner().get_or_intern("a")),
//       Some(ExprKind::Integer(1))
//     );
//   }

//   #[test]
//   fn functions_can_shadow_outer() {
//     let mut program = Program::new().with_core().unwrap();
//     program.eval_string("0 'a def '(fn 1 'a def) call").unwrap();

//     assert_eq!(
//       program
//         .scopes
//         .last()
//         .unwrap()
//         .get_val(interner().get_or_intern("a")),
//       Some(ExprKind::Integer(0))
//     );
//   }

//   #[test]
//   fn closures_can_access_vars() {
//     let mut program = Program::new().with_core().unwrap();
//     program
//       .eval_string("0 'a def '(fn 1 'a def '(fn a)) call call")
//       .unwrap();

//     assert_eq!(program.stack, vec![ExprKind::Integer(1)]);
//   }

//   #[test]
//   fn closures_can_mutate_vars() {
//     let mut program = Program::new().with_core().unwrap();
//     program
//       .eval_string("0 'a def '(fn 1 'a def '(fn 2 'a set a)) call call")
//       .unwrap();

//     assert_eq!(program.stack, vec![ExprKind::Integer(2)],);
//   }

//   #[test]
//   fn scopeless_functions_can_def_outer() {
//     let mut program = Program::new().with_core().unwrap();
//     program.eval_string("'(fn! 0 'a def) call").unwrap();

//     assert_eq!(
//       program
//         .scopes
//         .last()
//         .unwrap()
//         .get_val(interner().get_or_intern("a")),
//       Some(ExprKind::Integer(0))
//     );
//   }

//   #[test]
//   fn scopeless_function_macro_test() {
//     let mut program = Program::new().with_core().unwrap();
//     program
//       .eval_string("'(fn! def) 'define def 0 'a define")
//       .unwrap();

//     assert_eq!(
//       program
//         .scopes
//         .last()
//         .unwrap()
//         .get_val(interner().get_or_intern("a")),
//       Some(ExprKind::Integer(0))
//     );
//   }

//   #[test]
//   fn should_fail_on_invalid_symbol() {
//     let mut program = Program::new().with_core().unwrap();
//     let result = program.eval_string("a").unwrap_err();

//     assert_eq!(result.message, "unknown call a");
//   }

//   #[test]
//   fn should_fail_on_invalid_symbol_in_fn() {
//     let mut program = Program::new().with_core().unwrap();
//     let result = program.eval_string("'(fn a) call").unwrap_err();

//     assert_eq!(result.message, "unknown call a");
//   }

//   #[test]
//   fn variables_defined_from_scopeless_should_be_usable() {
//     let mut program = Program::new().with_core().unwrap();
//     program
//       .eval_string("'(fn! 0 'a def) '(fn call '(fn a)) call call")
//       .unwrap();
//   }
// }
