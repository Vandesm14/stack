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
    .add_func(Symbol::from_ref("into-list"), |_, mut context, expr| {
      let record = context.stack_pop(&expr)?;

      match record.kind {
        ExprKind::Record(x) => {
          let mut list: Vec<Expr> = Vec::new();
          x.into_iter().for_each(|(key, value)| {
            list.push(Expr {
              info: None,
              kind: ExprKind::Symbol(key),
            });
            list.push(value)
          });

          context.stack_push(Expr {
            info: None,
            kind: ExprKind::List(list),
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
    .add_func(Symbol::from_ref("into-pairs"), |_, mut context, expr| {
      let record = context.stack_pop(&expr)?;

      match record.kind {
        ExprKind::Record(x) => {
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

          context.stack_push(Expr {
            info: None,
            kind: ExprKind::List(list),
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
    .add_func(Symbol::from_ref("from-list"), |_, mut context, expr| {
      let list = context.stack_pop(&expr)?;

      match list.kind {
        ExprKind::List(x) => {
          let mut record: HashMap<Symbol, Expr> = HashMap::new();
          x.chunks(2)
            .filter(|chunk| chunk.len() == 2)
            .for_each(|chunk| {
              let key = Symbol::from_ref(chunk[0].kind.to_string().as_str());
              let value = &chunk[1];
              record.insert(key, value.clone());
            });

          context.stack_push(Expr {
            info: None,
            kind: ExprKind::Record(record),
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
    .add_func(Symbol::from_ref("from-pairs"), |_, mut context, expr| {
      let list = context.stack_pop(&expr)?;

      match list.kind {
        ExprKind::List(x) => {
          let mut record: HashMap<Symbol, Expr> = HashMap::new();
          x.into_iter().for_each(|item| {
            if let ExprKind::List(chunk) = item.kind {
              let key = Symbol::from_ref(chunk[0].kind.to_string().as_str());
              let value = &chunk[1];
              record.insert(key, value.clone());
            }
          });

          context.stack_push(Expr {
            info: None,
            kind: ExprKind::Record(record),
          })?;

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
