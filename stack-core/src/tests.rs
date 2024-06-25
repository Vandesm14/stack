#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn lisp_syntax() {
    let source = Source::new("", "(+ 2 2) (def 'a _) a");
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
      vec![&ExprKind::Integer(4),]
    );
  }
}
