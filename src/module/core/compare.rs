use crate::{interner::interner, DebugData, EvalError, ExprKind, Program};

pub fn module(program: &mut Program) -> Result<(), EvalError> {
  program.funcs.insert(
    interner().get_or_intern_static("="),
    |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      program.push(ExprKind::Boolean(lhs.val == rhs.val).into_expr(
        DebugData::only_ingredients(vec![lhs, rhs, trace_expr.clone()]),
      ))
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("!="),
    |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      program.push(ExprKind::Boolean(lhs.val != rhs.val).into_expr(
        DebugData::only_ingredients(vec![lhs, rhs, trace_expr.clone()]),
      ))
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("<"),
    |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      program.push(ExprKind::Boolean(lhs.val < rhs.val).into_expr(
        DebugData::only_ingredients(vec![lhs, rhs, trace_expr.clone()]),
      ))
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static(">"),
    |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      program.push(ExprKind::Boolean(lhs.val > rhs.val).into_expr(
        DebugData::only_ingredients(vec![lhs, rhs, trace_expr.clone()]),
      ))
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static("<="),
    |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      program.push(ExprKind::Boolean(lhs.val <= rhs.val).into_expr(
        DebugData::only_ingredients(vec![lhs, rhs, trace_expr.clone()]),
      ))
    },
  );

  program.funcs.insert(
    interner().get_or_intern_static(">="),
    |program, trace_expr| {
      let rhs = program.pop(trace_expr)?;
      let lhs = program.pop(trace_expr)?;

      program.push(ExprKind::Boolean(lhs.val >= rhs.val).into_expr(
        DebugData::only_ingredients(vec![lhs, rhs, trace_expr.clone()]),
      ))
    },
  );

  Ok(())
}

// #[cfg(test)]

// mod tests {
//   use super::*;

//   mod greater_than {
//     use super::*;

//     #[test]
//     fn greater_than_int() {
//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("1 1 >").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(false)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("1 2 >").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(false)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("2 1 >").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(true)]);
//     }

//     #[test]
//     fn greater_than_float() {
//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("1.0 1.0 >").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(false)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("1.0 1.1 >").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(false)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("1.1 1.0 >").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(true)]);
//     }

//     #[test]
//     fn greater_than_int_and_float() {
//       // Int first
//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("1 1.0 >").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(false)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("1 1.1 >").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(false)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("2 1.0 >").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(true)]);

//       // Float first
//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("1.0 1 >").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(false)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("1.0 1 >").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(false)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("1.1 1 >").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(true)]);
//     }
//   }

//   mod less_than {
//     use super::*;

//     #[test]
//     fn less_than_int() {
//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("1 1 <").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(false)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("1 2 <").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(true)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("2 1 <").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(false)]);
//     }

//     #[test]
//     fn less_than_float() {
//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("1.0 1.0 <").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(false)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("1.0 1.1 <").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(true)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("1.1 1.0 <").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(false)]);
//     }

//     #[test]
//     fn less_than_int_and_float() {
//       // Int first
//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("1 1.0 <").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(false)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("1 1.1 <").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(true)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("2 1.0 <").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(false)]);

//       // Float first
//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("1.0 1 <").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(false)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("0.9 1 <").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(true)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("1.1 1 <").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(false)]);
//     }
//   }

//   mod bitwise {
//     use super::*;

//     #[test]
//     fn and_int() {
//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("1 1 and").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(true)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("1 0 and").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(false)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("0 1 and").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(false)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("0 0 and").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(false)]);
//     }

//     #[test]
//     fn and_bool() {
//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("true true and").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(true)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("true false and").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(false)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("false true and").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(false)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("false false and").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(false)]);
//     }

//     #[test]
//     fn or_int() {
//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("1 1 or").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(true)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("1 0 or").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(true)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("0 1 or").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(true)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("0 0 or").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(false)]);
//     }

//     #[test]
//     fn or_bool() {
//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("true true or").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(true)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("true false or").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(true)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("false true or").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(true)]);

//       let mut program = Program::new().with_core().unwrap();
//       program.eval_string("false false or").unwrap();
//       assert_eq!(program.stack, vec![Expr::Boolean(false)]);
//     }
//   }
// }
