use std::io::stdout;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use crossterm::terminal::{Clear, ClearType};
use crossterm::{cursor, execute};
use notify::event::AccessKind;
use notify::{
  Config, EventKind, INotifyWatcher, RecommendedWatcher, RecursiveMode, Watcher,
};

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use stack::{EvalError, Program};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
  #[command(subcommand)]
  command: Option<Commands>,

  #[arg(long)]
  no_core: bool,
}

#[derive(Subcommand)]
enum Commands {
  #[command(about = "Run a file")]
  Run {
    path: PathBuf,

    #[arg(short, long)]
    watch: bool,

    #[arg(short, long)]
    debug: bool,
  },
}

fn eval_string(program: &Program, result: Result<(), EvalError>) {
  println!();
  if let Err(err) = result {
    err.print_report();
    eprintln!("{}", program.journal);
  } else {
    println!("{}", program);
  }
}

fn repl(with_core: bool) -> rustyline::Result<()> {
  let mut rl = DefaultEditor::new()?;
  let mut program = Program::new();

  if with_core {
    program = program
      .with_core()
      // .unwrap()
      // .with_module(map::module)
      .unwrap();
  }

  loop {
    let readline = rl.readline(">> ");
    match readline {
      Ok(line) => {
        rl.add_history_entry(line.as_str()).unwrap();

        let result = program.eval_string(line.as_str());
        program.journal.commit();
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

fn eval_file(
  path: PathBuf,
  watcher: Option<&mut INotifyWatcher>,
  debug: bool,
  with_core: bool,
) {
  let mut stdout = stdout();

  let mut program = Program::new();

  if debug {
    program = program.with_debug();
  }

  if with_core {
    program = program
      .with_core()
      // .unwrap()
      // .with_module(map::module)
      .unwrap();
  }

  if watcher.is_some() {
    execute!(stdout, Clear(ClearType::All)).unwrap();
    execute!(stdout, cursor::MoveTo(0, 0)).unwrap();
  }

  let path = match path.to_str() {
    Some(str) => str,
    None => panic!("Failed to read file"),
  };

  let result = program.eval_file(path);
  program.journal.commit();
  eval_string(&program, result);

  if let Some(watcher) = watcher {
    println!();
    println!("Watching files for changes...");

    println!(" - {}", path);
    for path in program.loaded_files().filter(|p| p.ends_with(".stack")) {
      println!(" - {}", path);
      watcher
        .watch(Path::new(path), RecursiveMode::NonRecursive)
        .unwrap();
    }
  }
}

fn main() {
  let cli = Cli::parse();

  match cli.command {
    Some(Commands::Run { path, watch, debug }) => match watch {
      true => {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher =
          RecommendedWatcher::new(tx, Config::default()).unwrap();
        watcher.watch(&path, RecursiveMode::NonRecursive).unwrap();

        eval_file(path.clone(), Some(&mut watcher), debug, !cli.no_core);
        for res in rx {
          match res {
            Ok(event) => {
              if let EventKind::Access(AccessKind::Close(_)) = event.kind {
                eval_file(
                  path.clone(),
                  Some(&mut watcher),
                  debug,
                  !cli.no_core,
                );
              }
            }
            Err(error) => eprintln!("Error: {error:?}"),
          }
        }
      }
      false => eval_file(path, None, debug, !cli.no_core),
    },
    None => {
      println!("Running REPL");
      repl(!cli.no_core).unwrap();
    }
  }
}
