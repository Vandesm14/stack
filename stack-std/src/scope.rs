use stack::prelude::*;

pub fn module() -> Module {
  let mut module = Module::new(Symbol::from_ref("scope"));

  module.add_func(Symbol::from_ref("where"), |engine, mut context, expr| {
    let symbol = context.stack_pop(&expr)?;

    match symbol.kind {
      ExprKind::Intrinsic(_) => context.stack_push(Expr {
        kind: ExprKind::String("intrinsic".into()),
        info: engine.track_info().then(|| ExprInfo::Runtime {
          components: vec![symbol, expr],
        }),
      }),
      ExprKind::Symbol(ref x) => {
        if engine.module(x).is_some() {
          context.stack_push(Expr {
            kind: ExprKind::String("module".into()),
            info: engine.track_info().then(|| ExprInfo::Runtime {
              components: vec![symbol, expr],
            }),
          })
        } else if context.let_get(*x).is_some() {
          context.stack_push(Expr {
            kind: ExprKind::String("let".into()),
            info: engine.track_info().then(|| ExprInfo::Runtime {
              components: vec![symbol, expr],
            }),
          })
        } else if context.scope_item(*x).is_some() {
          context.stack_push(Expr {
            kind: ExprKind::String("scope".into()),
            info: engine.track_info().then(|| ExprInfo::Runtime {
              components: vec![symbol, expr],
            }),
          })
        } else {
          context.stack_push(Expr {
            kind: ExprKind::Nil,
            info: engine.track_info().then(|| ExprInfo::Runtime {
              components: vec![symbol, expr],
            }),
          })
        }
      }
      _ => context.stack_push(Expr {
        kind: ExprKind::Nil,
        info: engine.track_info().then(|| ExprInfo::Runtime {
          components: vec![symbol, expr],
        }),
      }),
    }
    .map(|_| context)
  });

  module
}
