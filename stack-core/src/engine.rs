use core::{fmt, str::FromStr};
use std::{
  collections::HashMap,
  sync::Arc,
  time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

use crate::{
  context::Context,
  expr::{Expr, ExprKind, FnScope},
  intrinsic::Intrinsic,
  journal::JournalOp,
  module::Module,
  symbol::Symbol,
};

#[derive(Clone, Default)]
pub struct Engine {
  modules: HashMap<Symbol, Module>,
  start_time: Option<Instant>,
  timeout: Option<Duration>,
  debug_hook: Option<Arc<dyn Fn(String)>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CallResult {
  Once(Result<Context, RunError>),
  Recur(Context),
  None,
}

impl Engine {
  #[inline]
  pub fn new() -> Self {
    Self {
      modules: HashMap::new(),
      start_time: None,
      timeout: None,
      debug_hook: None,
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
  pub fn with_debug_hook(
    mut self,
    debug_hook: Option<Arc<dyn Fn(String)>>,
  ) -> Self {
    self.debug_hook = debug_hook;
    self
  }

  #[inline]
  pub fn module(&self, symbol: &Symbol) -> Option<&Module> {
    self.modules.get(symbol)
  }

  #[inline]
  pub fn debug_hook(&self) -> Option<Arc<dyn Fn(String)>> {
    self.debug_hook.clone()
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

  pub fn run_with_timeout(
    &mut self,
    mut context: Context,
    exprs: Vec<Expr>,
    timeout: Duration,
  ) -> Result<Context, RunError> {
    self.start_time = Some(Instant::now());
    self.timeout = Some(timeout);

    for expr in exprs {
      context = self.run_expr(context, expr)?;
    }

    Ok(context)
  }

  pub fn call_expr(
    &self,
    mut context: Context,
    expr: Expr,
  ) -> Result<Context, RunError> {
    let expr = context.scan_expr(expr)?;
    match expr.kind {
      ExprKind::List(exprs) => self.run(context, exprs),
      _ => self.run_expr(context, expr),
    }
  }

  pub fn run_expr(
    &self,
    mut context: Context,
    expr: Expr,
  ) -> Result<Context, RunError> {
    if let (Some(start_time), Some(timeout)) = (self.start_time, self.timeout) {
      if start_time.elapsed() > timeout {
        return Err(RunError {
          context,
          expr,
          reason: RunErrorReason::Timeout,
        });
      }
    }

    let expr = context.scan_expr(expr)?;

    if let ExprKind::SExpr { call, body } = &expr.kind {
      if let Some(journal) = context.journal_mut() {
        journal.commit();
        journal.push_op(JournalOp::SCall(expr.clone()));
      }

      let mut args: Vec<Expr> = Vec::new();
      for expr in body {
        let stack_len = context.stack().len();
        match expr.kind {
          ExprKind::Underscore => args.push(context.stack_pop(expr)?),
          ExprKind::SExpr { .. } => {
            context = self.run_expr(context, expr.clone())?;
            args.push(context.stack_pop(expr)?)
          }
          _ => {
            context = self.run_expr(context, expr.clone())?;

            if context.stack().len() != stack_len + 1 {
              todo!("throw an error when stack is different");
            }

            args.push(context.stack_pop(expr)?);
          }
        }
      }

      if let Ok(intrinsic) = Intrinsic::from_str(call.as_str()) {
        if intrinsic.has_flipped_s_expr_args() {
          // TODO: use a for loop and iterate normally, instead of reversing
          args.reverse();
        }
      }

      for expr in args.drain(..) {
        context.stack_push(expr)?;
      }

      return self.run_expr(
        context,
        Expr {
          kind: ExprKind::Symbol(*call),
          info: expr.info,
        },
      );
    }

    match expr.kind {
      ExprKind::Nil
      | ExprKind::Boolean(_)
      | ExprKind::Integer(_)
      | ExprKind::Float(_)
      | ExprKind::String(_)
      | ExprKind::List(_)
      | ExprKind::Record(_) => {
        context.stack_push(expr)?;
        Ok(context)
      }
      // TODO: This is temporary until a proper solution is created.
      ExprKind::Symbol(x) => {
        if let Some(journal) = context.journal_mut() {
          journal.commit();
        }

        if let Ok(intrinsic) = Intrinsic::from_str(x.as_str()) {
          if let Some(journal) = context.journal_mut() {
            journal.commit();
            journal.push_op(JournalOp::FnCall(expr.clone()));
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
              journal.push_op(JournalOp::FnCall(expr.clone()));
            }
            context = func(self, context, expr)?;
            if let Some(journal) = context.journal_mut() {
              journal.commit();
            }
            Ok(context)
          } else {
            Err(RunError {
              context: context.clone(),
              expr,
              reason: RunErrorReason::UnknownCall,
            })
          }
        } else if let Some(item) = context.scope_item(x) {
          if let ExprKind::Function { scope, body } = item.kind {
            let mut _call_result = CallResult::None;
            let mut is_recur = false;
            loop {
              _call_result =
                self.call_fn(&expr, &scope, &body, context, is_recur);
              is_recur = true;

              match _call_result {
                CallResult::Recur(c) => context = c,
                CallResult::Once(result) => return result,
                CallResult::None => unreachable!(),
              }
            }
          }
          if let ExprKind::SExpr { .. } = item.kind {
            self.call_expr(context, item)
          } else {
            if let Some(journal) = context.journal_mut() {
              journal.push_op(JournalOp::Call(expr.clone()));
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
        context.stack_push(*x.clone())?;
        Ok(context)
      }
      ExprKind::Function {
        ref scope,
        ref body,
      } => {
        let mut _call_result = CallResult::None;
        let mut is_recur = false;
        loop {
          _call_result = self.call_fn(&expr, scope, body, context, is_recur);
          is_recur = true;

          match _call_result {
            CallResult::Recur(c) => context = c,
            CallResult::Once(result) => return result,
            CallResult::None => unreachable!(),
          }
        }
      }
      ExprKind::SExpr { .. } => Ok(context),
      ExprKind::Underscore => Ok(context),
    }
  }

  /// Handles auto-calling symbols (calls) when they're pushed to the stack
  /// This is also triggered by the `call` keyword
  pub fn call_fn(
    &self,
    expr: &Expr,
    fn_scope: &FnScope,
    fn_body: &[Expr],
    mut context: Context,
    is_recur: bool,
  ) -> CallResult {
    if let Some(journal) = context.journal_mut() {
      journal.push_op(JournalOp::FnCall(expr.clone()));
    }

    if !is_recur {
      if let FnScope::Scoped(scope) = fn_scope {
        context.push_scope(scope.clone());
      }
    }

    if context.journal().is_some() {
      if fn_scope.is_scoped() {
        let scope = context.scope().clone();
        let journal = context.journal_mut().as_mut().unwrap();
        journal.commit();
        journal
          .push_op(JournalOp::ScopedFnStart(expr.info.clone(), scope.into()));
      } else {
        let journal = context.journal_mut().as_mut().unwrap();
        journal.commit();
        journal.push_op(JournalOp::ScopelessFnStart(expr.info.clone()));
      }
    }

    match self.run(context, fn_body.to_vec()) {
      Ok(mut context) => {
        if context.journal().is_some() {
          let scope = context.scope().clone();
          let journal = context.journal_mut().as_mut().unwrap();
          journal.commit();
          journal.push_op(JournalOp::FnEnd(expr.info.clone(), scope.into()));
        }

        if context.stack().last().map(|e| &e.kind)
          == Some(&ExprKind::Symbol(Symbol::from_ref("recur")))
        {
          return match context.stack_pop(expr) {
            Ok(_) => CallResult::Recur(context),
            Err(err) => CallResult::Once(Err(err)),
          };
        }

        if fn_scope.is_scoped() {
          context.pop_scope();
        }

        CallResult::Once(Ok(context))
      }
      Err(err) => CallResult::Once(Err(err)),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RunErrorReason {
  StackUnderflow,
  DoubleError,
  AssertionFailed,
  Halt,
  InvalidLet,
  Timeout,

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
      Self::Timeout => write!(f, "exceeded timeout"),
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

  // TODO: Move test for scopes/vars into src/scope.rs?
  #[test]
  fn can_define_vars() {
    let source = Source::new("", "0 'a def a");
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
    let source = Source::new("", "0 'a def a 1 'a def a");
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
    let source = Source::new("", "0 'a def a 1 'a set a");
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
    let source = Source::new("", "10 2 [a b -] [a b] let");
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
    let source = Source::new("", "0 'a def 1 [a] [a] let");
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
  fn lets_do_not_act_as_overlays() {
    let source = Source::new("", "0 'a def 1 [a 2 'a def a] [a] let a");
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
  fn functions_work_in_lets() {
    let source = Source::new("", "0 'a def 1 [(fn a 2 'a def a)] [a] let a");
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
  fn lets_dont_leak() {
    let source = Source::new(
      "",
      "0 'a def
      1 [a] [a] let
      1 [(fn! a)] [a] let
      1 [(fn a)] [a] let
      a",
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
        &ExprKind::Integer(1),
        &ExprKind::Integer(1),
        &ExprKind::Integer(1),
        &ExprKind::Integer(0),
      ]
    );
  }

  #[test]
  fn lets_can_set() {
    let source = Source::new("", "1 [a 2 'a set a] [a] let");
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
      vec![&ExprKind::Integer(1), &ExprKind::Integer(2),]
    );
  }
}
