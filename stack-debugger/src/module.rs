use crate::PrintOut;
use stack_core::prelude::*;
use std::sync::{mpsc, Arc};

pub fn module(tx: mpsc::Sender<PrintOut>) -> Module {
  let mut module = Module::new(Symbol::from_ref("dbg"));

  let tx2 = tx.clone();

  module
    .add_func(
      Symbol::from_ref("note"),
      Arc::new(move |_, mut context, expr| {
        let val = context.stack_pop(&expr)?;

        tx.send(PrintOut::Note(
          context
            .journal()
            .as_ref()
            .map(|j| j.total_commits())
            .unwrap_or_default(),
          val.to_string(),
        ))
        .unwrap();

        Ok(context)
      }),
    )
    .add_func(
      Symbol::from_ref("mark"),
      Arc::new(move |_, context, _| {
        tx2
          .send(PrintOut::Marker(
            context
              .journal()
              .as_ref()
              .map(|j| j.total_commits())
              .unwrap_or_default(),
          ))
          .unwrap();

        Ok(context)
      }),
    );

  module
}
