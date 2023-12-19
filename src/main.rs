use std::collections::HashMap;
use std::mem::swap;

use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use stack::Token;

struct Program {
  stack: Vec<Token>,
  scope: HashMap<String, Token>,
}

impl Program {
  fn new() -> Self {
    Self {
      stack: vec![],
      scope: HashMap::new(),
    }
  }

  fn eval(&mut self, line: String) {
    let tokens = stack::parse(line.clone());

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

fn main() -> Result<()> {
  // `()` can be used when no completer is required
  let mut rl = DefaultEditor::new()?;
  let mut program = Program::new();

  loop {
    let readline = rl.readline(">> ");
    match readline {
      Ok(line) => {
        rl.add_history_entry(line.as_str());

        program.eval(line);
        println!("Stack: {:?}", program.stack);
        println!("Scope: {:?}", program.scope);
      }
      Err(ReadlineError::Interrupted) => {
        println!("CTRL-C");
        break;
      }
      Err(ReadlineError::Eof) => {
        println!("CTRL-D");
        break;
      }
      Err(err) => {
        println!("Error: {:?}", err);
        break;
      }
    }
  }

  Ok(())
}
