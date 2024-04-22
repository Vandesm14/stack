use core::str::FromStr;

use stack::prelude::*;

pub fn module() -> Module {
  let mut module = Module::new(Symbol::from_ref("scope"));

  module.add_func(Symbol::from_ref("where"), |engine, mut context, expr| {
    let symbol = context.stack_pop(&expr)?;

    match symbol.kind {
      ExprKind::Symbol(ref x) => {
        if Intrinsic::from_str(x.as_str()).is_ok() {
          context.stack_push(Expr {
            kind: ExprKind::String("intrinsic".into()),
            info: None,
          })
        } else if engine
          .module(&Symbol::new(
            x.as_str().split(':').next().unwrap_or_default().into(),
          ))
          .is_some()
        {
          context.stack_push(Expr {
            kind: ExprKind::String("module".into()),
            info: None,
          })
        } else if context.let_get(*x).is_some() {
          context.stack_push(Expr {
            kind: ExprKind::String("let".into()),
            info: None,
          })
        } else if context.scope_item(*x).is_some() {
          context.stack_push(Expr {
            kind: ExprKind::String("scope".into()),
            info: None,
          })
        } else {
          context.stack_push(Expr {
            kind: ExprKind::Nil,
            info: None,
          })
        }
      }
      _ => context.stack_push(Expr {
        kind: ExprKind::Nil,
        info: None,
      }),
    }
    .map(|_| context)
  });

  module
}
