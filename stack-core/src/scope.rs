use core::fmt;
use std::{cell::RefCell, collections::HashMap, fmt::Formatter, rc::Rc};

use serde::{Deserialize, Deserializer};

use crate::{chain::Chain, expr::FnScope, prelude::*};

pub type Val = Rc<RefCell<Chain<Option<Expr>>>>;

#[derive(Default, PartialEq)]
pub struct Scope {
  pub items: HashMap<Symbol, Val>,
}

impl<'de> Deserialize<'de> for Scope {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    // Helper struct to deserialize the items
    #[derive(Deserialize)]
    struct ScopeHelper {
      items: HashMap<String, DeserializeVal>,
    }

    // Helper enum to deserialize Val
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum DeserializeVal {
      Some(Expr),
      None,
    }

    // Deserialize into the helper struct
    let helper = ScopeHelper::deserialize(deserializer)?;

    // Convert DeserializeVal to Val
    let mut items: HashMap<String, Val> = helper
      .items
      .into_iter()
      .map(|(k, v)| {
        let val = match v {
          DeserializeVal::Some(expr) => {
            Rc::new(RefCell::new(Chain::new(Some(expr))))
          }
          DeserializeVal::None => Rc::new(RefCell::new(Chain::new(None))),
        };
        (k, val)
      })
      .collect();
    let mut scope: HashMap<Symbol, Val> = HashMap::new();
    for (k, v) in items.drain() {
      scope.insert(Symbol::from_ref(k.as_str()), v);
    }

    Ok(Scope { items: scope })
  }
}

impl fmt::Debug for Scope {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let iter = self.items.iter().map(|(name, item)| (name.as_str(), item));
    write!(f, "{:?}", HashMap::<&str, &Val>::from_iter(iter))
  }
}

impl Clone for Scope {
  /// Clones the scope, using the same Rc's as self
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

  pub fn define(&mut self, name: Symbol, item: Expr) -> Val {
    if let Some(c) = self.items.get(&name) {
      let mut chain = RefCell::borrow_mut(c);
      match chain.is_root() {
        true => {
          chain.set(Some(item));
        }
        false => {
          chain.unlink_with(Some(item));
        }
      }

      c.clone()
    } else {
      let val = Rc::new(RefCell::new(Chain::new(Some(item))));
      self.items.insert(name, val.clone());

      val
    }
  }

  pub fn reserve(&mut self, name: Symbol) {
    self
      .items
      .entry(name)
      .or_insert_with(|| Rc::new(RefCell::new(Chain::new(None))));
  }

  pub fn set(
    &mut self,
    name: Symbol,
    item: Expr,
  ) -> Result<Val, RunErrorReason> {
    if let Some(c) = self.items.get_mut(&name) {
      let mut chain = RefCell::borrow_mut(c);
      chain.set(Some(item));

      Ok(c.clone())
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

  /// Creates a new scope, linking the new symbols to that of self (such as for a function call)
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
pub struct Scanner<'s> {
  pub scope: &'s mut Scope,
}

impl<'s> Scanner<'s> {
  pub fn new(scope: &'s mut Scope) -> Self {
    Self { scope }
  }

  pub fn scan(&mut self, expr: Expr) -> Result<Expr, (Expr, RunErrorReason)> {
    if expr.kind.is_function() {
      let expr = expr;
      if let ExprKind::Function { scope, mut body } = expr.kind {
        for item in body.iter_mut() {
          if item.kind.unlazy().is_function() {
            let mut duplicate = self.scope.duplicate();
            let mut scanner = Scanner::new(&mut duplicate);
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

        let fn_scope = if let FnScope::Scoped(scope) = scope {
          let mut fn_scope = scope.clone();
          fn_scope.merge(self.scope.clone());
          FnScope::Scoped(fn_scope)
        } else {
          FnScope::Scopeless
        };

        let new_expr = ExprKind::Function {
          scope: fn_scope,
          body,
        };

        Ok(Expr {
          kind: new_expr,
          info: expr.info,
        })
      } else {
        // If the expression is not a function, we just return it
        Ok(expr)
      }
    } else {
      Err((expr, RunErrorReason::InvalidFunction))
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn top_level_scopes() {
    let source = Source::new("", "0 'a def");
    let mut lexer = Lexer::new(source);
    let exprs = crate::parser::parse(&mut lexer).unwrap();

    let engine = Engine::new();
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
    let source = Source::new("", "'(fn 0 'a def) call");
    let mut lexer = Lexer::new(source);
    let exprs = crate::parser::parse(&mut lexer).unwrap();

    let engine = Engine::new();
    let mut context = Context::new().with_stack_capacity(32);
    context = engine.run(context, exprs).unwrap();

    assert_eq!(context.scope_item(Symbol::new("a".into())), None);
  }

  #[test]
  fn nested_function_scopes_are_isolated() {
    let source = Source::new(
      "",
      "0 'a def a '(fn 1 'a def a '(fn 2 'a def a) call a) call a",
    );
    let mut lexer = Lexer::new(source);
    let exprs = crate::parser::parse(&mut lexer).unwrap();

    let engine = Engine::new();
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
    let source = Source::new("", "0 'a def a '(fn a 1 'a set a) call a");
    let mut lexer = Lexer::new(source);
    let exprs = crate::parser::parse(&mut lexer).unwrap();

    let engine = Engine::new();
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
    let source = Source::new("", "0 'a def '(fn 1 'a def '(fn a)) call call a");
    let mut lexer = Lexer::new(source);
    let exprs = crate::parser::parse(&mut lexer).unwrap();

    let engine = Engine::new();
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
    let source =
      Source::new("", "0 'a def '(fn 1 'a def '(fn a 2 'a set a)) call call a");
    let mut lexer = Lexer::new(source);
    let exprs = crate::parser::parse(&mut lexer).unwrap();

    let engine = Engine::new();
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
    let source = Source::new("", "'(fn! 0 'a def) call a");
    let mut lexer = Lexer::new(source);
    let exprs = crate::parser::parse(&mut lexer).unwrap();

    let engine = Engine::new();
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

    let source = Source::new("", "0 'a def '(fn! a 1 'a def) call a");
    let mut lexer = Lexer::new(source);
    let exprs = crate::parser::parse(&mut lexer).unwrap();

    let engine = Engine::new();
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
    let source = Source::new("", "'(fn! def) 'define def 0 'a define a");
    let mut lexer = Lexer::new(source);
    let exprs = crate::parser::parse(&mut lexer).unwrap();

    let engine = Engine::new();
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
    let source =
      Source::new("", "'(fn! 0 'a def) '(fn call '(fn a)) call call");
    let mut lexer = Lexer::new(source);
    let exprs = crate::parser::parse(&mut lexer).unwrap();

    let engine = Engine::new();
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

  #[test]
  fn setting_fn_to_var_preserves_scope() {
    let source = Source::new(
      "",
      "'(fn 1 'a def 'f def f) 'test def (fn 0 'a def '(fn a)) test",
    );
    let mut lexer = Lexer::new(source);
    let exprs = crate::parser::parse(&mut lexer).unwrap();

    let engine = Engine::new();
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

  #[test]
  fn calling_function_with_same_var_preserves_scope() {
    let source = Source::new(
      "",
      "'(fn 1 'a def call) 'test def (fn 0 'a def '(fn a)) test",
    );
    let mut lexer = Lexer::new(source);
    let exprs = crate::parser::parse(&mut lexer).unwrap();

    let engine = Engine::new();
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
