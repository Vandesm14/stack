#[cfg(test)]
mod lispy {
  use std::collections::HashMap;

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
  fn less_arity_pops() {
    let source = Source::new("", "2 (- 10)");
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

  #[test]
  fn lisp_evaluates_eagerly() {
    let source = Source::new("", "(- (+ 8 2) (+ 0 2))");
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
  fn insert_works() {
    let source = Source::new("", "(insert {} \"key\" \"value\")");
    let mut lexer = Lexer::new(source);
    let exprs = crate::parser::parse(&mut lexer).unwrap();

    let engine = Engine::new();
    let mut context = Context::new().with_stack_capacity(32);
    context = engine.run(context, exprs).unwrap();

    assert_eq!(
      context
        .stack()
        .to_vec()
        .iter_mut()
        .map(|expr| {
          expr.recursively_strip_info();
          &expr.kind
        })
        .collect::<Vec<_>>(),
      vec![&ExprKind::Record(HashMap::from_iter(vec![(
        Symbol::new("key".to_owned().into()),
        ExprKind::String("value".into()).into()
      )]))]
    )
  }

  #[test]
  fn insert_works_outer_ordered() {
    let source = Source::new("", "\"value\" (insert {} \"key\" _)");
    let mut lexer = Lexer::new(source);
    let exprs = crate::parser::parse(&mut lexer).unwrap();

    let engine = Engine::new();
    let mut context = Context::new().with_stack_capacity(32);
    context = engine.run(context, exprs).unwrap();

    assert_eq!(
      context
        .stack()
        .to_vec()
        .iter_mut()
        .map(|expr| {
          expr.recursively_strip_info();
          &expr.kind
        })
        .collect::<Vec<_>>(),
      vec![&ExprKind::Record(HashMap::from_iter(vec![(
        Symbol::new("key".to_owned().into()),
        ExprKind::String("value".into()).into()
      )]))]
    )
  }

  #[test]
  fn insert_works_ordered() {
    let source = Source::new("", "\"value\" {} (insert _ \"key\" _)");
    let mut lexer = Lexer::new(source);
    let exprs = crate::parser::parse(&mut lexer).unwrap();

    let engine = Engine::new();
    let mut context = Context::new().with_stack_capacity(32);
    context = engine.run(context, exprs).unwrap();

    assert_eq!(
      context
        .stack()
        .to_vec()
        .iter_mut()
        .map(|expr| {
          expr.recursively_strip_info();
          &expr.kind
        })
        .collect::<Vec<_>>(),
      vec![&ExprKind::Record(HashMap::from_iter(vec![(
        Symbol::new("key".to_owned().into()),
        ExprKind::String("value".into()).into()
      )]))]
    )
  }

  #[test]
  fn insert_works_inner_ordered() {
    let source = Source::new("", "{} (insert _ \"key\" \"value\")");
    let mut lexer = Lexer::new(source);
    let exprs = crate::parser::parse(&mut lexer).unwrap();

    let engine = Engine::new();
    let mut context = Context::new().with_stack_capacity(32);
    context = engine.run(context, exprs).unwrap();

    assert_eq!(
      context
        .stack()
        .to_vec()
        .iter_mut()
        .map(|expr| {
          expr.recursively_strip_info();
          &expr.kind
        })
        .collect::<Vec<_>>(),
      vec![&ExprKind::Record(HashMap::from_iter(vec![(
        Symbol::new("key".to_owned().into()),
        ExprKind::String("value".into()).into()
      )]))]
    )
  }
}
