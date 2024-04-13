use core::fmt;
use std::collections::HashMap;

use crate::{
  context::Context,
  expr::{Expr, ExprInfo, ExprKind, Symbol},
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
        context.stack_push(expr);
        Ok(context)
      }
      ExprKind::Error(_) => Err(RunError {
        reason: RunErrorReason::DoubleError,
        context,
        expr,
      }),
      // TODO: This is temporary until a proper solution is created.
      ExprKind::Symbol(x) => {
        if let Some((a, b)) = x.split_once(':') {
          if let Some(func) = self
            .modules
            .get(&Symbol::from_ref(a))
            .and_then(|module| module.func(Symbol::from_ref(b)))
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
            });

            Ok(context)
          }
        } else {
          todo!()
        }
      }
      ExprKind::Lazy(x) => {
        context.stack_push(*x);
        Ok(context)
      }
      ExprKind::List(ref x) => {
        let mut list_context = Context::new();
        list_context = self.run(list_context, x.clone())?;

        context.stack_push(Expr {
          kind: ExprKind::List(core::mem::take(list_context.stack_mut())),
          info: self.track_info.then(|| ExprInfo::Runtime {
            components: vec![expr],
          }),
        });

        Ok(context)
      }
      ExprKind::Intrinsic(x) => x.run(self, context, expr),
      ExprKind::Fn(_) => todo!(),
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
}

impl std::error::Error for RunErrorReason {}

impl fmt::Display for RunErrorReason {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::StackUnderflow => write!(f, "stack underflow"),
      Self::DoubleError => write!(f, "double error"),
      Self::AssertionFailed => write!(f, "assertion failed"),
      Self::Halt => write!(f, "halt"),
    }
  }
}
