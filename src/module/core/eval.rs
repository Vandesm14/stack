use crate::{
  interner::interner, EvalError, Expr, Lexer, Parser, Program, Type,
};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  // TODO: This reimplements the same thing that program.eval does for lists. We should consolidate the code instead of having it in two places
  program.funcs.insert(
    interner().get_or_intern_static("parse"),
    |program, trace_expr| {
      let item = program.pop_expr(trace_expr)?;

      match item {
        Expr::String(string) => {
          let source = interner().resolve(&string).to_string();

          let lexer = Lexer::new(&source);
          let parser = Parser::new(lexer, &mut program.ast);

          let result = parser.parse();

          match result {
            Ok(new_exprs) => program.eval_indicies(new_exprs),
            Err(parse_error) => Err(EvalError {
              expr: trace_expr,
              program: program.clone(),
              message: format!("failed to parse {}, {}", item, parse_error,),
            }),
          }
        }
        _ => Err(EvalError {
          expr: trace_expr,
          program: program.clone(),
          message: format!(
            "expected {}, found {}",
            Type::String,
            item.type_of(&program.ast),
          ),
        }),
      }
    },
  );

  Ok(())
}
