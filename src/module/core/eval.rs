use crate::{
  interner::interner, EvalError, Expr, Lexer, Parser, Program, Type,
};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program.funcs.insert(
    interner().get_or_intern_static("parse"),
    |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item {
        Expr::String(string) => {
          let source = interner().resolve(&string).to_string();

          let lexer = Lexer::new(&source);
          let parser = Parser::new(lexer);
          let expr = parser.parse().ok().map(Expr::List).unwrap_or(Expr::Nil);

          program.push(expr)
        }
        _ => Err(EvalError {
          expr: trace_expr.clone(),
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::String,
            item.type_of(),
          ),
        }),
      }
    },
  );

  Ok(())
}
