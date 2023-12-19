use crate::Token;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Program {
  pub stack: Vec<Token>,
  pub scope: HashMap<String, Token>,
}

impl Program {
  pub fn new() -> Self {
    Self {
      stack: vec![],
      scope: HashMap::new(),
    }
  }

  pub fn eval(&mut self, line: String) {
    let tokens = crate::parse(line.clone());

    for token in tokens {
      match token {
        Token::Call(call) => match call.as_str() {
          "+" => {
            let a = self.stack.pop();
            let b = self.stack.pop();
            if let (Some(Token::Integer(a)), Some(Token::Integer(b))) = (a, b) {
              self.stack.push(Token::Integer(a + b));
            } else {
              panic!("Invalid args for: {}", call);
            }
          }
          "-" => {
            let a = self.stack.pop();
            let b = self.stack.pop();
            if let (Some(Token::Integer(a)), Some(Token::Integer(b))) = (a, b) {
              self.stack.push(Token::Integer(a - b));
            } else {
              panic!("Invalid args for: {}", call);
            }
          }
          "*" => {
            let a = self.stack.pop();
            let b = self.stack.pop();
            if let (Some(Token::Integer(a)), Some(Token::Integer(b))) = (a, b) {
              self.stack.push(Token::Integer(a * b));
            } else {
              panic!("Invalid args for: {}", call);
            }
          }
          "/" => {
            let a = self.stack.pop();
            let b = self.stack.pop();
            if let (Some(Token::Integer(a)), Some(Token::Integer(b))) = (a, b) {
              self.stack.push(Token::Integer(a / b));
            } else {
              panic!("Invalid args for: {}", call);
            }
          }
          "set" => {
            let a = self.stack.pop();
            let b = self.stack.pop();
            if let (Some(Token::Symbol(a)), Some(b)) = (a, b) {
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
            let a = self.stack.pop();
            if let Some(a) = a {
              self.stack.push(a.clone());
              self.stack.push(a);
            } else {
              panic!("Invalid args for: {}", call);
            }
          }
          "swap" => {
            let a = self.stack.pop();
            let b = self.stack.pop();
            if let (Some(a), Some(b)) = (a, b) {
              self.stack.push(a);
              self.stack.push(b);
            } else {
              panic!("Invalid args for: {}", call);
            }
          }
          "iswap" => {
            let index = self.stack.pop();
            if let Some(Token::Integer(index)) = index {
              let len = self.stack.len();
              self.stack.swap(len - 1, index as usize);
            } else {
              panic!("Invalid args for: {}", call);
            }
          }
          "call" => {
            let a = self.stack.pop();
            if let Some(Token::Symbol(a)) | Some(Token::Call(a)) = a {
              self.eval(a);
            } else {
              panic!("Invalid operation");
            }
          }
          _ => {
            // Try to evaluate from scope
            if let Some(value) = self.scope.get(&call) {
              self.stack.push(value.clone());
            } else {
              panic!("Unknown call: {}", call);
            }
          }
        },
        _ => {
          self.stack.push(token);
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
      program.eval("1 2".to_string());
      assert_eq!(program.stack, vec![Token::Integer(1), Token::Integer(2)]);
    }

    #[test]
    fn symbols_are_pushed() {
      let mut program = Program::new();
      program.eval("'a".to_string());
      assert_eq!(program.stack, vec![Token::Symbol("a".to_string())]);
    }

    #[test]
    fn add_two_numbers() {
      let mut program = Program::new();
      program.eval("1 2 +".to_string());
      assert_eq!(program.stack, vec![Token::Integer(3)]);
    }

    #[test]
    fn complex_operations() {
      let mut program = Program::new();
      program.eval("1 2 + 3 *".to_string());
      assert_eq!(program.stack, vec![Token::Integer(9)]);
    }
  }

  mod variables {
    use super::*;

    #[test]
    fn storing_variables() {
      let mut program = Program::new();
      program.eval("1 'a set".to_string());
      assert_eq!(
        program.scope,
        HashMap::from_iter(vec![("a".to_string(), Token::Integer(1))])
      );
    }

    #[test]
    fn retrieving_variables() {
      let mut program = Program::new();
      program.eval("1 'a set a".to_string());
      assert_eq!(program.stack, vec![Token::Integer(1)]);
    }

    #[test]
    fn evaluating_variables() {
      let mut program = Program::new();
      program.eval("1 'a set a 2 +".to_string());
      assert_eq!(program.stack, vec![Token::Integer(3)]);
    }
  }

  mod stack_ops {
    use super::*;

    #[test]
    fn clearing_stack() {
      let mut program = Program::new();
      program.eval("1 2 clear".to_string());
      assert_eq!(program.stack, vec![]);
    }

    #[test]
    fn popping_from_stack() {
      let mut program = Program::new();
      program.eval("1 2 pop".to_string());
      assert_eq!(program.stack, vec![Token::Integer(1)]);
    }

    #[test]
    fn duplicating_stack_item() {
      let mut program = Program::new();
      program.eval("1 dup".to_string());
      assert_eq!(program.stack, vec![Token::Integer(1), Token::Integer(1)]);
    }

    #[test]
    fn swapping_stack_items() {
      let mut program = Program::new();
      program.eval("1 2 swap".to_string());
      assert_eq!(program.stack, vec![Token::Integer(2), Token::Integer(1)]);
    }

    #[test]
    fn swapping_with_index() {
      let mut program = Program::new();
      program.eval("1 2 3 4 0 iswap".to_string());
      assert_eq!(
        program.stack,
        vec![
          Token::Integer(4),
          Token::Integer(2),
          Token::Integer(3),
          Token::Integer(1)
        ]
      );
    }

    #[test]
    fn swapping_with_index2() {
      let mut program = Program::new();
      program.eval("1 2 3 4 1 iswap".to_string());
      assert_eq!(
        program.stack,
        vec![
          Token::Integer(1),
          Token::Integer(4),
          Token::Integer(3),
          Token::Integer(2)
        ]
      );
    }
  }
}