use std::sync::Arc;

use compact_str::ToCompactString;
use stack_core::prelude::*;

pub fn module(sandbox: bool) -> Module {
  let mut module = Module::new(Symbol::from_ref("fs"));

  if !sandbox {
    module
      .add_func(
        Symbol::from_ref("cwd"),
        Arc::new(|_, mut context, _| {
          context.stack_push(
            std::env::current_dir()
              .map(|x| {
                ExprKind::String(
                  x.to_string_lossy().into_owned().to_compact_string(),
                )
              })
              .unwrap_or(ExprKind::Nil)
              .into(),
          )?;

          Ok(context)
        }),
      )
      .add_func(
        Symbol::from_ref("read-file"),
        Arc::new(|_, mut context, expr| {
          let path = context.stack_pop(&expr)?;

          let kind = match path.kind {
            ExprKind::String(ref x) => std::fs::read_to_string(x.as_str())
              .map(|x| x.to_compact_string())
              .map(ExprKind::String)
              .unwrap_or(ExprKind::Nil),
            _ => ExprKind::Nil,
          };

          context.stack_push(kind.into())?;

          Ok(context)
        }),
      );
  }

  module
}
