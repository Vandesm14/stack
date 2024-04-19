use core::{fmt, num::FpCategory};

use unicode_segmentation::UnicodeSegmentation as _;

use crate::{
  context::Context,
  engine::{Engine, RunError, RunErrorReason},
  expr::{Expr, ExprInfo, ExprKind, Symbol},
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

  Len,
  Nth,
  Split,
  Concat,

  Cast,

  If,
  Halt,

  Call,

  Let,
}

impl Intrinsic {
  #[allow(clippy::should_implement_trait)]
  pub fn from_str(s: &str) -> Option<Self> {
    match s {
      "+" => Some(Self::Add),
      "-" => Some(Self::Sub),
      "*" => Some(Self::Mul),
      "/" => Some(Self::Div),
      "%" => Some(Self::Rem),

      "=" => Some(Self::Eq),
      "!=" => Some(Self::Ne),
      "<" => Some(Self::Lt),
      "<=" => Some(Self::Le),
      ">" => Some(Self::Gt),
      ">=" => Some(Self::Ge),

      "or" => Some(Self::Or),
      "and" => Some(Self::And),

      "assert" => Some(Self::Assert),

      "drop" => Some(Self::Drop),
      "dupe" => Some(Self::Dupe),
      "swap" => Some(Self::Swap),
      "rot" => Some(Self::Rot),

      "len" => Some(Self::Len),
      "nth" => Some(Self::Nth),
      "split" => Some(Self::Split),
      "concat" => Some(Self::Concat),

      "cast" => Some(Self::Cast),

      "if" => Some(Self::If),
      "halt" => Some(Self::Halt),

      "call" => Some(Self::Call),

      "let" => Some(Self::Let),

      _ => None,
    }
  }

