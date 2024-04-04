use crate::{
  interner::interner, Ast, EvalError, Expr, Lexer, Parser, Program, Type,
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
          let old_ast_size = program.ast.len();
          let parser = Parser::new(lexer, &mut program.ast);

          let result = parser.parse();

          match result {
            Ok(_) => {
              let new_exprs = old_ast_size..program.ast.len();

              if let Some(new_exprs) = program.ast.expr_range(new_exprs) {
                program.eval(new_exprs.to_vec())
              } else {
                Err(EvalError {
                  program: program.clone(),
                  message: "Failed to find parsed exprs".into(),
                  expr: Ast::NIL,
                })
              }
            }
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
