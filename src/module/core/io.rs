use crate::{
  interner::interner, DebugData, EvalError, EvalErrorKind, ExprKind, Program,
  SourceFile, Type,
};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program.funcs.insert(
    interner().get_or_intern_static("read-file"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item.val {
        ExprKind::String(ref path) => {
          let file_is_newer =
            if let Some(loaded_file) = program.sources.get(path) {
              let metadata = std::fs::metadata(path).ok().unwrap();
              let mtime = metadata.modified().ok().unwrap();
              mtime > loaded_file.mtime
            } else {
              true
            };

          if file_is_newer {
            match std::fs::read_to_string(path.clone()) {
              Ok(contents) => {
                program.sources.insert(
                  path.to_string(),
                  SourceFile {
                    contents: contents.clone(),
                    mtime: std::fs::metadata(path).unwrap().modified().unwrap(),
                  },
                );
                program.push(
                  ExprKind::String(contents).into_expr(DebugData::default()),
                )
              }
              Err(e) => Err(EvalError {
                expr: Some(trace_expr.clone()),
                kind: EvalErrorKind::UnableToRead(path.into(), e.to_string()),
              }),
            }
          } else {
            let contents = &program.sources.get(path).unwrap().contents;
            program.push(
              ExprKind::String(contents.clone())
                .into_expr(DebugData::default()),
            )
          }
        }
        _ => Err(EvalError {
          expr: Some(trace_expr.clone()),
          kind: EvalErrorKind::ExpectedFound(Type::String, item.val.type_of()),
        }),
      }
    },
  );

  Ok(())
}
