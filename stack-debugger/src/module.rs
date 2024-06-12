use crate::IOHookEvent;
use stack_core::prelude::*;
use std::sync::{mpsc, Arc};

pub fn module(tx: mpsc::Sender<IOHookEvent>) -> Module {
  let mut module = Module::new(Symbol::from_ref("dbg"));

  let tx2 = tx.clone();
  let tx3 = tx.clone();

  module
    .add_func(
      Symbol::from_ref("note"),
      Arc::new(move |_, mut context, expr| {
        let val = context.stack_pop(&expr)?;

        tx.send(IOHookEvent::Note(
          context
            .journal()
            .as_ref()
            .map(|j| j.entries().len())
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
          .send(IOHookEvent::Marker(
            context
              .journal()
              .as_ref()
              .map(|j| j.entries().len())
              .unwrap_or_default(),
          ))
          .unwrap();

        Ok(context)
      }),
    )
    .add_func(
      Symbol::from_ref("goto"),
      Arc::new(move |_, context, _| {
        tx3
          .send(IOHookEvent::GoTo(
            context
              .journal()
              .as_ref()
              .map(|j| j.entries().len())
              .unwrap_or_default(),
          ))
          .unwrap();

        Ok(context)
      }),
    );

  module
}
