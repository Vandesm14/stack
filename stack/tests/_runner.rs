use core::str::FromStr;
use stack::{
  engine::{RunError, RunErrorReason},
  prelude::*,
};
use std::{path::PathBuf, rc::Rc};
use test_case::case;

#[inline]
const fn e(kind: ExprKind) -> Expr {
  Expr { kind, info: None }
}

#[case("intrinsics/arithmetic.stack" => Ok(vec![e(ExprKind::Integer(3)), e(ExprKind::Integer(-1)), e(ExprKind::Integer(6)), e(ExprKind::Integer(2)), e(ExprKind::Integer(0))]) ; "arithmetic")]
#[case("intrinsics/compare.stack" => Ok(vec![e(ExprKind::Boolean(true)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(false))]) ; "compare")]
#[case("intrinsics/logical.stack" => Ok(vec![e(ExprKind::Boolean(false)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(false))]) ; "logical")]
#[case("intrinsics/assert_fail.stack" => Err(RunError { reason: RunErrorReason::AssertionFailed, expr: e(ExprKind::Integer(123)) }) ; "assert fail")]
#[case("intrinsics/assert_okay.stack" => Ok(vec![]) ; "assert okay")]
fn integration(subpath: &str) -> Result<Vec<Expr>, RunError> {
  let mut path = PathBuf::from_str("tests").unwrap();
  path.push(subpath);

  let source = Rc::new(FileSource::new(path).unwrap());
  let exprs = Parser::new(Lexer::new(source)).parse().unwrap();

  let engine = Engine::new().with_track_info(false);
  let mut context = Context::new().with_stack_capacity(32);
  context = engine.run(context, exprs)?;

  Ok(core::mem::take(context.stack_mut()))
}
