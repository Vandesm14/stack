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
