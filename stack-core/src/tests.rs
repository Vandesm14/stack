#[cfg(test)]
mod lispy {
  use crate::prelude::*;

  #[test]
  fn lisp_syntax() {
    let source = Source::new("", "(+ 2 1)");
    let mut lexer = Lexer::new(source);
    let exprs = crate::parser::parse(&mut lexer).unwrap();

    let expected: Vec<Expr> = vec![ExprKind::SExpr {
      call: ExprKind::Symbol(Symbol::new("+".into())).into(),
      body: vec![ExprKind::Integer(2).into(), ExprKind::Integer(1).into()],
    }
    .into()];

    assert_eq!(
      exprs
        .into_iter()
        .map(|mut expr| {
          expr.recursively_strip_info();
          expr
        })
        .collect::<Vec<_>>(),
      expected
    )
  }

  #[test]
  fn lisp_pushes_correctly() {
    let engine = Engine::new();
    let mut context = Context::new().with_stack_capacity(32);
    context = engine
      .run(
        context,
        vec![ExprKind::SExpr {
          call: ExprKind::Symbol(Symbol::new("+".into())).into(),
          body: vec![ExprKind::Integer(2).into(), ExprKind::Integer(1).into()],
        }
        .into()],
      )
      .unwrap();

    assert_eq!(
      context
        .stack()
        .iter()
        .map(|expr| &expr.kind)
        .collect::<Vec<_>>(),
      vec![&ExprKind::Integer(1), &ExprKind::Integer(2),]
    );
  }

  #[test]
  fn integration_test() {
    // let source = Source::new("", "(+ 2 2) (def 'a _) a");
    // let mut lexer = Lexer::new(source);
    // let exprs = crate::parser::parse(&mut lexer).unwrap();
  }
}
