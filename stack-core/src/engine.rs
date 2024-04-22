use core::{fmt, str::FromStr};
use std::collections::HashMap;

use crate::{
  context::Context,
  expr::{
    vec_fn_body, vec_fn_symbol, vec_is_function, Error, Expr, ExprKind, FnIdent,
  },
  intrinsic::Intrinsic,
  journal::JournalOp,
  module::Module,
  symbol::Symbol,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Engine {
  modules: HashMap<Symbol, Module>,
}

impl Engine {
  #[inline]
  pub fn new() -> Self {
    Self {
      modules: HashMap::new(),
    }
  }

  #[inline]
  pub fn with_module(mut self, module: Module) -> Self {
    self.add_module(module);
    self
  }

  #[inline]
  pub fn add_module(&mut self, module: Module) -> &mut Self {
    self.modules.insert(module.name(), module);
    self
  }

  #[inline]
  pub fn module(&self, symbol: &Symbol) -> Option<&Module> {
    self.modules.get(symbol)
  }

  pub fn run(
    &self,
    mut context: Context,
    exprs: Vec<Expr>,
  ) -> Result<Context, RunError> {
    for expr in exprs {
      context = self.run_expr(context, expr)?;
    }

    Ok(context)
  }

  #[allow(clippy::only_used_in_recursion)]
  pub fn run_expr(
    &self,
    mut context: Context,
    expr: Expr,
  ) -> Result<Context, RunError> {
    match expr.kind {
      ExprKind::Nil
      | ExprKind::Boolean(_)
      | ExprKind::Integer(_)
      | ExprKind::Float(_)
      | ExprKind::String(_) => {
        context.stack_push(expr)?;
        Ok(context)
      }
      ExprKind::Error(_) => Err(RunError {
        reason: RunErrorReason::DoubleError,
        context,
        expr,
      }),
      // TODO: This is temporary until a proper solution is created.
      ExprKind::Symbol(x) => {
        if let Some(journal) = context.journal_mut() {
          journal.commit();
        }

        if let Ok(intrinsic) = Intrinsic::from_str(x.as_str()) {
          if let Some(journal) = context.journal_mut() {
            journal.commit();
            journal.op(JournalOp::FnCall(expr.clone()));
          }
          let mut context = intrinsic.run(self, context, expr)?;
          if let Some(journal) = context.journal_mut() {
            journal.commit();
          }

          Ok(context)
        } else if let Some((namespace, func)) = x.as_str().split_once(':') {
          if let Some(func) = self
            .modules
            .get(&Symbol::from_ref(namespace))
            .and_then(|module| module.func(Symbol::from_ref(func)))
          {
            if let Some(journal) = context.journal_mut() {
              journal.op(JournalOp::FnCall(expr.clone()));
            }
            context = func(self, context, expr)?;
            if let Some(journal) = context.journal_mut() {
              journal.commit();
            }
            Ok(context)
          } else {
            context.stack_push(Expr {
              kind: ExprKind::Error(Error::new("unknown function".into())),
              info: None,
            })?;

            Ok(context)
          }
        } else if let Some(r#let) = context.let_get(x).cloned() {
          context.stack_push(r#let)?;
          Ok(context)
        } else if let Some(item) = context.scope_item(x) {
          if item.kind.is_function() {
            let fn_ident = item.kind.fn_symbol().unwrap();
            let fn_body = item.kind.fn_body().unwrap();
            self.call_fn(&expr, fn_ident, fn_body, context)
          } else {
            if let Some(journal) = context.journal_mut() {
              journal.op(JournalOp::Call(expr.clone()));
            }
            let result = context.stack_push(item);
            if let Some(journal) = context.journal_mut() {
              journal.commit();
            }

            result.map(|_| context)
          }
        } else {
          Err(RunError {
            context: context.clone(),
            expr,
            reason: RunErrorReason::UnknownCall,
          })
        }
      }
      ExprKind::Lazy(x) => {
        context.stack_push(*x)?;
        Ok(context)
      }
      ExprKind::List(ref x) => match vec_is_function(x) {
        true => {
          let fn_ident = vec_fn_symbol(x).unwrap();
          let fn_body = vec_fn_body(x).unwrap();
          self.call_fn(&expr, fn_ident, fn_body, context)
        }
        false => self.run(context, x.to_vec()),
      },
      ExprKind::Fn(_) => Ok(context),
    }
  }

  /// Handles auto-calling symbols (calls) when they're pushed to the stack
  /// This is also triggered by the `call` keyword
  pub fn call_fn(
    &self,
    expr: &Expr,
    fn_ident: &FnIdent,
    fn_body: &[Expr],
    mut context: Context,
  ) -> Result<Context, RunError> {
    if let Some(journal) = context.journal_mut() {
      journal.op(JournalOp::FnCall(expr.clone()));
    }

    if fn_ident.scoped {
      context.push_scope(fn_ident.scope.clone());
    }

    if let Some(journal) = context.journal_mut() {
      journal.commit();
      journal.op(JournalOp::FnStart(fn_ident.scoped));
    }

    match self.run(context, fn_body.to_vec()) {
      Ok(mut context) => {
        if let Some(journal) = context.journal_mut() {
          journal.commit();
          journal.op(JournalOp::FnEnd);
        }

        if fn_ident.scoped {
          context.pop_scope();
        }

        Ok(context)
      }
      Err(err) => Err(err),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunError {
  pub reason: RunErrorReason,
  pub context: Context,
  pub expr: Expr,
}

impl std::error::Error for RunError {}

impl fmt::Display for RunError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{} caused by ", self.reason)?;

    if let Some(ref info) = self.expr.info {
      write!(f, "{}", info)
    } else {
      write!(f, "{}", self.expr)
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RunErrorReason {
  StackUnderflow,
  DoubleError,
  AssertionFailed,
  Halt,
  InvalidLet,

  // Scope Errors
  UnknownCall,
  InvalidDefinition,
  InvalidFunction,
  CannotSetBeforeDef,
}

impl std::error::Error for RunErrorReason {}

impl fmt::Display for RunErrorReason {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::StackUnderflow => write!(f, "stack underflow"),
      Self::DoubleError => write!(f, "double error"),
      Self::AssertionFailed => write!(f, "assertion failed"),
      Self::Halt => write!(f, "halt"),
      Self::InvalidLet => write!(f, "invalid let"),
      Self::UnknownCall => write!(f, "unknown call"),
      Self::InvalidDefinition => write!(f, "invalid definition"),
      Self::InvalidFunction => write!(f, "invalid function"),
      Self::CannotSetBeforeDef => {
        write!(f, "cannot set to a nonexistent variable")
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::prelude::*;
  use std::rc::Rc;

  // TODO: Move test for scopes/vars into src/scope.rs?
  #[test]
  fn can_define_vars() {
    let source = Rc::new(Source::new(Symbol::from_ref(""), "0 'a def a"));
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
  }

  #[test]
  fn can_redefine_vars() {
    let source =
      Rc::new(Source::new(Symbol::from_ref(""), "0 'a def a 1 'a def a"));
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
  fn can_set_vars() {
    let source =
      Rc::new(Source::new(Symbol::from_ref(""), "0 'a def a 1 'a set a"));
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

  // TODO: Move test for lets into a better place?
  #[test]
  fn can_use_lets() {
    let source = Rc::new(Source::new(
      Symbol::from_ref(""),
      "10 2 '(a b -) '(a b) let",
    ));
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
      vec![&ExprKind::Integer(8),]
    );
  }

  #[test]
  fn lets_take_precedence_over_scope() {
    let source = Rc::new(Source::new(
      Symbol::from_ref(""),
      "0 'a def 1 '(a) '(a) let",
    ));
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
      vec![&ExprKind::Integer(1),]
    );
  }

  #[test]
  fn lets_act_as_overlays() {
    let source = Rc::new(Source::new(
      Symbol::from_ref(""),
      "0 'a def 1 '(a 2 'a def a) '(a) let a",
    ));
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
        &ExprKind::Integer(2),
      ]
    );
  }

  #[test]
  fn functions_work_in_lets() {
    let source = Rc::new(Source::new(
      Symbol::from_ref(""),
      "0 'a def 1 '(fn a 2 'a def a) '(a) let a",
    ));
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
  fn scopeless_functions_work_in_lets() {
    let source = Rc::new(Source::new(
      Symbol::from_ref(""),
      "0 'a def 1 '(fn! a 2 'a def a) '(a) let a",
    ));
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
        &ExprKind::Integer(2),
      ]
    );
  }

  #[test]
  fn lets_dont_leak() {
    let source = Rc::new(Source::new(
      Symbol::from_ref(""),
      "0 'a def
      1 '(a) '(a) let
      1 '(fn! a) '(a) let
      1 '(fn a) '(a) let
      a",
    ));
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
        &ExprKind::Integer(1),
        &ExprKind::Integer(1),
        &ExprKind::Integer(0),
      ]
    );
  }

  #[test]
  fn lets_cant_set() {
    let source =
      Rc::new(Source::new(Symbol::from_ref(""), "1 '(2 'a set) '(a) let"));
    let mut lexer = Lexer::new(source);
    let exprs = crate::parser::parse(&mut lexer).unwrap();

    let engine = Engine::new();
    let context = Context::new().with_stack_capacity(32);
    assert_eq!(
      engine.run(context, exprs).map_err(|err| err.reason),
      Err(RunErrorReason::CannotSetBeforeDef)
    );
  }
}