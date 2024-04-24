use stack_core::prelude::*;

pub fn module() -> Module {
  Module::new(Symbol::from_ref("tag"))
    .with_func(Symbol::from_ref("add"), |_, mut context, expr| {
      let item = context.stack_pop(&expr)?;
      let value = context.stack_pop(&expr)?;
      let key = context.stack_pop(&expr)?;

      let mut clone = item.clone();
      clone.tags.insert(key.kind.into(), value);

      context.stack_push(clone)?;

      Ok(context)
    })
    .with_func(Symbol::from_ref("remove"), |_, mut context, expr| {
      let item = context.stack_pop(&expr)?;
      let key = context.stack_pop(&expr)?;

      let mut clone = item.clone();
      clone.tags.remove(&key.kind.into());

      context.stack_push(clone)?;

      Ok(context)
    })
    .with_func(Symbol::from_ref("get"), |_, mut context, expr| {
      let item = context.stack_pop(&expr)?;
      let key = context.stack_pop(&expr)?;

      let result = match item.tags.get(&key.kind.into()) {
        Some(expr) => expr.clone(),
        None => ExprKind::Nil.into(),
      };

      context.stack_push(item)?;
      context.stack_push(result)?;

      Ok(context)
    })
}
