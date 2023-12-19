use crate::Expr;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Program {
  pub stack: Vec<Expr>,
  pub scope: HashMap<String, Expr>,
}

impl Program {
  pub fn new() -> Self {
    Self {
      stack: vec![],
      scope: HashMap::new(),
    }
  }

  fn pop_eval(&mut self) -> Expr {
    let expr = self.stack.pop();
    if let Some(expr) = expr {
      if let Some(result) = self.eval_expr(expr) {
        result
      } else {
        Expr::Nil
      }
    } else {
      Expr::Nil
    }
  }

  fn eval_call(&mut self, call: String) -> Option<Expr> {
    match call.as_str() {
      "+" => {
        let a = self.pop_eval();
        let b = self.pop_eval();
        if let (Expr::Integer(a), Expr::Integer(b)) = (a, b) {
          Some(Expr::Integer(a + b))
        } else {
          panic!("Invalid args for: {}", call);
        }
      }
      "-" => {
        let a = self.pop_eval();
        let b = self.pop_eval();
        if let (Expr::Integer(a), Expr::Integer(b)) = (a, b) {
          Some(Expr::Integer(a - b))
        } else {
          panic!("Invalid args for: {}", call);
        }
      }
      "*" => {
        let a = self.pop_eval();
        let b = self.pop_eval();
        if let (Expr::Integer(a), Expr::Integer(b)) = (a, b) {
          Some(Expr::Integer(a * b))
        } else {
          panic!("Invalid args for: {}", call);
        }
      }
      "/" => {
        let a = self.pop_eval();
        let b = self.pop_eval();
        if let (Expr::Integer(a), Expr::Integer(b)) = (a, b) {
          Some(Expr::Integer(a / b))
        } else {
          panic!("Invalid args for: {}", call);
        }
      }
      "set" => {
        let a = self.pop_eval();
        let b = self.pop_eval();
        if let (Expr::Symbol(a), b) = (a, b) {
          self.scope.insert(a, b);
          None
        } else {
          panic!("Invalid args for: {}", call);
        }
      }
      "unset" => {
        let a = self.pop_eval();
        if let Expr::Symbol(a) = a {
          self.scope.remove(&a);
          None
        } else {
          panic!("Invalid args for: {}", call);
        }
      }
      "clear" => {
        self.stack.clear();
        None
      }
      "pop" => {
        self.stack.pop();
        None
      }
      "dup" => {
        if self.stack.is_empty() {
          panic!("Not enough items on stack");
        }

        let a = self.pop_eval();
        self.stack.push(a.clone());
        self.stack.push(a);
        None
      }
      "swap" => {
        if self.stack.len() < 2 {
          panic!("Not enough items on stack");
        }

        let a = self.pop_eval();
        let b = self.pop_eval();
        self.stack.push(a);
        self.stack.push(b);
        None
      }
      "iswap" => {
        if self.stack.len() < 2 {
          panic!("Not enough items on stack");
        }

        let index = self.pop_eval();
        if let Expr::Integer(index) = index {
          let len = self.stack.len();
          self.stack.swap(len - 1, index as usize);
          None
        } else {
          panic!("Invalid args for: {}", call);
        }
      }
      "call" => {
        let a = self.stack.pop();

        if let Some(Expr::Symbol(a)) | Some(Expr::Call(a)) = a {
          self.eval_expr(Expr::Call(a))
        } else {
          panic!("Invalid args for: {}", call);
        }
      }
      "unwrap" => {
        let a = self.pop_eval();
        if let Expr::List(a) | Expr::Block(a) = a {
          for expr in a {
            self.stack.push(expr);
          }
          None
        } else {
          panic!("Invalid args for: {}", call);
        }
      }
      _ => {
        if let Some(value) = self.scope.get(&call) {
          self.eval_expr(value.clone())
        } else {
          eprintln!("Unknown call: {}", call);
          None
        }
      }
    }
  }

  fn eval_expr(&mut self, expr: Expr) -> Option<Expr> {
    match expr {
      Expr::Call(call) => self.eval_call(call),
      Expr::List(list) => {
        let exprs: Vec<Expr> = list
          .into_iter()
          .filter_map(|expr| self.eval_expr(expr))
          .collect();

        Some(Expr::List(exprs))
      }
      _ => Some(expr),
    }
  }

  pub fn eval_string(&mut self, line: String) {
    let tokens = crate::lex(line);
    let exprs = crate::parse(tokens);

    self.eval(exprs);
  }

