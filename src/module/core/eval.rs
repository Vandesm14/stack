use crate::{
  interner::interner, EvalError, Expr, ExprKind, Lexer, Parser, Program, Type,
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
          let parser = Parser::new(lexer);
          let expr = parser
            .parse()
            .ok()
            .map(ExprKind::List)
            .unwrap_or(ExprKind::Nil);

          program.push(expr.into_expr())
        }
        _ => Err(EvalError {
          expr: trace_expr.clone(),
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::String,
            item.val.type_of(),
          ),
        }),
      }
    },
  );

  Ok(())
}
