use compact_str::{CompactString, ToCompactString};
use regex::Regex;
use stack_core::prelude::*;
use unicode_segmentation::UnicodeSegmentation;

// TODO: Add str:escape and str:unescape.

pub fn module() -> Module {
  Module::new(Symbol::from_ref("str"))
    .with_func(Symbol::from_ref("trim-start"), |_, mut context, expr| {
      let item = context.stack_pop(&expr)?;

      let kind = match item.kind {
        ExprKind::String(ref x) => ExprKind::String(x.trim_start().into()),
        _ => ExprKind::Nil,
      };

      context.stack_push(kind.into())?;

      Ok(context)
    })
    .with_func(Symbol::from_ref("trim-end"), |_, mut context, expr| {
      let item = context.stack_pop(&expr)?;

      let kind = match item.kind {
        ExprKind::String(ref x) => ExprKind::String(x.trim_end().into()),
        _ => ExprKind::Nil,
      };

      context.stack_push(kind.into())?;

      Ok(context)
    })
    .with_func(Symbol::from_ref("trim"), |_, mut context, expr| {
      let item = context.stack_pop(&expr)?;

      let kind = match item.kind {
        ExprKind::String(ref x) => ExprKind::String(x.trim().into()),
        _ => ExprKind::Nil,
      };

      context.stack_push(kind.into())?;

      Ok(context)
    })
    .with_func(Symbol::from_ref("starts-with"), |_, mut context, expr| {
      let patt = context.stack_pop(&expr)?;
      let item = context.stack_pop(&expr)?;

      let kind = match (item.kind.clone(), patt.kind.clone()) {
        (ExprKind::String(ref x), ExprKind::String(ref y)) => {
          ExprKind::Boolean(x.starts_with(y.as_str()))
        }
        _ => ExprKind::Nil,
      };

      context.stack_push(kind.into())?;

      Ok(context)
    })
    .with_func(Symbol::from_ref("ends-with"), |_, mut context, expr| {
      let patt = context.stack_pop(&expr)?;
      let item = context.stack_pop(&expr)?;

      let kind = match (item.kind.clone(), patt.kind.clone()) {
        (ExprKind::String(ref x), ExprKind::String(ref y)) => {
          ExprKind::Boolean(x.ends_with(y.as_str()))
        }
        _ => ExprKind::Nil,
      };

      context.stack_push(kind.into())?;

      Ok(context)
    })
    .with_func(Symbol::from_ref("split-by"), |_, mut context, expr| {
      let patt = context.stack_pop(&expr)?;
      let item = context.stack_pop(&expr)?;

      let kind = match (item.kind.clone(), patt.kind.clone()) {
        (ExprKind::String(ref x), ExprKind::String(ref y)) => ExprKind::List(
          x.split(y.as_str())
            .map(|x| ExprKind::String(x.into()).into())
            .collect(),
        ),
        _ => ExprKind::Nil,
      };

      context.stack_push(kind.into())?;

      Ok(context)
    })
    .with_func(
      Symbol::from_ref("split-whitespace"),
      |_, mut context, expr| {
        let item = context.stack_pop(&expr)?;

        let kind = match item.kind {
          ExprKind::String(ref x) => ExprKind::List(
            x.split_whitespace()
              .map(|x| ExprKind::String(x.into()).into())
              .collect(),
          ),
          _ => ExprKind::Nil,
        };

        context.stack_push(kind.into())?;

        Ok(context)
      },
    )
    .with_func(Symbol::from_ref("to-lowercase"), |_, mut context, expr| {
      let item = context.stack_pop(&expr)?;

      let kind = match item.kind {
        ExprKind::String(ref x) => ExprKind::String(x.to_lowercase()),
        _ => ExprKind::Nil,
      };

      context.stack_push(kind.into())?;

      Ok(context)
    })
    .with_func(Symbol::from_ref("to-uppercase"), |_, mut context, expr| {
      let item = context.stack_pop(&expr)?;

      let kind = match item.kind {
        ExprKind::String(ref x) => ExprKind::String(x.to_uppercase()),
        _ => ExprKind::Nil,
      };

      context.stack_push(kind.into())?;

      Ok(context)
    })
    .with_func(Symbol::from_ref("is-ascii"), |_, mut context, expr| {
      let item = context.stack_pop(&expr)?;

      let kind = match item.kind {
        ExprKind::String(ref x) => ExprKind::Boolean(x.is_ascii()),
        _ => ExprKind::Nil,
      };

      context.stack_push(kind.into())?;

      Ok(context)
    })
    .with_func(Symbol::from_ref("is-char"), |_, mut context, expr| {
      let item = context.stack_pop(&expr)?;

      let kind = match item.kind {
        ExprKind::String(ref x) => {
          ExprKind::Boolean(x.graphemes(true).count() == 1)
        }
        _ => ExprKind::Nil,
      };

      context.stack_push(kind.into())?;

      Ok(context)
    })
    .with_func(Symbol::from_ref("to-bytes"), |_, mut context, expr| {
      let item = context.stack_pop(&expr)?;

      let kind = match item.kind {
        ExprKind::String(ref x) => ExprKind::List(
          x.as_bytes()
            .iter()
            .copied()
            .map(|x| ExprKind::Integer(x as i64).into())
            .collect(),
        ),
        _ => ExprKind::Nil,
      };

      context.stack_push(kind.into())?;

      Ok(context)
    })
    .with_func(Symbol::from_ref("from-bytes"), |_, mut context, expr| {
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
            CompactString::from_utf8(x)
              .map(ExprKind::String)
              .unwrap_or(ExprKind::Nil)
          })
          .unwrap_or(ExprKind::Nil),
        _ => ExprKind::Nil,
      };

      context.stack_push(kind.into())?;

      Ok(context)
    })
    .with_func(Symbol::from_ref("to-chars"), |_, mut context, expr| {
      let item = context.stack_pop(&expr)?;

      let kind = match item.kind {
        ExprKind::String(ref x) => ExprKind::List(
          x.as_str()
            .graphemes(true)
            .map(|x| ExprKind::String(x.into()).into())
            .collect(),
        ),
        _ => ExprKind::Nil,
      };

      context.stack_push(kind.into())?;

      Ok(context)
    })
    .with_func(Symbol::from_ref("from-chars"), |_, mut context, expr| {
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
          .map(|x| x.to_compact_string())
          .map(ExprKind::String)
          .unwrap_or(ExprKind::Nil),
        _ => ExprKind::Nil,
      };

      context.stack_push(kind.into())?;

      Ok(context)
    })
    .with_func(Symbol::from_ref("regex-test"), |_, mut context, expr| {
      let pattern = context.stack_pop(&expr)?;
      let string = context.stack_pop(&expr)?;

      let kind = match (string.kind, pattern.kind) {
        (ExprKind::String(ref string), ExprKind::String(ref pattern)) => {
          let re = Regex::new(pattern);
          match re {
            Ok(re) => ExprKind::Boolean(re.captures(string).is_some()),
            Err(err) => {
              todo!()
            }
          }
        }
        (_, _) => ExprKind::Nil,
      };

      context.stack_push(kind.into())?;

      Ok(context)
    })
    .with_func(Symbol::from_ref("regex-match"), |_, mut context, expr| {
      let pattern = context.stack_pop(&expr)?;
      let string = context.stack_pop(&expr)?;

      let kind = match (string.kind, pattern.kind) {
        (ExprKind::String(ref string), ExprKind::String(ref pattern)) => {
          let re = Regex::new(pattern);
          match re {
            Ok(re) => re
              .find(string)
              .map(|m| {
                ExprKind::String(
                  string
                    .chars()
                    .skip(m.start())
                    .take(m.end() - m.start())
                    .collect::<String>()
                    .into(),
                )
              })
              .unwrap_or(ExprKind::Nil),
            Err(err) => {
              todo!()
            }
          }
        }
        (_, _) => ExprKind::Nil,
      };

      context.stack_push(kind.into())?;

      Ok(context)
    })
    .with_func(
      Symbol::from_ref("regex-match-all"),
      |_, mut context, expr| {
        let pattern = context.stack_pop(&expr)?;
        let string = context.stack_pop(&expr)?;

        let kind = match (string.kind, pattern.kind) {
          (ExprKind::String(ref string), ExprKind::String(ref pattern)) => {
            let re = Regex::new(pattern);
            match re {
              Ok(re) => ExprKind::List(
                re.find_iter(string)
                  .map(|m| {
                    ExprKind::String(
                      string
                        .chars()
                        .skip(m.start())
                        .take(m.end() - m.start())
                        .collect::<String>()
                        .into(),
                    )
                    .into()
                  })
                  .collect::<Vec<_>>(),
              ),
              Err(err) => {
                todo!()
              }
            }
          }
          (_, _) => ExprKind::Nil,
        };

        context.stack_push(kind.into())?;

        Ok(context)
      },
    )
}
