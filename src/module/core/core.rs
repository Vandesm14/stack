use std::time::SystemTime;

use crate::{EvalError, Program, SourceFile};

// TODO: Split `core` into `list` and `module` modules
pub fn module(program: &mut Program) -> Result<(), EvalError> {
  let source = include_str!("./core.stack");
  program.eval_string_with_name(source, "core")?;
  program.sources.insert(
    "core".into(),
    SourceFile {
      contents: source.into(),
      mtime: SystemTime::now(),
    },
  );

  Ok(())
}
