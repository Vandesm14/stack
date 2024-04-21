use core::fmt;
use std::{cell::RefCell, collections::HashMap, fmt::Formatter, rc::Rc};

use crate::{chain::Chain, expr::FnIdent, prelude::*};

pub type Val = Rc<RefCell<Chain<Option<Expr>>>>;

#[derive(Default, PartialEq)]
pub struct Scope {
  pub items: HashMap<Symbol, Val>,
}

impl fmt::Debug for Scope {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let iter = self.items.iter().map(|(name, item)| (name.as_str(), item));
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

  pub fn from(items: HashMap<Symbol, Val>) -> Self {
    Self { items }
  }

  pub fn define(&mut self, name: Symbol, item: Expr) {
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
  }

  pub fn reserve(&mut self, name: Symbol) {
    if self.items.get(&name).is_none() {
      let val = Rc::new(RefCell::new(Chain::new(None)));
      self.items.insert(name, val);
    }
  }

  pub fn set(
    &mut self,
    name: Symbol,
    item: Expr,
  ) -> Result<(), RunErrorReason> {
    if let Some(chain) = self.items.get_mut(&name) {
      let mut chain = RefCell::borrow_mut(chain);
      chain.set(Some(item));
      Ok(())
    } else {
      Err(RunErrorReason::CannotSetBeforeDef)
    }
  }

  pub fn remove(&mut self, name: Symbol) {
    self.items.remove(&name);
  }

  pub fn has(&self, name: Symbol) -> bool {
    self.items.contains_key(&name)
  }

  pub fn get_val(&self, name: Symbol) -> Option<Expr> {
    self.items.get(&name).and_then(|item| item.borrow().val())
  }

