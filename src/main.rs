use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use stack::Program;

fn main() -> Result<()> {
  // `()` can be used when no completer is required
  let mut rl = DefaultEditor::new()?;
  let mut program = Program::new();

  loop {
    let readline = rl.readline(">> ");
    match readline {
      Ok(line) => {
        rl.add_history_entry(line.as_str()).unwrap();

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
