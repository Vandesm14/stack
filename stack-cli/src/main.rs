use std::{path::PathBuf, rc::Rc};

use clap::Parser as _;
use stack::prelude::*;

fn main() {
  let cli = Cli::parse();

  match cli.subcommand {
    Subcommand::Run { path, fast } => {
      let source = match FileSource::new(path.clone()) {
        Ok(source) => Rc::new(source),
        Err(_) => {
          eprintln!("error: unable to read file '{}'", path.display());
          std::process::exit(1);
        }
      };

      let exprs = match Parser::new(Lexer::new(source)).parse() {
        Ok(exprs) => exprs,
        Err(err) => {
          eprintln!("error: {err}");
          std::process::exit(1);
        }
      };

      let engine = Engine::new().with_track_info(!fast);
      let mut context = Context::new();

      context = match engine.run(context, exprs) {
        Ok(context) => context,
        Err(err) => {
          eprintln!("error: {err}");
          std::process::exit(1);
        }
      };

      println!("stack:");

      core::iter::repeat("  ")
        .zip(context.stack())
        .for_each(|(sep, x)| println!("{sep}{x}"));
    }
  }
}

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
  #[command(subcommand)]
  subcommand: Subcommand,
}

#[derive(clap::Subcommand)]
enum Subcommand {
  /// Run a file.
  Run {
    /// The input file path.
    path: PathBuf,
    /// Whether to disable tracking extra information for debugging.
    #[arg(short, long)]
    fast: bool,
  },
}
