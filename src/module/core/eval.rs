use internment::Intern;

use crate::{
  DebugData, EvalError, EvalErrorKind, ExprKind, Lexer, Parser, Program, Type,
};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program
    .funcs
    .insert(Intern::from_ref("parse"), |program, trace_expr| {
      let item = program.pop(trace_expr)?;

      match item.val {
        ExprKind::String(string) => {
          let lexer = Lexer::new(&string);
          let parser = Parser::new(lexer, Intern::from_ref("internal"));
          let expr = parser
            .parse()
            .ok()
            .map(ExprKind::List)
            .unwrap_or(ExprKind::Nil);

          program.push(expr.into_expr(DebugData::default()))
        }
        _ => Err(EvalError {
          expr: Some(trace_expr.clone()),
          kind: EvalErrorKind::ExpectedFound(Type::String, item.val.type_of()),
        }),
      }
    });

  Ok(())
}
