use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use stack::Token;

struct Program {
  stack: Vec<Token>,
}

impl Program {
  fn new() -> Self {
    Self { stack: vec![] }
  }

  fn parse_line(&mut self, line: String) {
    let tokens = stack::parse(line);
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
      _ => {}
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
        println!("{:?}", program.stack);
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
