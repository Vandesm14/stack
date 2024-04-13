use core::str::FromStr;
use stack::prelude::*;
use std::{path::PathBuf, rc::Rc};
use test_case::case;

#[case("simple.stack" => vec![Expr { kind: ExprKind::Integer(3), info: None }, Expr { kind: ExprKind::Integer(-1), info: None }, Expr { kind: ExprKind::Integer(6), info: None }, Expr { kind: ExprKind::Integer(2), info: None }, Expr { kind: ExprKind::Integer(0), info: None }] ; "simple")]
fn integration(subpath: &str) -> Vec<Expr> {
  let mut path = PathBuf::from_str("tests").unwrap();
  path.push(subpath);

  let source = Rc::new(FileSource::new(path).unwrap());
  let exprs = Parser::new(Lexer::new(source)).parse().unwrap();

  let engine = Engine::new().with_track_info(false);
  let mut context = Context::new().with_stack_capacity(32);
  context = engine.run(context, exprs).unwrap();

  core::mem::take(context.stack_mut())
}
