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
        ExprKind::String(path) => {
          let path_str = interner().resolve(&path);
          let file_is_newer =
            if let Some(loaded_file) = program.sources.get(path_str) {
              let metadata = std::fs::metadata(path_str).ok().unwrap();
              let mtime = metadata.modified().ok().unwrap();
              mtime > loaded_file.mtime
            } else {
              true
            };

          if file_is_newer {
            match std::fs::read_to_string(path_str) {
              Ok(contents) => {
                let content = interner().get_or_intern(contents);
                program.sources.insert(
                  path_str.to_string(),
                  SourceFile {
                    contents: content,
                    mtime: std::fs::metadata(path_str)
                      .unwrap()
                      .modified()
                      .unwrap(),
                  },
                );
                program.push(
                  ExprKind::String(content).into_expr(DebugData::default()),
                )
              }
              Err(e) => Err(EvalError {
                expr: Some(trace_expr.clone()),
                kind: EvalErrorKind::UnableToRead(
                  path_str.into(),
                  e.to_string(),
                ),
              }),
            }
          } else {
            let contents = program.sources.get(path_str).unwrap().contents;
            program
              .push(ExprKind::String(contents).into_expr(DebugData::default()))
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
