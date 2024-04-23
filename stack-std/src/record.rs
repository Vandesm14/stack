use std::collections::HashMap;

use stack_core::prelude::*;

pub fn module() -> Module {
  let mut module = Module::new(Symbol::from_ref("record"));

  module
    .add_func(Symbol::from_ref("new"), |_, mut context, _| {
      context
        .stack_push(Expr {
          info: None,
          kind: ExprKind::Record(HashMap::new()),
        })
        .map(|_| context)
    })
    .add_func(Symbol::from_ref("insert"), |_, mut context, expr| {
      let record = context.stack_pop(&expr)?;
      let value = context.stack_pop(&expr)?;
      let symbol = context.stack_pop(&expr)?;

      match record.kind {
        ExprKind::Record(ref record) => {
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
    })
    .add_func(Symbol::from_ref("get"), |_, mut context, expr| {
      let record = context.stack_pop(&expr)?;
      let symbol = context.stack_pop(&expr)?;

      match record.kind {
        ExprKind::Record(ref record) => {
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

          let result = record.get(&symbol).unwrap_or_else(|| &Expr {
            info: None,
            kind: ExprKind::Nil,
          });

          context.stack_push(result.clone())?;

          Ok(())
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
