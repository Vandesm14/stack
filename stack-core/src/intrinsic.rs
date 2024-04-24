use core::{fmt, num::FpCategory, str::FromStr};
use std::collections::HashMap;

use compact_str::ToCompactString;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
  context::Context,
  expr::{Expr, ExprKind},
  journal::JournalOp,
  lexer::Lexer,
  prelude::{parse, Engine, RunError, RunErrorReason},
  source::Source,
  symbol::Symbol,
};

macro_rules! intrinsics {
  ($($ident:ident => $s:literal),* $(,)?) => {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Intrinsic {
      $($ident),*
    }

    impl Intrinsic {
      /// Returns the <code>&[str]</code> name of this [`Intrinsic`].
      pub const fn as_str(self) -> &'static str {
        match self {
          $(Self::$ident => $s),*
        }
      }
    }

    impl FromStr for Intrinsic {
      type Err = ParseIntrinsicError;

      fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
          $($s => Ok(Self::$ident),)*
          _ => Err(ParseIntrinsicError),
        }
      }
    }

    impl fmt::Display for Intrinsic {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
      }
    }
  };
}

intrinsics! {
  Add => "+",
  Sub => "-",
  Mul => "*",
  Div => "/",
  Rem => "%",

  Eq => "=",
  Ne => "!=",
  Lt => "<",
  Le => "<=",
  Gt => ">",
  Ge => ">=",

  Or => "or",
  And => "and",
  Not => "not",

  Assert => "assert",

  Drop => "drop",
  Dupe => "dupe",
  Swap => "swap",
  Rot => "rot",

  Len => "len",
  Nth => "nth",
  Split => "split",
  Concat => "concat",
  Push => "push",
  Pop => "pop",

  Insert => "insert",
  Prop => "prop",
  Has => "has",
  Remove => "remove",
  Keys => "keys",
  Values => "values",

  Cast => "cast",
  Lazy => "lazy",

  If => "if",
  Halt => "halt",

  Call => "call",

  Let => "let",
  Def => "def",
  Set => "set",
  Get => "get",

  Print => "print",
  Pretty => "pretty",
  Recur => "recur",

  OrElse => "orelse",

  Import => "import",
}

impl Intrinsic {
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

        context.stack_push(Expr { kind, info: None })?;

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

        context.stack_push(Expr { kind, info: None })?;

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

        context.stack_push(Expr { kind, info: None })?;

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

        context.stack_push(Expr { kind, info: None })?;

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

        context.stack_push(Expr { kind, info: None })?;