  pub fn run(
    &self,
    engine: &Engine,
    mut context: Context,
    expr: Expr,
  ) -> Result<Context, RunError> {
    match self {
      // MARK: Add
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
      // MARK: Sub
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
      // MARK: Mul
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
      // MARK: Div
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
      // MARK: Rem
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

      // MARK: Eq
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
      // MARK: Ne
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
      // MARK: Lt
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
      // MARK: Le
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
      // MARK: Gt
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
      // MARK: Ge
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

      // MARK: Or
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
      // MARK: And
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

      // MARK: Assert
      Self::Assert => {
        let bool = context.stack_pop(&expr)?;
        let message = context.stack_pop(&expr)?;

        if bool.kind.is_truthy() {
          Ok(context)
        } else {
          Err(RunError {
            reason: RunErrorReason::AssertionFailed,
            context,
            expr: Expr {
              info: engine.track_info().then(|| ExprInfo::Runtime {
                components: vec![message.clone(), bool, expr],
              }),
              kind: message.kind,
            },
          })
        }
      }

      // MARK: Drop
      Self::Drop => {
        context.stack_pop(&expr)?;
        Ok(context)
      }
      // MARK: Dupe
      Self::Dupe => {
        let item = context.stack().last().cloned().ok_or_else(|| RunError {
          reason: RunErrorReason::StackUnderflow,
          context: context.clone(),
          expr,
        })?;

        context.stack_push(item);
        Ok(context)
      }
      // MARK: Swap
      Self::Swap => {
        let len = context.stack().len();

        if len >= 2 {
          context.stack_mut().swap(len - 1, len - 2);
          Ok(context)
        } else {
          Err(RunError {
            reason: RunErrorReason::StackUnderflow,
            context,
            expr,
          })
        }
      }
      // MARK: Rot
      Self::Rot => {
        let len = context.stack().len();

        if len >= 3 {
          context.stack_mut().swap(len - 1, len - 3);
          context.stack_mut().swap(len - 2, len - 3);

          Ok(context)
        } else {
          Err(RunError {
            reason: RunErrorReason::StackUnderflow,
            context,
            expr,
          })
        }
      }

      // MARK: Len
      Self::Len => {
        let item = context.stack_pop(&expr)?;

        let kind = match item.kind {
          ExprKind::List(ref x) => {
            debug_assert!(x.len() <= i64::MAX as usize);
            ExprKind::Integer(x.len() as i64)
          }
          ExprKind::String(ref x) => {
            let len = x.graphemes(true).count();
            debug_assert!(len <= i64::MAX as usize);
            ExprKind::Integer(len as i64)
          }
          _ => ExprKind::Nil,
        };

        context.stack_push(item.clone());

        context.stack_push(Expr {
          kind,
          info: engine.track_info().then(|| ExprInfo::Runtime {
            components: vec![item, expr],
          }),
        });

        Ok(context)
      }
      // MARK: Nth
      Self::Nth => {
        let index = context.stack_pop(&expr)?;
        let item = context.stack_pop(&expr)?;

        // TODO: Can these eager clones be removed?
        let kind = match (item.kind.clone(), index.kind.clone()) {
          (ExprKind::List(x), ExprKind::Integer(i)) if i >= 0 => x
            .get(i as usize)
            .map(|x| x.kind.clone())
            .unwrap_or(ExprKind::Nil),
          (ExprKind::String(x), ExprKind::Integer(i)) if i >= 0 => x
            .as_str()
            .graphemes(true)
            .nth(i as usize)
            .map(|x| ExprKind::String(x.into()))
            .unwrap_or(ExprKind::Nil),
          _ => ExprKind::Nil,
        };

        context.stack_push(item.clone());

        context.stack_push(Expr {
          kind,
          info: engine.track_info().then(|| ExprInfo::Runtime {
            components: vec![item, index, expr],
          }),
        });

        Ok(context)
      }
      // MARK: Split
      Self::Split => {
        let index = context.stack_pop(&expr)?;
        let item = context.stack_pop(&expr)?;

        // TODO: Can these eager clones be removed?
        match (item.kind.clone(), index.kind.clone()) {
          (ExprKind::List(mut x), ExprKind::Integer(i)) if i >= 0 => {
            if (i as usize) < x.len() {
              let rest = x.split_off(i as usize);

              context.stack_push(Expr {
                kind: ExprKind::List(x),
                info: engine.track_info().then(|| ExprInfo::Runtime {
                  components: vec![item.clone(), index.clone(), expr.clone()],
                }),
              });

              context.stack_push(Expr {
                kind: ExprKind::List(rest),
                info: engine.track_info().then(|| ExprInfo::Runtime {
                  components: vec![item, index, expr],
                }),
              });
            } else {
              context.stack_push(Expr {
                kind: ExprKind::List(x),
                info: engine.track_info().then(|| ExprInfo::Runtime {
                  components: vec![item.clone(), index.clone(), expr.clone()],
                }),
              });

              context.stack_push(Expr {
                kind: ExprKind::Nil,
                info: engine.track_info().then(|| ExprInfo::Runtime {
                  components: vec![item, index, expr],
                }),
              });
            }
          }
          (ExprKind::String(mut x), ExprKind::Integer(i)) if i >= 0 => {
            match x.as_str().grapheme_indices(true).nth(i as usize) {
              Some((i, _)) => {
                let rest = x.split_off(i);

                context.stack_push(Expr {
                  kind: ExprKind::String(x),
                  info: engine.track_info().then(|| ExprInfo::Runtime {
                    components: vec![item.clone(), index.clone(), expr.clone()],
                  }),
                });

                context.stack_push(Expr {
                  kind: ExprKind::String(rest),
                  info: engine.track_info().then(|| ExprInfo::Runtime {
                    components: vec![item, index, expr],
                  }),
                });
              }
              None => {
                context.stack_push(Expr {
                  kind: ExprKind::String(x),
                  info: engine.track_info().then(|| ExprInfo::Runtime {
                    components: vec![item.clone(), index.clone(), expr.clone()],
                  }),
                });

                context.stack_push(Expr {
                  kind: ExprKind::Nil,
                  info: engine.track_info().then(|| ExprInfo::Runtime {
                    components: vec![item, index, expr],
                  }),
                });
              }
            }
          }
          _ => {
            context.stack_push(Expr {
              kind: ExprKind::Nil,
              info: engine.track_info().then(|| ExprInfo::Runtime {
                components: vec![item.clone(), index.clone(), expr.clone()],
              }),
            });

            context.stack_push(Expr {
              kind: ExprKind::Nil,
              info: engine.track_info().then(|| ExprInfo::Runtime {
                components: vec![item, index, expr],
              }),
            });
          }
        }

        Ok(context)
      }
      // MARK: Concat
      Self::Concat => {
        let rhs = context.stack_pop(&expr)?;
        let lhs = context.stack_pop(&expr)?;

        // TODO: Can these eager clones be removed?
        let kind = match (lhs.kind.clone(), rhs.kind.clone()) {
          (ExprKind::List(mut lhs), ExprKind::List(rhs)) => {
            lhs.extend(rhs);
            ExprKind::List(lhs)
          }
          (ExprKind::String(mut lhs), ExprKind::String(rhs)) => {
            lhs.push_str(&rhs);
            ExprKind::String(lhs)
          }
          _ => ExprKind::Nil,
        };

        context.stack_push(Expr {
          kind,
          info: engine.track_info().then(|| ExprInfo::Runtime {
            components: vec![rhs, lhs, expr],
          }),
        });

        Ok(context)
      }

      // MARK: Cast
      Self::Cast => {
        let ty = context.stack_pop(&expr)?;
        let item = context.stack_pop(&expr)?;

        // TODO: Can these eager clones be removed?
        let kind = match ty.kind {
          ExprKind::String(ref x) => match (item.kind.clone(), x.as_str()) {
            (ExprKind::Nil, "boolean") => ExprKind::Boolean(false),
            (ExprKind::Boolean(x), "boolean") => ExprKind::Boolean(x),
            (ExprKind::Integer(x), "boolean") => ExprKind::Boolean(x != 0),
            (ExprKind::Float(x), "boolean") => ExprKind::Boolean(x == 0.0),

            (ExprKind::Nil, "integer") => ExprKind::Integer(0),
            (ExprKind::Boolean(x), "integer") => ExprKind::Integer(x as i64),
            (ExprKind::Integer(x), "integer") => ExprKind::Integer(x),
            (ExprKind::Float(x), "integer") => {
              let x = x.floor();

              match x.classify() {
                FpCategory::Zero => ExprKind::Integer(0),
                FpCategory::Normal
                  if x >= i64::MIN as f64 && x <= i64::MAX as f64 =>
                {
                  ExprKind::Integer(x as i64)
                }
                _ => ExprKind::Nil,
              }
            }

            (ExprKind::Nil, "float") => ExprKind::Float(0.0),
            (ExprKind::Boolean(x), "float") => ExprKind::Float(x as i64 as f64),
            (ExprKind::Integer(x), "float") => ExprKind::Float(x as f64),
            (ExprKind::Float(x), "float") => ExprKind::Float(x),

            (ExprKind::Nil, "string") => ExprKind::String("nil".into()),
            (ExprKind::Boolean(x), "string") => ExprKind::String(x.to_string()),
            (ExprKind::Integer(x), "string") => ExprKind::String(x.to_string()),
            (ExprKind::Float(x), "string") => ExprKind::String(x.to_string()),
            (ExprKind::String(x), "string") => ExprKind::String(x),
            (ExprKind::Symbol(x), "string") => {
              ExprKind::String(x.as_str().into())
            }
            (ExprKind::Intrinsic(x), "string") => {
              ExprKind::String(x.to_string())
            }

            (ExprKind::Nil, "symbol") => ExprKind::Nil,
            (ExprKind::Boolean(x), "symbol") => ExprKind::Boolean(x),
            (ExprKind::String(x), "symbol") => ExprKind::Symbol(Symbol::new(x)),
            (ExprKind::Symbol(x), "symbol") => Self::from_str(x.as_str())
              .map(ExprKind::Intrinsic)
              .unwrap_or(ExprKind::Symbol(x)),
            (ExprKind::Intrinsic(x), "symbol") => ExprKind::Intrinsic(x),

            _ => ExprKind::Nil,
          },
          _ => ExprKind::Nil,
        };

        context.stack_push(Expr {
          kind,
          info: engine.track_info().then(|| ExprInfo::Runtime {
            components: vec![item, ty, expr],
          }),
        });

        Ok(context)
      }

      // MARK: If
      Self::If => {
        let cond = context.stack_pop(&expr)?;
        let body = context.stack_pop(&expr)?;

        if cond.kind.is_truthy() {
          context = engine.run_expr(context, body)?;
        }

        Ok(context)
      }
      // MARK: Halt
      Self::Halt => Err(RunError {
        reason: RunErrorReason::Halt,
        context,
        expr,
      }),

      // MARK: Call
      Self::Call => {
        let item = context.stack_pop(&expr)?;
        engine.run_expr(context, item)
      }

      // MARK: Let
      Self::Let => {
        let names = context.stack_pop(&expr)?;
        let body = context.stack_pop(&expr)?;

        match names.kind {
          ExprKind::List(x) => {
            let x_len = x.len();

            let n = x.into_iter().try_fold(
              Vec::with_capacity(x_len),
              |mut v, x| match x.kind {
                ExprKind::Symbol(x) => {
                  v.push(x);
                  Ok(v)
                }
                _ => Err(RunError {
                  reason: RunErrorReason::InvalidLet,
                  context: context.clone(),
                  expr: expr.clone(),
                }),
              },
            )?;

            context.let_push();

            for name in n.into_iter().rev() {
              let expr = context.stack_pop(&expr)?;
              context.let_set(name, expr);
            }

            context = engine.run_expr(context, body)?;
            context.let_pop().unwrap();

            Ok(context)
          }
          _ => Err(RunError {
            reason: RunErrorReason::InvalidLet,
            context: context.clone(),
            expr: expr.clone(),
          }),
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

      Self::Len => write!(f, "len"),
      Self::Nth => write!(f, "nth"),
      Self::Split => write!(f, "split"),
      Self::Concat => write!(f, "concat"),

      Self::Cast => write!(f, "cast"),

      Self::If => write!(f, "if"),
      Self::Halt => write!(f, "halt"),

      Self::Call => write!(f, "call"),

      Self::Let => write!(f, "let"),
    }
  }
}
