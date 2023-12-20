use std::fs;
use std::io::stdout;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use crossterm::terminal::{Clear, ClearType};
use crossterm::{cursor, execute};
use notify::event::AccessKind;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
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

fn eval_string(program: &Program, result: Result<(), String>) {
  println!("Stack: {:?}", program.stack);
  println!();
  println!("Scope: {:?}", program.scope);

  if let Err(err) = result {
    eprintln!("Error: {}", err);
  }
}

fn repl() -> rustyline::Result<()> {
  let mut rl = DefaultEditor::new()?;
  let mut program = Program::new();

  loop {
    let readline = rl.readline(">> ");
    match readline {
      Ok(line) => {
        rl.add_history_entry(line.as_str()).unwrap();

        let result = program.eval_string(line);
        eval_string(&program, result);
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

fn eval_file(path: PathBuf, is_watching: bool) {
  let mut stdout = stdout();

  match fs::read(path) {
    Ok(contents) => {
      let contents = String::from_utf8(contents).unwrap();
      let mut program = Program::new();

      if is_watching {
        execute!(stdout, Clear(ClearType::All)).unwrap();
        execute!(stdout, cursor::MoveTo(0, 0)).unwrap();
      }

      let result = program.eval_string(contents);
      eval_string(&program, result);

      if is_watching {
        println!();
        println!("Watching file for changes...");
      }
    }
    Err(err) => {
      eprintln!("Error: {:?}", err);
    }
  }
}

fn main() {
  let cli = Cli::parse();

  match cli.command {
    Some(Commands::Run { path, watch }) => match watch {
      true => {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher =
          RecommendedWatcher::new(tx, Config::default()).unwrap();
        watcher.watch(&path, RecursiveMode::NonRecursive).unwrap();

        eval_file(path.clone(), true);
        for res in rx {
          match res {
            Ok(event) => {
              if let EventKind::Access(AccessKind::Close(_)) = event.kind {
                eval_file(path.clone(), true);
              }
            }
            Err(error) => eprintln!("Error: {error:?}"),
          }
        }
      }
      false => eval_file(path, false),
    },
    None => {
      println!("Running REPL");
      repl().unwrap();
    }
  }
}
