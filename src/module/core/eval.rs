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
            Ok(new_exprs) => {
              program.push_expr(Expr::List(new_exprs))?;
              Ok(())
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

#[cfg(test)]
mod tests {
  use crate::{interner::interner, ExprTree, FnSymbol, Program, Scope};

  #[test]
  fn parse_valid_string() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("\"1 2 +\" parse").unwrap();
    assert_eq!(
      program.stack_exprs(),
      vec![ExprTree::List(vec![
        ExprTree::Integer(1),
        ExprTree::Integer(2),
        ExprTree::Call(interner().get_or_intern_static("+")),
      ])]
    );
  }

  #[test]
  fn parse_lazy_list() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("\"'(fn 'a)\" parse").unwrap();
    assert_eq!(
      program.stack_exprs(),
      vec![ExprTree::List(vec![ExprTree::Lazy(Box::new(
        ExprTree::List(vec![
          ExprTree::Fn(FnSymbol {
            scoped: true,
            scope: Scope::new(),
          }),
          ExprTree::Lazy(Box::new(ExprTree::Call(
            interner().get_or_intern_static("a")
          ))),
        ])
      ),)])]
    );
  }
}
