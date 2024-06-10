use stack_core::prelude::*;

pub fn module() -> Module {
  let mut module = Module::new(Symbol::from_ref("dbg"));

  module.add_func(Symbol::from_ref("note"), |engine, mut context, expr| {
    let val = context.stack_pop(&expr)?;

    println!(
      "op({}) {}",
      context
        .journal()
        .as_ref()
        .map(|j| j.total_commits())
        .unwrap_or_default()
        .saturating_sub(2),
      val
    );

    Ok(context)
  });

  module
}
