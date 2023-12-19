use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use stack::Program;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
  #[command(subcommand)]
  command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
  #[command(about = "Run a file")]
  Run {
    path: PathBuf,

    #[arg(long)]
    watch: bool,
  },
}

fn repl() -> Result<()> {
  let mut rl = DefaultEditor::new()?;
  let mut program = Program::new();

  loop {
    let readline = rl.readline(">> ");
    match readline {
      Ok(line) => {
        rl.add_history_entry(line.as_str()).unwrap();

        program.eval_string(line);
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

fn main() {
  let cli = Cli::parse();

  match cli.command {
    Some(Commands::Run { path, watch }) => match fs::read(path) {
      Ok(contents) => {
        let contents = String::from_utf8(contents).unwrap();
        let tokens = stack::lex(contents);
        let exprs = stack::parse(tokens);

        let mut program = Program::new();
        program.eval(exprs);

        println!("Stack: {:?}", program.stack);
      }
      Err(err) => {
        eprintln!("Error: {:?}", err);
      }
    },
    None => {
      println!("Running REPL");
      repl().unwrap();
    }
  }
}