  pub fn eval(&mut self, exprs: Vec<Expr>) {
    for expr in exprs {
      let result = self.eval_expr(expr);

      if let Some(expr) = result {
        self.stack.push(expr);
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod eval {
    use super::*;

    #[test]
    fn implicitly_adds_to_stack() {
      let mut program = Program::new();
      program.eval_string("1 2".to_string());
      assert_eq!(program.stack, vec![Expr::Integer(1), Expr::Integer(2)]);
    }

    #[test]
    fn symbols_are_pushed() {
      let mut program = Program::new();
      program.eval_string("'a".to_string());
      assert_eq!(program.stack, vec![Expr::Symbol("a".to_string())]);
    }

    #[test]
    fn add_two_numbers() {
      let mut program = Program::new();
      program.eval_string("1 2 +".to_string());
      assert_eq!(program.stack, vec![Expr::Integer(3)]);
    }

    #[test]
    fn complex_operations() {
      let mut program = Program::new();
      program.eval_string("1 2 + 3 *".to_string());
      assert_eq!(program.stack, vec![Expr::Integer(9)]);
    }

    #[test]
    fn eval_from_stack() {
      let mut program = Program::new();
      program.eval_string("1 2 '+ call".to_string());
      assert_eq!(program.stack, vec![Expr::Integer(3)]);
    }

    #[test]
    fn dont_eval_blocks() {
      let mut program = Program::new();
      program.eval_string("6 'var set (var)".to_string());
      assert_eq!(
        program.stack,
        vec![Expr::Block(vec![Expr::Call("var".to_string())])]
      );
    }

    #[test]
    fn dont_eval_blocks_symbols() {
      let mut program = Program::new();
      program.eval_string("6 'var set ('var)".to_string());
      assert_eq!(
        program.stack,
        vec![Expr::Block(vec![Expr::Symbol("var".to_string())])]
      );
    }

    #[test]
    fn eval_lists() {
      let mut program = Program::new();
      program.eval_string("[1 2 3]".to_string());
      assert_eq!(
        program.stack,
        vec![Expr::List(vec![
          Expr::Integer(1),
          Expr::Integer(2),
          Expr::Integer(3)
        ])]
      );
    }

    #[test]
    fn eval_lists_eagerly() {
      let mut program = Program::new();
      program.eval_string("6 'var set [var]".to_string());
      assert_eq!(program.stack, vec![Expr::List(vec![Expr::Integer(6)])]);
    }
  }

  mod variables {
    use super::*;

    #[test]
    fn storing_variables() {
      let mut program = Program::new();
      program.eval_string("1 'a set".to_string());
      assert_eq!(
        program.scope,
        HashMap::from_iter(vec![("a".to_string(), Expr::Integer(1))])
      );
    }

    #[test]
    fn retrieving_variables() {
      let mut program = Program::new();
      program.eval_string("1 'a set a".to_string());
      assert_eq!(program.stack, vec![Expr::Integer(1)]);
    }

    #[test]
    fn evaluating_variables() {
      let mut program = Program::new();
      program.eval_string("1 'a set a 2 +".to_string());
      assert_eq!(program.stack, vec![Expr::Integer(3)]);
    }

    #[test]
    fn removing_variables() {
      let mut program = Program::new();
      program.eval_string("1 'a set 'a unset".to_string());
      assert_eq!(program.scope, HashMap::new());
    }
  }

  mod stack_ops {
    use super::*;

    #[test]
    fn clearing_stack() {
      let mut program = Program::new();
      program.eval_string("1 2 clear".to_string());
      assert_eq!(program.stack, vec![]);
    }

    #[test]
    fn popping_from_stack() {
      let mut program = Program::new();
      program.eval_string("1 2 pop".to_string());
      assert_eq!(program.stack, vec![Expr::Integer(1)]);
    }

    #[test]
    fn duplicating_stack_item() {
      let mut program = Program::new();
      program.eval_string("1 dup".to_string());
      assert_eq!(program.stack, vec![Expr::Integer(1), Expr::Integer(1)]);
    }

    #[test]
    fn swapping_stack_items() {
      let mut program = Program::new();
      program.eval_string("1 2 swap".to_string());
      assert_eq!(program.stack, vec![Expr::Integer(2), Expr::Integer(1)]);
    }

    #[test]
    fn swapping_with_index() {
      let mut program = Program::new();
      program.eval_string("1 2 3 4 0 iswap".to_string());
      assert_eq!(
        program.stack,
        vec![
          Expr::Integer(4),
          Expr::Integer(2),
          Expr::Integer(3),
          Expr::Integer(1)
        ]
      );
    }

    #[test]
    fn swapping_with_index2() {
      let mut program = Program::new();
      program.eval_string("1 2 3 4 1 iswap".to_string());
      assert_eq!(
        program.stack,
        vec![
          Expr::Integer(1),
          Expr::Integer(4),
          Expr::Integer(3),
          Expr::Integer(2)
        ]
      );
    }
  }
}
