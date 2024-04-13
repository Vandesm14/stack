use core::fmt;

use crate::{
  context::Context,
  engine::{Engine, RunError},
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
}

impl Intrinsic {
  pub fn run(
    &self,
    engine: &Engine,
    mut context: Context,
    expr: Expr,
  ) -> Result<Context, RunError> {
    match self {
      Intrinsic::Add => {
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
      Intrinsic::Sub => {
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
      Intrinsic::Mul => {
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
      Intrinsic::Div => {
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
      Intrinsic::Rem => {
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

      Intrinsic::Eq => {
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
      Intrinsic::Ne => {
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
      Intrinsic::Lt => {
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
      Intrinsic::Le => {
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
      Intrinsic::Gt => {
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
      Intrinsic::Ge => {
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

      Intrinsic::Or => {
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
      Intrinsic::And => {
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
    }
  }
}
