use stack::prelude::*;
use unicode_segmentation::UnicodeSegmentation;

pub fn module() -> Module {
  Module::new(Symbol::from_ref("str"))
    .with_func(
      Symbol::from_ref("trim-start"),
      |engine, mut context, expr| {
        let item = context.stack_pop(&expr)?;

        let kind = match item.kind {
          ExprKind::String(ref x) => ExprKind::String(x.trim_start().into()),
          _ => ExprKind::Nil,
        };

        context.stack_push(Expr {
          kind,
          info: engine.track_info().then(|| ExprInfo::Runtime {
            components: vec![item, expr],
          }),
        });

        Ok(context)
      },
    )
    .with_func(Symbol::from_ref("trim-end"), |engine, mut context, expr| {
      let item = context.stack_pop(&expr)?;

      let kind = match item.kind {
        ExprKind::String(ref x) => ExprKind::String(x.trim_end().into()),
        _ => ExprKind::Nil,
      };

      context.stack_push(Expr {
        kind,
        info: engine.track_info().then(|| ExprInfo::Runtime {
          components: vec![item, expr],
        }),
      });

      Ok(context)
    })
    .with_func(Symbol::from_ref("trim"), |engine, mut context, expr| {
      let item = context.stack_pop(&expr)?;

      let kind = match item.kind {
        ExprKind::String(ref x) => ExprKind::String(x.trim().into()),
        _ => ExprKind::Nil,
      };

      context.stack_push(Expr {
        kind,
        info: engine.track_info().then(|| ExprInfo::Runtime {
          components: vec![item, expr],
        }),
      });

      Ok(context)
    })
    .with_func(
      Symbol::from_ref("starts-with"),
      |engine, mut context, expr| {
        let patt = context.stack_pop(&expr)?;
        let item = context.stack_pop(&expr)?;

        let kind = match (item.kind.clone(), patt.kind.clone()) {
          (ExprKind::String(ref x), ExprKind::String(ref y)) => {
            ExprKind::Boolean(x.starts_with(y))
          }
          _ => ExprKind::Nil,
        };

        context.stack_push(Expr {
          kind,
          info: engine.track_info().then(|| ExprInfo::Runtime {
            components: vec![item, patt, expr],
          }),
        });

        Ok(context)
      },
    )
    .with_func(
      Symbol::from_ref("ends-with"),
      |engine, mut context, expr| {
        let patt = context.stack_pop(&expr)?;
        let item = context.stack_pop(&expr)?;

        let kind = match (item.kind.clone(), patt.kind.clone()) {
          (ExprKind::String(ref x), ExprKind::String(ref y)) => {
            ExprKind::Boolean(x.ends_with(y))
          }
          _ => ExprKind::Nil,
        };

        context.stack_push(Expr {
          kind,
          info: engine.track_info().then(|| ExprInfo::Runtime {
            components: vec![item, patt, expr],
          }),
        });

        Ok(context)
      },
    )
    .with_func(Symbol::from_ref("split-by"), |engine, mut context, expr| {
      let patt = context.stack_pop(&expr)?;
      let item = context.stack_pop(&expr)?;

      let kind = match (item.kind.clone(), patt.kind.clone()) {
        (ExprKind::String(ref x), ExprKind::String(ref y)) => ExprKind::List(
          x.split(y)
            .map(|x| Expr {
              kind: ExprKind::String(x.into()),
              info: engine.track_info().then(|| ExprInfo::Runtime {
                components: vec![item.clone(), patt.clone(), expr.clone()],
              }),
            })
            .collect(),
        ),
        _ => ExprKind::Nil,
      };

      context.stack_push(Expr {
        kind,
        info: engine.track_info().then(|| ExprInfo::Runtime {
          components: vec![item, patt, expr],
        }),
      });

      Ok(context)
    })
    .with_func(
      Symbol::from_ref("split-whitespace"),
      |engine, mut context, expr| {
        let item = context.stack_pop(&expr)?;

        let kind = match item.kind {
          ExprKind::String(ref x) => ExprKind::List(
            x.split_whitespace()
              .map(|x| Expr {
                kind: ExprKind::String(x.into()),
                info: engine.track_info().then(|| ExprInfo::Runtime {
                  components: vec![item.clone(), expr.clone()],
                }),
              })
              .collect(),
          ),
          _ => ExprKind::Nil,
        };

        context.stack_push(Expr {
          kind,
          info: engine.track_info().then(|| ExprInfo::Runtime {
            components: vec![item, expr],
          }),
        });

        Ok(context)
      },
    )
    .with_func(
      Symbol::from_ref("to-lowercase"),
      |engine, mut context, expr| {
        let item = context.stack_pop(&expr)?;

        let kind = match item.kind {
          ExprKind::String(ref x) => ExprKind::String(x.to_lowercase()),
          _ => ExprKind::Nil,
        };

        context.stack_push(Expr {
          kind,
          info: engine.track_info().then(|| ExprInfo::Runtime {
            components: vec![item, expr],
          }),
        });

        Ok(context)
      },
    )
    .with_func(
      Symbol::from_ref("to-uppercase"),
      |engine, mut context, expr| {
        let item = context.stack_pop(&expr)?;

        let kind = match item.kind {
          ExprKind::String(ref x) => ExprKind::String(x.to_uppercase()),
          _ => ExprKind::Nil,
        };

        context.stack_push(Expr {
          kind,
          info: engine.track_info().then(|| ExprInfo::Runtime {
            components: vec![item, expr],
          }),
        });

        Ok(context)
      },
    )
    .with_func(Symbol::from_ref("is-ascii"), |engine, mut context, expr| {
      let item = context.stack_pop(&expr)?;

      let kind = match item.kind {
        ExprKind::String(ref x) => ExprKind::Boolean(x.is_ascii()),
        _ => ExprKind::Nil,
      };

      context.stack_push(Expr {
        kind,
        info: engine.track_info().then(|| ExprInfo::Runtime {
          components: vec![item, expr],
        }),
      });

      Ok(context)
    })
    .with_func(Symbol::from_ref("is-char"), |engine, mut context, expr| {
      let item = context.stack_pop(&expr)?;

      let kind = match item.kind {
        ExprKind::String(ref x) => {
          ExprKind::Boolean(x.graphemes(true).count() == 1)
        }
        _ => ExprKind::Nil,
      };

      context.stack_push(Expr {
        kind,
        info: engine.track_info().then(|| ExprInfo::Runtime {
          components: vec![item, expr],
        }),
      });

      Ok(context)
    })
    .with_func(Symbol::from_ref("to-bytes"), |engine, mut context, expr| {
      let item = context.stack_pop(&expr)?;

      let kind = match item.kind {
        ExprKind::String(ref x) => ExprKind::List(
          x.as_bytes()
            .iter()
            .copied()
            .map(|x| Expr {
              kind: ExprKind::Integer(x as i64),
              info: engine.track_info().then(|| ExprInfo::Runtime {
                components: vec![item.clone(), expr.clone()],
              }),
            })
            .collect(),
        ),
        _ => ExprKind::Nil,
      };

      context.stack_push(Expr {
        kind,
        info: engine.track_info().then(|| ExprInfo::Runtime {
          components: vec![item, expr],
        }),
      });

      Ok(context)
    })
    .with_func(
      Symbol::from_ref("from-bytes"),
      |engine, mut context, expr| {
        let item = context.stack_pop(&expr)?;

        let kind = match item.kind {
          ExprKind::List(ref x) => x
            .iter()
            .try_fold(Vec::with_capacity(x.len()), |mut v, x| match x.kind {
              ExprKind::Integer(i) if i >= 0 && i <= u8::MAX as i64 => {
                v.push(i as u8);
                Ok(v)
              }
              _ => Err(()),
            })
            .map(|x| {
              String::from_utf8(x)
                .map(ExprKind::String)
                .unwrap_or(ExprKind::Nil)
            })
            .unwrap_or(ExprKind::Nil),
          _ => ExprKind::Nil,
        };

        context.stack_push(Expr {
          kind,
          info: engine.track_info().then(|| ExprInfo::Runtime {
            components: vec![item, expr],
          }),
        });

        Ok(context)
      },
    )
    .with_func(Symbol::from_ref("to-chars"), |engine, mut context, expr| {
      let item = context.stack_pop(&expr)?;

      let kind = match item.kind {
        ExprKind::String(ref x) => ExprKind::List(
          x.as_str()
            .graphemes(true)
            .map(|x| Expr {
              kind: ExprKind::String(x.into()),
              info: engine.track_info().then(|| ExprInfo::Runtime {
                components: vec![item.clone(), expr.clone()],
              }),
            })
            .collect(),
        ),
        _ => ExprKind::Nil,
      };

      context.stack_push(Expr {
        kind,
        info: engine.track_info().then(|| ExprInfo::Runtime {
          components: vec![item, expr],
        }),
      });

      Ok(context)
    })
    .with_func(
      Symbol::from_ref("from-chars"),
      |engine, mut context, expr| {
        let item = context.stack_pop(&expr)?;

        let kind = match item.kind {
          ExprKind::List(ref x) => x
            .iter()
            .try_fold(String::with_capacity(x.len()), |mut v, x| match x.kind {
              ExprKind::String(ref s) => {
                v.push_str(s);
                Ok(v)
              }
              _ => Err(()),
            })
            .map(ExprKind::String)
            .unwrap_or(ExprKind::Nil),
          _ => ExprKind::Nil,
        };

        context.stack_push(Expr {
          kind,
          info: engine.track_info().then(|| ExprInfo::Runtime {
            components: vec![item, expr],
          }),
        });

        Ok(context)
      },
    )
}
