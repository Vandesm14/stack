use core::str::FromStr;
use stack::prelude::*;
use std::{path::PathBuf, rc::Rc};
use test_case::case;

#[inline]
const fn e(kind: ExprKind) -> Expr {
  Expr { kind, info: None }
}

#[case("intrinsics/arithmetic.stack" => vec![e(ExprKind::Integer(3)), e(ExprKind::Integer(-1)), e(ExprKind::Integer(6)), e(ExprKind::Integer(2)), e(ExprKind::Integer(0))] ; "arithmetic")]
#[case("intrinsics/compare.stack" => vec![e(ExprKind::Boolean(true)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(false))] ; "compare")]
#[case("intrinsics/logical.stack" => vec![e(ExprKind::Boolean(false)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(false))] ; "logical")]
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
