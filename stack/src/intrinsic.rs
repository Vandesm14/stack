use core::fmt;

use crate::{
  context::Context,
  engine::{Engine, RunError, RunErrorReason},
  expr::{Expr, ExprInfo, ExprKind},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Intrinsic {
  Add,
  Sub,
  Mul,
  Div,
  Rem,

  Eq,
  Ne,
  Lt,
  Le,
  Gt,
  Ge,

  Or,
  And,

  Assert,

  Drop,
  Dupe,
  Swap,
  Rot,
}

impl Intrinsic {
  pub fn run(
    &self,
    engine: &Engine,
    mut context: Context,
    expr: Expr,
  ) -> Result<Context, RunError> {
    match self {
      Self::Add => {
        let rhs = context.stack_pop(&expr)?;
        let lhs = context.stack_pop(&expr)?;

        let kind = match lhs.kind.clone() + rhs.kind.clone() {
          Ok(res) => res,
          Err(_) => ExprKind::Nil,
        };

        context.stack_push(Expr {
          kind,
          info: engine.track_info().then(|| ExprInfo::Runtime {
            components: vec![rhs, lhs, expr],
          }),
        });

        Ok(context)
      }
      Self::Sub => {
        let rhs = context.stack_pop(&expr)?;
        let lhs = context.stack_pop(&expr)?;

        let kind = match lhs.kind.clone() - rhs.kind.clone() {
          Ok(res) => res,
          Err(_) => ExprKind::Nil,
        };

        context.stack_push(Expr {
          kind,
          info: engine.track_info().then(|| ExprInfo::Runtime {
            components: vec![rhs, lhs, expr],
          }),
        });

        Ok(context)
      }
      Self::Mul => {
        let rhs = context.stack_pop(&expr)?;
        let lhs = context.stack_pop(&expr)?;

        let kind = match lhs.kind.clone() * rhs.kind.clone() {
          Ok(res) => res,
          Err(_) => ExprKind::Nil,
        };

        context.stack_push(Expr {
          kind,
          info: engine.track_info().then(|| ExprInfo::Runtime {
            components: vec![rhs, lhs, expr],
          }),
        });

        Ok(context)
      }
      Self::Div => {
        let rhs = context.stack_pop(&expr)?;
        let lhs = context.stack_pop(&expr)?;

        let kind = match lhs.kind.clone() / rhs.kind.clone() {
          Ok(res) => res,
          Err(_) => ExprKind::Nil,
        };

        context.stack_push(Expr {
          kind,
          info: engine.track_info().then(|| ExprInfo::Runtime {
            components: vec![rhs, lhs, expr],
          }),
        });

        Ok(context)
      }
      Self::Rem => {
        let rhs = context.stack_pop(&expr)?;
        let lhs = context.stack_pop(&expr)?;

        let kind = match lhs.kind.clone() % rhs.kind.clone() {
          Ok(res) => res,
          Err(_) => ExprKind::Nil,
        };

        context.stack_push(Expr {
          kind,
          info: engine.track_info().then(|| ExprInfo::Runtime {
            components: vec![rhs, lhs, expr],
          }),
        });

        Ok(context)
      }

      Self::Eq => {
        let rhs = context.stack_pop(&expr)?;
        let lhs = context.stack_pop(&expr)?;

        let kind = ExprKind::Boolean(lhs.kind == rhs.kind);

        context.stack_push(Expr {
          kind,
          info: engine.track_info().then(|| ExprInfo::Runtime {
            components: vec![rhs, lhs, expr],
          }),
        });

        Ok(context)
      }
      Self::Ne => {
        let rhs = context.stack_pop(&expr)?;
        let lhs = context.stack_pop(&expr)?;

        let kind = ExprKind::Boolean(lhs.kind != rhs.kind);

        context.stack_push(Expr {
          kind,
          info: engine.track_info().then(|| ExprInfo::Runtime {
            components: vec![rhs, lhs, expr],
          }),
        });

        Ok(context)
      }
      Self::Lt => {
        let rhs = context.stack_pop(&expr)?;
        let lhs = context.stack_pop(&expr)?;

        let kind = ExprKind::Boolean(lhs.kind < rhs.kind);

        context.stack_push(Expr {
          kind,
          info: engine.track_info().then(|| ExprInfo::Runtime {
            components: vec![rhs, lhs, expr],
          }),
        });

        Ok(context)
      }
      Self::Le => {
        let rhs = context.stack_pop(&expr)?;
        let lhs = context.stack_pop(&expr)?;

        let kind = ExprKind::Boolean(lhs.kind <= rhs.kind);

        context.stack_push(Expr {
          kind,
          info: engine.track_info().then(|| ExprInfo::Runtime {
            components: vec![rhs, lhs, expr],
          }),
        });

        Ok(context)
      }
      Self::Gt => {
        let rhs = context.stack_pop(&expr)?;
        let lhs = context.stack_pop(&expr)?;

        let kind = ExprKind::Boolean(lhs.kind > rhs.kind);

        context.stack_push(Expr {
          kind,
          info: engine.track_info().then(|| ExprInfo::Runtime {
            components: vec![rhs, lhs, expr],
          }),
        });

        Ok(context)
      }
      Self::Ge => {
        let rhs = context.stack_pop(&expr)?;
        let lhs = context.stack_pop(&expr)?;

        let kind = ExprKind::Boolean(lhs.kind >= rhs.kind);

        context.stack_push(Expr {
          kind,
          info: engine.track_info().then(|| ExprInfo::Runtime {
            components: vec![rhs, lhs, expr],
          }),
        });

        Ok(context)
      }

      Self::Or => {
        let rhs = context.stack_pop(&expr)?;
        let lhs = context.stack_pop(&expr)?;

        let kind =
          ExprKind::Boolean(lhs.kind.is_truthy() || rhs.kind.is_truthy());

        context.stack_push(Expr {
          kind,
          info: engine.track_info().then(|| ExprInfo::Runtime {
            components: vec![rhs, lhs, expr],
          }),
        });

        Ok(context)
      }
      Self::And => {
        let rhs = context.stack_pop(&expr)?;
        let lhs = context.stack_pop(&expr)?;

        let kind =
          ExprKind::Boolean(lhs.kind.is_truthy() && rhs.kind.is_truthy());

        context.stack_push(Expr {
          kind,
          info: engine.track_info().then(|| ExprInfo::Runtime {
            components: vec![rhs, lhs, expr],
          }),
        });

        Ok(context)
      }

      Self::Assert => {
        let bool = context.stack_pop(&expr)?;
        let message = context.stack_pop(&expr)?;

        if bool.kind.is_truthy() {
          Ok(context)
        } else {
          Err(RunError {
            expr: Expr {
              info: engine.track_info().then(|| ExprInfo::Runtime {
                components: vec![message.clone(), bool, expr],
              }),
              kind: message.kind,
            },
            reason: RunErrorReason::AssertionFailed,
          })
        }
      }

      Self::Drop => {
        context.stack_pop(&expr)?;
        Ok(context)
      }
      Self::Dupe => {
        let item = context.stack().last().cloned().ok_or_else(|| RunError {
          reason: RunErrorReason::StackUnderflow,
          expr,
        })?;

        context.stack_push(item);
        Ok(context)
      }
      Self::Swap => {
        let len = context.stack().len();

        if len >= 2 {
          context.stack_mut().swap(len - 1, len - 2);
          Ok(context)
        } else {
          Err(RunError {
            reason: RunErrorReason::StackUnderflow,
            expr,
          })
        }
      }
      Self::Rot => {
        let len = context.stack().len();

        if len >= 3 {
          context.stack_mut().swap(len - 1, len - 3);
          context.stack_mut().swap(len - 2, len - 3);

          Ok(context)
        } else {
          Err(RunError {
            reason: RunErrorReason::StackUnderflow,
            expr,
          })
        }
      }
    }
  }
}

impl fmt::Display for Intrinsic {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Add => write!(f, "+"),
      Self::Sub => write!(f, "-"),
      Self::Mul => write!(f, "*"),
      Self::Div => write!(f, "/"),
      Self::Rem => write!(f, "%"),

      Self::Eq => write!(f, "="),
      Self::Ne => write!(f, "!="),
      Self::Lt => write!(f, "<"),
      Self::Le => write!(f, "<="),
      Self::Gt => write!(f, ">"),
      Self::Ge => write!(f, ">="),

      Self::Or => write!(f, "or"),
      Self::And => write!(f, "and"),

      Self::Assert => write!(f, "assert"),

      Self::Drop => write!(f, "drop"),
      Self::Dupe => write!(f, "dupe"),
      Self::Swap => write!(f, "swap"),
      Self::Rot => write!(f, "rot"),
    }
  }
}
