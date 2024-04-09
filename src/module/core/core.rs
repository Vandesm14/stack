use std::time::SystemTime;

use crate::{EvalError, Program, SourceFile};

// TODO: Split `core` into `list` and `module` modules
pub fn module(program: &mut Program) -> Result<(), EvalError> {
  let source = include_str!("./core.stack");
  let source_name = "<internal/core.stack>".to_owned();
  program.eval_string_with_name(source, &source_name)?;
  program.sources.insert(
    source_name,
    SourceFile {
      contents: source.into(),
      mtime: SystemTime::now(),
    },
  );

  Ok(())
}