  pub fn get_ref(&self, name: Symbol) -> Option<&Val> {
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
pub struct Scanner {
  pub scope: Scope,
}

impl Scanner {
  pub fn new(scope: Scope) -> Self {
    Self { scope }
  }

  pub fn scan(&mut self, expr: Expr) -> Result<Expr, RunErrorReason> {
    if expr.kind.is_function() {
      let expr = expr;
      // We can unwrap here because we know the expression is a function
      let fn_symbol = match expr.kind.fn_symbol() {
        Some(fn_symbol) => fn_symbol,
        None => return Err(RunErrorReason::InvalidFunction),
      };
      let mut fn_body = match expr.kind.fn_body() {
        Some(fn_body) => fn_body.to_vec(),
        None => return Err(RunErrorReason::InvalidFunction),
      };

      for item in fn_body.iter_mut() {
        if let ExprKind::Symbol(call) = item.kind.unlazy() {
          if !self.scope.has(*call) {
            self.scope.reserve(*call);
          }
        } else if item.kind.unlazy().is_function() {
          let mut scanner = Scanner::new(self.scope.duplicate());
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn top_level_scopes() {
    let source = Rc::new(Source::new("", "0 'a def"));
    let exprs = Parser::new(Lexer::new(source)).parse().unwrap();

    let engine = Engine::new().with_track_info(false);
    let mut context = Context::new().with_stack_capacity(32);
    context = engine.run(context, exprs).unwrap();

    assert_eq!(
      context
        .scope_item(Symbol::new("a".into()))
        .map(|expr| expr.kind),
      Some(ExprKind::Integer(0))
    );
  }

  #[test]
  fn function_scopes_are_isolated() {
    let source = Rc::new(Source::new("", "'(fn 0 'a def) call"));
    let exprs = Parser::new(Lexer::new(source)).parse().unwrap();

    let engine = Engine::new().with_track_info(false);
    let mut context = Context::new().with_stack_capacity(32);
    context = engine.run(context, exprs).unwrap();

    assert_eq!(context.scope_item(Symbol::new("a".into())), None);
  }

  #[test]
  fn nested_function_scopes_are_isolated() {
    let source = Rc::new(Source::new(
      "",
      "0 'a def a '(fn 1 'a def a '(fn 2 'a def a) call a) call a",
    ));
    let exprs = Parser::new(Lexer::new(source)).parse().unwrap();

    let engine = Engine::new().with_track_info(false);
    let mut context = Context::new().with_stack_capacity(32);
    context = engine.run(context, exprs).unwrap();

    assert_eq!(
      context
        .stack()
        .iter()
        .map(|expr| &expr.kind)
        .collect::<Vec<_>>(),
      vec![
        &ExprKind::Integer(0),
        &ExprKind::Integer(1),
        &ExprKind::Integer(2),
        &ExprKind::Integer(1),
        &ExprKind::Integer(0)
      ]
    );
  }

  #[test]
  fn functions_can_set_to_outer() {
    let source =
      Rc::new(Source::new("", "0 'a def a '(fn a 1 'a set a) call a"));
    let exprs = Parser::new(Lexer::new(source)).parse().unwrap();

    let engine = Engine::new().with_track_info(false);
    let mut context = Context::new().with_stack_capacity(32);
    context = engine.run(context, exprs).unwrap();

    assert_eq!(
      context
        .stack()
        .iter()
        .map(|expr| &expr.kind)
        .collect::<Vec<_>>(),
      vec![
        &ExprKind::Integer(0),
        &ExprKind::Integer(0),
        &ExprKind::Integer(1),
        &ExprKind::Integer(1),
      ]
    );
  }

  #[test]
  fn closures_can_access_vars() {
    let source = Rc::new(Source::new(
      "",
      "0 'a def '(fn 1 'a def '(fn a)) call call a",
    ));
    let exprs = Parser::new(Lexer::new(source)).parse().unwrap();

    let engine = Engine::new().with_track_info(false);
    let mut context = Context::new().with_stack_capacity(32);
    context = engine.run(context, exprs).unwrap();

    assert_eq!(
      context
        .stack()
        .iter()
        .map(|expr| &expr.kind)
        .collect::<Vec<_>>(),
      vec![&ExprKind::Integer(1), &ExprKind::Integer(0),]
    );
  }

  #[test]
  fn closures_can_mutate_vars() {
    let source = Rc::new(Source::new(
      "",
      "0 'a def '(fn 1 'a def '(fn a 2 'a set a)) call call a",
    ));
    let exprs = Parser::new(Lexer::new(source)).parse().unwrap();

    let engine = Engine::new().with_track_info(false);
    let mut context = Context::new().with_stack_capacity(32);
    context = engine.run(context, exprs).unwrap();

    assert_eq!(
      context
        .stack()
        .iter()
        .map(|expr| &expr.kind)
        .collect::<Vec<_>>(),
      vec![
        &ExprKind::Integer(1),
        &ExprKind::Integer(2),
        &ExprKind::Integer(0),
      ]
    );
  }

  #[test]
  fn scopeless_functions_can_def_outer() {
    let source = Rc::new(Source::new("", "'(fn! 0 'a def) call a"));
    let exprs = Parser::new(Lexer::new(source)).parse().unwrap();

    let engine = Engine::new().with_track_info(false);
    let mut context = Context::new().with_stack_capacity(32);
    context = engine.run(context, exprs).unwrap();

    assert_eq!(
      context
        .stack()
        .iter()
        .map(|expr| &expr.kind)
        .collect::<Vec<_>>(),
      vec![&ExprKind::Integer(0),]
    );

    let source = Rc::new(Source::new("", "0 'a def '(fn! a 1 'a def) call a"));
    let exprs = Parser::new(Lexer::new(source)).parse().unwrap();

    let engine = Engine::new().with_track_info(false);
    let mut context = Context::new().with_stack_capacity(32);
    context = engine.run(context, exprs).unwrap();

    assert_eq!(
      context
        .stack()
        .iter()
        .map(|expr| &expr.kind)
        .collect::<Vec<_>>(),
      vec![&ExprKind::Integer(0), &ExprKind::Integer(1),]
    );
  }

  #[test]
  fn scopeless_function_can_reuse_define() {
    let source =
      Rc::new(Source::new("", "'(fn! def) 'define def 0 'a define a"));
    let exprs = Parser::new(Lexer::new(source)).parse().unwrap();

    let engine = Engine::new().with_track_info(false);
    let mut context = Context::new().with_stack_capacity(32);
    context = engine.run(context, exprs).unwrap();

    assert_eq!(
      context
        .stack()
        .iter()
        .map(|expr| &expr.kind)
        .collect::<Vec<_>>(),
      vec![&ExprKind::Integer(0)]
    );
  }

  // TODO: test failure cases
  // #[test]
  // fn should_fail_on_invalid_symbol() {
  //   let mut program = Program::new().with_core().unwrap();
  //   let result = program.eval_string("a").unwrap_err();

  //   assert_eq!(result.message, "unknown call a");
  // }

  // #[test]
  // fn should_fail_on_invalid_symbol_in_fn() {
  //   let mut program = Program::new().with_core().unwrap();
  //   let result = program.eval_string("'(fn a) call").unwrap_err();

  //   assert_eq!(result.message, "unknown call a");
  // }

  #[test]
  fn variables_defined_from_scopeless_should_be_usable() {
    let source = Rc::new(Source::new(
      "",
      "'(fn! 0 'a def) '(fn call '(fn a)) call call",
    ));
    let exprs = Parser::new(Lexer::new(source)).parse().unwrap();

    let engine = Engine::new().with_track_info(false);
    let mut context = Context::new().with_stack_capacity(32);
    context = engine.run(context, exprs).unwrap();

    assert_eq!(context.scope_item(Symbol::new("a".into())), None);
    assert_eq!(
      context
        .stack()
        .iter()
        .map(|expr| &expr.kind)
        .collect::<Vec<_>>(),
      vec![&ExprKind::Integer(0),]
    );
  }
}
