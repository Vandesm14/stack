use core::str::FromStr;
use std::path::PathBuf;

use stack_core::prelude::*;
use test_case::case;

#[inline]
const fn e(kind: ExprKind) -> Expr {
  Expr { kind, info: None }
}

// TODO: Add tests for missing intrinsics.

#[case("intrinsics/arithmetic.stack" => Ok(vec![e(ExprKind::Integer(3)), e(ExprKind::Integer(-1)), e(ExprKind::Integer(6)), e(ExprKind::Integer(2)), e(ExprKind::Integer(0))]) ; "arithmetic")]
#[case("intrinsics/compare.stack" => Ok(vec![e(ExprKind::Boolean(true)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(false))]) ; "compare")]
#[case("intrinsics/logical.stack" => Ok(vec![e(ExprKind::Boolean(false)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(false)), e(ExprKind::Boolean(true)), e(ExprKind::Boolean(false))]) ; "logical")]
// TODO: Fix this.
// #[case("intrinsics/assert_fail.stack" => Err(RunError { reason: RunErrorReason::AssertionFailed, expr: e(ExprKind::Integer(123)) }) ; "assert fail")]
#[case("intrinsics/assert_okay.stack" => Ok(vec![]) ; "assert okay")]
#[case("intrinsics/stack.stack" => Ok(vec![e(ExprKind::Integer(1)), e(ExprKind::Integer(3)), e(ExprKind::Integer(3)), e(ExprKind::Integer(5)), e(ExprKind::Integer(4)), e(ExprKind::Integer(7)), e(ExprKind::Integer(8)), e(ExprKind::Integer(6))]) ; "stack")]
#[case("intrinsics/orelse.stack" => Ok(vec![e(ExprKind::Integer(1)), e(ExprKind::Integer(2)), e(ExprKind::Integer(1)), e(ExprKind::Nil)]) ; "orelse")]
#[case("intrinsics/push.stack" => Ok(vec![e(ExprKind::List(vec![e(ExprKind::Integer(1)), e(ExprKind::Integer(2)), e(ExprKind::Integer(3))])), e(ExprKind::String("he".into())), e(ExprKind::String("he".into()))]) ; "push")]
#[case("intrinsics/record.stack" => Ok(vec![e(ExprKind::Boolean(true)),e(ExprKind::Boolean(true)),e(ExprKind::Boolean(true)),e(ExprKind::Boolean(true)),e(ExprKind::Boolean(true)),e(ExprKind::Boolean(true)),e(ExprKind::Boolean(true)),e(ExprKind::Boolean(true)),e(ExprKind::Boolean(false)),e(ExprKind::Boolean(false)),]) ; "record")]
#[case("intrinsics/pop.stack" => Ok(vec![e(ExprKind::List(vec![e(ExprKind::Integer(1)), e(ExprKind::Integer(2))])), e(ExprKind::Integer(3)), e(ExprKind::String("h".into())), e(ExprKind::String("e".into()))]) ; "pop")]
fn integration(subpath: &str) -> Result<Vec<Expr>, RunError> {
  let mut path = PathBuf::from_str("tests").unwrap();
  path.push(subpath);

  let source = Source::from_path(path).unwrap();
  let mut lexer = Lexer::new(source);
  let exprs = crate::parse(&mut lexer).unwrap();

  let engine = Engine::new();
  let mut context = Context::new().with_stack_capacity(32);
  context = engine.run(context, exprs)?;

  Ok(core::mem::take(context.stack_mut()))
}
