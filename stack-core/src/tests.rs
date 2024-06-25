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
  fn lisp_evaluates_correctly() {
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
      vec![&ExprKind::Integer(3)]
    );
  }

  #[test]
  fn underscores_pop() {
    let source = Source::new("", "2 (- 10 _)");
    let mut lexer = Lexer::new(source);
    let exprs = crate::parser::parse(&mut lexer).unwrap();

    let engine = Engine::new();
    let mut context = Context::new().with_stack_capacity(32);
    context = engine.run(context, exprs).unwrap();

    assert_eq!(
      context
        .stack()
        .iter()
        .map(|expr| &expr.kind)
        .collect::<Vec<_>>(),
      vec![&ExprKind::Integer(8)]
    )
  }

  #[test]
  fn underscores_order() {
    let source = Source::new("", "10 (- _ 2)");
    let mut lexer = Lexer::new(source);
    let exprs = crate::parser::parse(&mut lexer).unwrap();

    let engine = Engine::new();
    let mut context = Context::new().with_stack_capacity(32);
    context = engine.run(context, exprs).unwrap();

    assert_eq!(
      context
        .stack()
        .iter()
        .map(|expr| &expr.kind)
        .collect::<Vec<_>>(),
      vec![&ExprKind::Integer(8)]
    )
  }

  #[test]
  fn underscores_order_many() {
    let source = Source::new("", "2 10 (- _ _)");
    let mut lexer = Lexer::new(source);
    let exprs = crate::parser::parse(&mut lexer).unwrap();

    let engine = Engine::new();
    let mut context = Context::new().with_stack_capacity(32);
    context = engine.run(context, exprs).unwrap();

    assert_eq!(
      context
        .stack()
        .iter()
        .map(|expr| &expr.kind)
        .collect::<Vec<_>>(),
      vec![&ExprKind::Integer(8)]
    )
  }
}
