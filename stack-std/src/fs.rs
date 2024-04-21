use stack::{expr::Error, prelude::*};

pub fn module(sandbox: bool) -> Module {
  let mut module = Module::new(Symbol::from_ref("fs"));

  if !sandbox {
    module
      .add_func(Symbol::from_ref("cwd"), |engine, mut context, expr| {
        context.stack_push(Expr {
          kind: std::env::current_dir()
            .map(|x| ExprKind::String(x.to_string_lossy().into_owned()))
            .unwrap_or(ExprKind::Nil),
          info: engine.track_info().then(|| ExprInfo::Runtime {
            components: vec![expr],
          }),
        })?;

        Ok(context)
      })
      .add_func(
        Symbol::from_ref("read-file"),
        |engine, mut context, expr| {
          let path = context.stack_pop(&expr)?;

          let kind = match path.kind {
            ExprKind::String(ref x) => std::fs::read_to_string(x)
              .map(ExprKind::String)
              .unwrap_or_else(|e| ExprKind::Error(Error::new(e.to_string()))),
            _ => ExprKind::Nil,
          };

          context.stack_push(Expr {
            kind,
            info: engine.track_info().then(|| ExprInfo::Runtime {
              components: vec![path, expr],
            }),
          })?;

          Ok(context)
        },
      );
  }

  module
}
