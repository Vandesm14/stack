use std::sync::{mpsc, Arc};

use stack_core::prelude::*;

pub fn module(tx: mpsc::Sender<String>) -> Module {
  let mut module = Module::new(Symbol::from_ref("dbg"));

  let tx2 = tx.clone();

  module
    .add_func(
      Symbol::from_ref("note"),
      Arc::new(move |_, mut context, expr| {
        let val = context.stack_pop(&expr)?;

        tx.send(format!(
          "dbg:note op({}) {}",
          context
            .journal()
            .as_ref()
            .map(|j| j.total_commits())
            .unwrap_or_default()
            .saturating_sub(2),
          val
        ))
        .unwrap();

        Ok(context)
      }),
    )
    .add_func(
      Symbol::from_ref("mark"),
      Arc::new(move |_, context, _| {
        tx2
          .send(format!(
            "dbg:mark op({})",
            context
              .journal()
              .as_ref()
              .map(|j| j.total_commits())
              .unwrap_or_default()
              .saturating_sub(1)
          ))
          .unwrap();

        Ok(context)
      }),
    );

  module
}
