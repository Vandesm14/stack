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

  pub fn eval_string(&mut self, line: String) {
    let tokens = crate::lex(line.clone());
    let exprs = crate::parse(tokens);

    self.eval(exprs);
  }

  fn eval_call(&mut self, call: String) {
    match call.as_str() {
      "+" => {
        let a = self.pop_eval();
        let b = self.pop_eval();
        if let (Expr::Integer(a), Expr::Integer(b)) = (a, b) {
          self.stack.push(Expr::Integer(a + b));
        } else {
          panic!("Invalid args for: {}", call);
        }
      }
      "-" => {
        let a = self.pop_eval();
        let b = self.pop_eval();
        if let (Expr::Integer(a), Expr::Integer(b)) = (a, b) {
          self.stack.push(Expr::Integer(a - b));
        } else {
          panic!("Invalid args for: {}", call);
        }
      }
      "*" => {
        let a = self.pop_eval();
        let b = self.pop_eval();
        if let (Expr::Integer(a), Expr::Integer(b)) = (a, b) {
          self.stack.push(Expr::Integer(a * b));
        } else {
          panic!("Invalid args for: {}", call);
        }
      }
      "/" => {
        let a = self.pop_eval();
        let b = self.pop_eval();
        if let (Expr::Integer(a), Expr::Integer(b)) = (a, b) {
          self.stack.push(Expr::Integer(a / b));
        } else {
          panic!("Invalid args for: {}", call);
        }
      }
      "set" => {
        let a = self.pop_eval();
        let b = self.pop_eval();
        if let (Expr::Symbol(a), b) = (a, b) {
          self.scope.insert(a, b);
        } else {
          panic!("Invalid args for: {}", call);
        }
      }
      "clear" => {
        self.stack.clear();
      }
      "pop" => {
        self.stack.pop();
      }
      "dup" => {
        if self.stack.is_empty() {
          panic!("Not enough items on stack");
        }

        let a = self.pop_eval();
        self.stack.push(a.clone());
        self.stack.push(a);
      }
      "swap" => {
        if self.stack.len() < 2 {
          panic!("Not enough items on stack");
        }

        let a = self.pop_eval();
        let b = self.pop_eval();
        self.stack.push(a);
        self.stack.push(b);
      }
      "iswap" => {
        if self.stack.len() < 2 {
          panic!("Not enough items on stack");
        }

        let index = self.pop_eval();
        if let Expr::Integer(index) = index {
          let len = self.stack.len();
          self.stack.swap(len - 1, index as usize);
        } else {
          panic!("Invalid args for: {}", call);
        }
      }
      "call" => {
        let a = self.stack.pop();
        if let Some(Expr::Symbol(a)) | Some(Expr::Call(a)) = a {
          self.eval_string(a);
        } else {
          panic!("Invalid operation");
        }
      }
      _ => {
        if let Some(value) = self.scope.get(&call) {
          let result = self.eval_expr(value.clone());
          self.stack.push(result);
        }
      }
    }
  }

  fn eval_expr(&mut self, expr: Expr) -> Expr {
    println!("Eval: {:?}", expr);

    match expr {
      Expr::Call(call) => {
        self.eval_call(call);
        Expr::Nil
      }
      Expr::List(list) => {
        let mut exprs: Vec<Expr> = Vec::new();
        for expr in list {
          exprs.push(self.eval_expr(expr));
        }

        Expr::List(exprs)
      }
      _ => expr,
    }
  }

  fn pop_eval(&mut self) -> Expr {
    let expr = self.stack.pop().unwrap();
    self.eval_expr(expr)
  }

  pub fn eval(&mut self, exprs: Vec<Expr>) {
    for expr in exprs {
      match expr {
        Expr::Call(call) => {
          self.eval_call(call);
        }
        _ => {
          self.stack.push(expr);
        }
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