        Ok(context)
      }

      // MARK: Eq
      Self::Eq => {
        let rhs = context.stack_pop(&expr)?;
        let lhs = context.stack_pop(&expr)?;

        let kind = ExprKind::Boolean(lhs.kind == rhs.kind);

        context.stack_push(Expr { kind, info: None })?;

        Ok(context)
      }
      // MARK: Ne
      Self::Ne => {
        let rhs = context.stack_pop(&expr)?;
        let lhs = context.stack_pop(&expr)?;

        let kind = ExprKind::Boolean(lhs.kind != rhs.kind);

        context.stack_push(Expr { kind, info: None })?;

        Ok(context)
      }
      // MARK: Lt
      Self::Lt => {
        let rhs = context.stack_pop(&expr)?;
        let lhs = context.stack_pop(&expr)?;

        let kind = ExprKind::Boolean(lhs.kind < rhs.kind);

        context.stack_push(Expr { kind, info: None })?;

        Ok(context)
      }
      // MARK: Le
      Self::Le => {
        let rhs = context.stack_pop(&expr)?;
        let lhs = context.stack_pop(&expr)?;

        let kind = ExprKind::Boolean(lhs.kind <= rhs.kind);

        context.stack_push(Expr { kind, info: None })?;

        Ok(context)
      }
      // MARK: Gt
      Self::Gt => {
        let rhs = context.stack_pop(&expr)?;
        let lhs = context.stack_pop(&expr)?;

        let kind = ExprKind::Boolean(lhs.kind > rhs.kind);

        context.stack_push(Expr { kind, info: None })?;

        Ok(context)
      }
      // MARK: Ge
      Self::Ge => {
        let rhs = context.stack_pop(&expr)?;
        let lhs = context.stack_pop(&expr)?;

        let kind = ExprKind::Boolean(lhs.kind >= rhs.kind);

        context.stack_push(Expr { kind, info: None })?;

        Ok(context)
      }

      // MARK: Or
      Self::Or => {
        let rhs = context.stack_pop(&expr)?;
        let lhs = context.stack_pop(&expr)?;

        let kind =
          ExprKind::Boolean(lhs.kind.is_truthy() || rhs.kind.is_truthy());

        context.stack_push(Expr { kind, info: None })?;

        Ok(context)
      }
      // MARK: And
      Self::And => {
        let rhs = context.stack_pop(&expr)?;
        let lhs = context.stack_pop(&expr)?;

        let kind =
          ExprKind::Boolean(lhs.kind.is_truthy() && rhs.kind.is_truthy());

        context.stack_push(Expr { kind, info: None })?;

        Ok(context)
      }
      // MARK: Not
      Self::Not => {
        let rhs = context.stack_pop(&expr)?;

        let kind = ExprKind::Boolean(!rhs.kind.is_truthy());

        context.stack_push(Expr { kind, info: None })?;

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
              info: None,
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

        context.stack_push(item)?;
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
          ExprKind::Record(ref x) => {
            debug_assert!(x.len() <= i64::MAX as usize);
            ExprKind::Integer(x.len() as i64)
          }
          _ => ExprKind::Nil,
        };

        context.stack_push(item.clone())?;

        context.stack_push(Expr { kind, info: None })?;

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

        context.stack_push(item.clone())?;

        context.stack_push(Expr { kind, info: None })?;

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
                info: None,
              })?;

              context.stack_push(Expr {
                kind: ExprKind::List(rest),
                info: None,
              })?;
            } else {
              context.stack_push(Expr {
                kind: ExprKind::List(x),
                info: None,
              })?;

              context.stack_push(Expr {
                kind: ExprKind::Nil,
                info: None,
              })?;
            }
          }
          (ExprKind::String(mut x), ExprKind::Integer(i)) if i >= 0 => {
            match x.as_str().grapheme_indices(true).nth(i as usize) {
              Some((i, _)) => {
                let rest = x.split_off(i);

                context.stack_push(Expr {
                  kind: ExprKind::String(x),
                  info: None,
                })?;

                context.stack_push(Expr {
                  kind: ExprKind::String(rest),
                  info: None,
                })?;
              }
              None => {
                context.stack_push(Expr {
                  kind: ExprKind::String(x),
                  info: None,
                })?;

                context.stack_push(Expr {
                  kind: ExprKind::Nil,
                  info: None,
                })?;
              }
            }
          }
          _ => {
            context.stack_push(Expr {
              kind: ExprKind::Nil,
              info: None,
            })?;

            context.stack_push(Expr {
              kind: ExprKind::Nil,
              info: None,
            })?;
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

        context.stack_push(Expr { kind, info: None })?;

        Ok(context)
      }
      // MARK: Push
      Self::Push => {
        let list = context.stack_pop(&expr)?;
        let item = context.stack_pop(&expr)?;

        let kind = match (list.kind.clone(), item.kind.clone()) {
          (ExprKind::List(mut x), i) => {
            x.push(Expr {
              kind: i,
              info: item.info.clone(),
            });
            ExprKind::List(x)
          }
          (ExprKind::String(mut x), ExprKind::String(s)) => {
            x.push_str(&s);
            ExprKind::String(x)
          }
          (ExprKind::String(mut x), ExprKind::Integer(c))
            if c >= 0 && c <= u32::MAX as i64 =>
          {
            if let Some(c) = char::from_u32(c as u32) {
              x.push(c);
              ExprKind::String(x)
            } else {
              ExprKind::Nil
            }
          }
          _ => ExprKind::Nil,
        };

        context.stack_push(Expr { kind, info: None })?;

        Ok(context)
      }
      // MARK: Pop
      Self::Pop => {
        let list = context.stack_pop(&expr)?;

        match list.kind.clone() {
          ExprKind::List(mut x) => {
            let e = x.pop().unwrap_or(Expr {
              kind: ExprKind::Nil,
              info: None,
            });

            context.stack_push(Expr {
              kind: ExprKind::List(x),
              info: list.info,
            })?;
            context.stack_push(e)?;
          }
          ExprKind::String(mut x) => {
            let e = x
              .pop()
              .map(|e| Expr {
                kind: ExprKind::String(e.to_compact_string()),
                info: None,
              })
              .unwrap_or(Expr {
                kind: ExprKind::Nil,
                info: None,
              });

            context.stack_push(Expr {
              kind: ExprKind::String(x),
              info: list.info,
            })?;
            context.stack_push(e)?;
          }
          _ => {
            context.stack_push(list.clone())?;
            context.stack_push(Expr {
              kind: ExprKind::Nil,
              info: None,
            })?;
          }
        }

        Ok(context)
      }

      // MARK: Insert
      Self::Insert => {
        let record = context.stack_pop(&expr)?;
        let value = context.stack_pop(&expr)?;
        let name = context.stack_pop(&expr)?;

        match record.kind {
          ExprKind::Record(ref record) => {
            let symbol = match name.kind {
              ExprKind::Symbol(s) => s,
              ExprKind::String(s) => Symbol::from_ref(s.as_str()),
              _ => {
                return Err(RunError {
                  context,
                  expr,
                  reason: RunErrorReason::UnknownCall,
                })
              }
            };

            let mut new_record = record.clone();
            new_record.insert(symbol, value);

            context.stack_push(Expr {
              kind: ExprKind::Record(new_record),
              info: None,
            })?;

            Ok(())
          }
          _ => context.stack_push(Expr {
            kind: ExprKind::Nil,
            info: None,
          }),
        }
        .map(|_| context)
      }
      // MARK: Prop
      Self::Prop => {
        let symbol = context.stack_pop(&expr)?;
        let record = context.stack_pop(&expr)?;

        match record.kind {
          ExprKind::Record(ref r) => {
            let symbol = match symbol.kind {
              ExprKind::Symbol(s) => s,
              ExprKind::String(s) => Symbol::from_ref(s.as_str()),
              _ => {
                return Err(RunError {
                  context,
                  expr,
                  reason: RunErrorReason::UnknownCall,
                })
              }
            };

            let result = r.get(&symbol).unwrap_or_else(|| &Expr {
              info: None,
              kind: ExprKind::Nil,
            });

            context.stack_push(record.clone())?;
            context.stack_push(result.clone())?;

            Ok(())
          }
          _ => context.stack_push(Expr {
            kind: ExprKind::Nil,
            info: None,
          }),
        }
        .map(|_| context)
      }
      // MARK: Has
      Self::Has => {
        let symbol = context.stack_pop(&expr)?;
        let record = context.stack_pop(&expr)?;

        match record.kind {
          ExprKind::Record(ref r) => {
            let symbol = match symbol.kind {
              ExprKind::Symbol(s) => s,
              ExprKind::String(s) => Symbol::from_ref(s.as_str()),
              _ => {
                return Err(RunError {
                  context,
                  expr,
                  reason: RunErrorReason::UnknownCall,
                })
              }
            };

            let result = r.contains_key(&symbol);

            context.stack_push(record.clone())?;
            context.stack_push(Expr {
              info: None,
              kind: ExprKind::Boolean(result),
            })?;

            Ok(())
          }
          _ => context.stack_push(Expr {
            kind: ExprKind::Nil,
            info: None,
          }),
        }
        .map(|_| context)
      }
      // MARK: Remove
      Self::Remove => {
        let record = context.stack_pop(&expr)?;
        let name = context.stack_pop(&expr)?;

        match record.kind {
          ExprKind::Record(ref record) => {
            let symbol = match name.kind {
              ExprKind::Symbol(s) => s,
              ExprKind::String(s) => Symbol::from_ref(s.as_str()),
              _ => {
                return Err(RunError {
                  context,
                  expr,
                  reason: RunErrorReason::UnknownCall,
                })
              }
            };

            let mut new_record = record.clone();
            new_record.remove(&symbol);

            context.stack_push(Expr {
              kind: ExprKind::Record(new_record),
              info: None,
            })?;

            Ok(())
          }
          _ => context.stack_push(Expr {
            kind: ExprKind::Nil,
            info: None,
          }),
        }
        .map(|_| context)
      }
      // MARK: Keys
      Self::Keys => {
        let record = context.stack_pop(&expr)?;

        match record.kind {
          ExprKind::Record(ref r) => {
            let result = r
              .keys()
              .copied()
              .map(|s| Expr {
                info: None,
                kind: ExprKind::Symbol(s),
              })
              .collect::<Vec<_>>();

            context.stack_push(record.clone())?;
            context.stack_push(Expr {
              info: None,
              kind: ExprKind::List(result),
            })?;

            Ok(())
          }
          _ => context.stack_push(Expr {
            kind: ExprKind::Nil,
            info: None,
          }),
        }
        .map(|_| context)
      }
      // MARK: Values
      Self::Values => {
        let record = context.stack_pop(&expr)?;

        match record.kind {
          ExprKind::Record(ref r) => {
            let result = r.values().cloned().collect::<Vec<_>>();

            context.stack_push(record.clone())?;
            context.stack_push(Expr {
              info: None,
              kind: ExprKind::List(result),
            })?;

            Ok(())
          }
          _ => context.stack_push(Expr {
            kind: ExprKind::Nil,
            info: None,
          }),
        }
        .map(|_| context)
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
            (ExprKind::Boolean(x), "string") => {
              ExprKind::String(x.to_compact_string())
            }
            (ExprKind::Integer(x), "string") => {
              ExprKind::String(x.to_compact_string())
            }
            (ExprKind::Float(x), "string") => {
              ExprKind::String(x.to_compact_string())
            }
            (ExprKind::String(x), "string") => ExprKind::String(x),
            (ExprKind::Symbol(x), "string") => {
              ExprKind::String(x.as_str().into())
            }

            // TODO: Make sure these are correct, because the logic is pretty
            //       nuanced in terms of when to choose a Symbol or Intrinsic.
            (ExprKind::Nil, "symbol") => ExprKind::Nil,
            (ExprKind::Boolean(x), "symbol") => ExprKind::Boolean(x),
            // TODO: Handle conversion into `fn` and `fn!`.
            (ExprKind::String(x), "symbol") => ExprKind::Symbol(Symbol::new(x)),
            (ExprKind::Symbol(x), "symbol") => ExprKind::Symbol(x),

            (ExprKind::Record(x), "record") => ExprKind::Record(x),
            (ExprKind::Record(x), "list") => {
              let mut list: Vec<Expr> = Vec::new();
              x.into_iter().for_each(|(key, value)| {
                list.push(Expr {
                  info: None,
                  kind: ExprKind::List(vec![
                    Expr {
                      info: None,
                      kind: ExprKind::Symbol(key),
                    },
                    value,
                  ]),
                });
              });

              ExprKind::List(list)
            }

            (ExprKind::List(x), "record") => {
              let mut record: HashMap<Symbol, Expr> = HashMap::new();
              x.into_iter().for_each(|item| {
                if let ExprKind::List(chunk) = item.kind {
                  let key =
                    Symbol::from_ref(chunk[0].kind.to_string().as_str());
                  let value = &chunk[1];
                  record.insert(key, value.clone());
                }
              });

              ExprKind::Record(record)
            }

            _ => ExprKind::Nil,
          },
          _ => ExprKind::Nil,
        };

        context.stack_push(Expr { kind, info: None })?;

        Ok(context)
      }

      // MARK: Lazy
      Self::Lazy => {
        let expr = context.stack_pop(&expr)?;

        context.stack_push(Expr {
          kind: ExprKind::Lazy(Box::new(expr)),
          info: None,
        })?;

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

            if let Some(journal) = context.journal_mut() {
              journal.commit();
              journal.op(JournalOp::FnStart(false));
            }

            context = engine.run_expr(context, body)?;
            context.let_pop().unwrap();

            if let Some(journal) = context.journal_mut() {
              journal.commit();
              journal.op(JournalOp::FnEnd);
            }

            Ok(context)
          }
          _ => Err(RunError {
            reason: RunErrorReason::InvalidLet,
            context: context.clone(),
            expr: expr.clone(),
          }),
        }
      }

      // MARK: Def
      Self::Def => {
        let name = context.stack_pop(&expr)?;
        let value = context.stack_pop(&expr)?;

        match name.kind {
          ExprKind::Symbol(symbol) => {
            context.def_scope_item(symbol, value);

            Ok(context)
          }
          _ => Err(RunError {
            reason: RunErrorReason::InvalidDefinition,
            context: context.clone(),
            expr: expr.clone(),
          }),
        }
      }

      // MARK: Set
      Self::Set => {
        let name = context.stack_pop(&expr)?;
        let value = context.stack_pop(&expr)?;

        match name.kind {
          ExprKind::Symbol(symbol) => {
            context.set_scope_item(symbol, value).map(|_| context)
          }
          _ => Err(RunError {
            reason: RunErrorReason::InvalidDefinition,
            context: context.clone(),
            expr: expr.clone(),
          }),
        }
      }

      // MARK: Get
      Self::Get => {
        let name = context.stack_pop(&expr)?;

        match name.kind {
          ExprKind::Symbol(symbol) => context
            .stack_push(context.scope_item(symbol).ok_or_else(|| RunError {
              context: context.clone(),
              expr,
              reason: RunErrorReason::UnknownCall,
            })?)
            .map(|_| context),
          _ => Err(RunError {
            reason: RunErrorReason::UnknownCall,
            context: context.clone(),
            expr: expr.clone(),
          }),
        }
      }

      // MARK: Print
      Self::Print => {
        let val = context.stack_pop(&expr)?;

        println!("{}", val);

        Ok(context)
      }
      // MARK: Pretty
      Self::Pretty => {
        let val = context.stack_pop(&expr)?;

        println!("{:#}", val);

        Ok(context)
      }
      // MARK: Recur
      // Functionality is implemented in [`Engine::call_fn`]
      Self::Recur => {
        context.stack_push(Expr {
          kind: ExprKind::Symbol(Symbol::from_ref("recur")),
          info: None,
        })?;

        Ok(context)
      }

      // MARK: OrElse
      Self::OrElse => {
        let rhs = context.stack_pop(&expr)?;
        let lhs = context.stack_pop(&expr)?;

        match lhs.kind {
          ExprKind::Nil => context.stack_push(rhs)?,
          _ => context.stack_push(lhs)?,
        }

        Ok(context)
      }

      // MARK: Import
      Self::Import => {
        let path = context.stack_pop(&expr)?;

        match path.kind {
          ExprKind::String(str) => {
            if let Ok(source) = Source::from_path(str.as_str()) {
              context.add_source(source.clone());
              let mut lexer = Lexer::new(source);
              if let Ok(exprs) = parse(&mut lexer) {
                return engine.run(context, exprs);
              }
            }
          }
          _ => {
            todo!()
          }
        }

        Ok(context)
      }
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParseIntrinsicError;

impl std::error::Error for ParseIntrinsicError {}

impl fmt::Display for ParseIntrinsicError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "unknown intrinsic")
  }
}
