use std::collections::HashMap;

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

  fn parse_line(&mut self, line: String) {
    let tokens = stack::parse(line.clone());
    self.stack.extend(tokens);
  }

  fn eval(&mut self) {
    let last = self.stack.pop();
    let Some(Token::Symbol(symbol)) = last else {
      if let Some(token) = last {
        self.stack.push(token);
      }
      return;
    };

    // Only evaluate if we have a symbol at the end
    match symbol.as_str() {
      "+" => {
        let a = self.stack.pop();
        let b = self.stack.pop();
        if let (Some(Token::Integer(a)), Some(Token::Integer(b))) = (a, b) {
          self.stack.push(Token::Integer(a + b));
        } else {
          panic!("Invalid operation");
        }
      }
      "-" => {
        let a = self.stack.pop();
        let b = self.stack.pop();
        if let (Some(Token::Integer(a)), Some(Token::Integer(b))) = (a, b) {
          self.stack.push(Token::Integer(a - b));
        } else {
          panic!("Invalid operation");
        }
      }
      "*" => {
        let a = self.stack.pop();
        let b = self.stack.pop();
        if let (Some(Token::Integer(a)), Some(Token::Integer(b))) = (a, b) {
          self.stack.push(Token::Integer(a * b));
        } else {
          panic!("Invalid operation");
        }
      }
      "/" => {
        let a = self.stack.pop();
        let b = self.stack.pop();
        if let (Some(Token::Integer(a)), Some(Token::Integer(b))) = (a, b) {
          self.stack.push(Token::Integer(a / b));
        } else {
          panic!("Invalid operation");
        }
      }
      "set" => {
        let a = self.stack.pop();
        let b = self.stack.pop();
        if let (Some(Token::Symbol(a)), Some(b)) = (a, b) {
          self.scope.insert(a, b);
        } else {
          panic!("Invalid operation");
        }
      }
      _ => {
        // If we can't find a function for it, push it back to the stack
        self.stack.push(Token::Symbol(symbol));
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

        program.parse_line(line);
        program.eval();
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
