use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

enum Value {
  Integer(usize),
  Float(f64),
  String(String),
  Symbol(String),
  Nil,
}

struct Program {
  stack: Vec<Value>,
}

impl Program {
  fn new() -> Self {
    Self { stack: vec![] }
  }
}

fn main() -> Result<()> {
  // `()` can be used when no completer is required
  let mut rl = DefaultEditor::new()?;

  loop {
    let readline = rl.readline(">> ");
    match readline {
      Ok(line) => {
        rl.add_history_entry(line.as_str());
        println!("Line: {}", line);
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
