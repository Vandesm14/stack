use core::fmt;
use std::collections::HashMap;

use crate::{
  context::Context,
  expr::{
    vec_fn_body, vec_fn_symbol, vec_is_function, Expr, ExprInfo, ExprKind,
    FnIdent, Symbol,
  },
  journal::{self, JournalOp},
  module::Module,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Engine {
  modules: HashMap<Symbol, Module>,
  track_info: bool,
}

impl Engine {
  #[inline]
  pub fn new() -> Self {
    Self {
      modules: HashMap::new(),
      track_info: true,
    }
  }

  #[inline]
  pub fn with_module(mut self, module: Module) -> Self {
    self.add_module(module);
    self
  }

  #[inline]
  pub fn with_track_info(mut self, track_info: bool) -> Self {
    self.set_track_info(track_info);
    self
  }

  #[inline]
  pub fn add_module(&mut self, module: Module) -> &mut Self {
    self.modules.insert(module.name(), module);
    self
  }

  #[inline]
  pub fn set_track_info(&mut self, track_info: bool) -> &mut Self {
    self.track_info = track_info;
    self
  }

  #[inline]
  pub const fn track_info(&self) -> bool {
    self.track_info
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
        if let Some((namespace, func)) = x.split_once(':') {
          if let Some(func) = self
            .modules
            .get(&Symbol::from_ref(namespace))
            .and_then(|module| module.func(Symbol::from_ref(func)))
          {
            context = func(self, context, expr)?;
            Ok(context)
          } else {
            context.stack_push(Expr {
              kind: ExprKind::Error(Box::new(Expr {
                kind: ExprKind::String("unknown function".into()),
                info: self.track_info.then(|| ExprInfo::Runtime {
                  components: vec![expr.clone()],
                }),
              })),
              info: self.track_info.then(|| ExprInfo::Runtime {
                components: vec![expr],
              }),
            })?;

            Ok(context)
          }
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
        } else if let Some(r#let) = context.let_get(x).cloned() {
          context.stack_push(r#let)?;
          Ok(context)
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
      ExprKind::List(x) => match vec_is_function(&x) {
        true => {
          let fn_ident = vec_fn_symbol(&x).unwrap();
          let fn_body = vec_fn_body(&x).unwrap();
          self.call_fn(&expr, fn_ident, fn_body, context)
        }
        false => self.run(context, x),
      },
      ExprKind::Intrinsic(x) => x.run(self, context, expr),
      ExprKind::Fn(_) => todo!(),
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
