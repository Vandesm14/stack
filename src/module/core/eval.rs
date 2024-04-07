use crate::{
  interner::interner, DebugData, EvalError, EvalErrorKind, ExprKind, Lexer,
  Parser, Program, Type,
};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program.funcs.insert(
    interner().get_or_intern_static("parse"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item.val {
        ExprKind::String(string) => {
          let source = interner().resolve(&string).to_string();

          let lexer = Lexer::new(&source);
          let parser = Parser::new(lexer, interner().get_or_intern("internal"));
          let expr = parser
            .parse()
            .ok()
            .map(ExprKind::List)
            .unwrap_or(ExprKind::Nil);

          program.push(expr.into_expr(DebugData::only_ingredients(vec![
            item,
            trace_expr.clone(),
          ])))
        }
        _ => Err(EvalError {
          expr: Some(trace_expr),
          kind: EvalErrorKind::ExpectedFound(Type::String, item.val.type_of()),
        }),
      }
    },
  );

  Ok(())
}
