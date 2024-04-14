use std::{
  io::{self, Write},
  path::PathBuf,
  rc::Rc,
};

use clap::Parser as _;
use notify::{RecommendedWatcher, Watcher as _};
use stack::{prelude::*, source::Source as _};

fn main() {
  let mut cli = Cli::parse();

  cli.enable_str = cli.enable_all || cli.enable_str;
  cli.enable_fs = cli.enable_all || cli.enable_fs;

  match cli.subcommand {
    Subcommand::Run { path, fast, watch } => {
      let mut engine = Engine::new().with_track_info(!fast);

      #[cfg(feature = "stack-std")]
      {
        if cli.enable_str {
          engine.add_module(stack_std::str::module());
        }

        if cli.enable_fs {
          engine.add_module(stack_std::fs::module(cli.sandbox));
        }
      }

      let source = match FileSource::new(path.clone()) {
        Ok(source) => Rc::new(source),
        Err(_) => {
          eprintln!("error: unable to read file '{}'", path.display());
          std::process::exit(1);
        }
      };

      match run_file_source(&engine, source.clone()) {
        Ok(context) => print_context(&context),
        Err(err) => {
          eprintln!("error: {err}");
          eprint_context(&err.context);
        }
      };

      if watch {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher =
          RecommendedWatcher::new(tx, notify::Config::default()).unwrap();

        watcher
          .watch(source.path(), notify::RecursiveMode::NonRecursive)
          .unwrap();

        for event in rx {
          match event {
            Ok(notify::Event {
              kind: notify::EventKind::Modify(_),
              ..
            }) => {
              // Clear screen and reset cursor position to the top-left.
              const ANSI: &[u8; 10] = b"\x1b[2J\x1b[1;1H";

              io::stdout().write_all(ANSI).unwrap();
              io::stderr().write_all(ANSI).unwrap();

              let source = match FileSource::new(path.clone()) {
                Ok(source) => Rc::new(source),
                Err(_) => {
                  eprintln!("error: unable to read file '{}'", path.display());
                  std::process::exit(1);
                }
              };

              match run_file_source(&engine, source) {
                Ok(context) => print_context(&context),
                Err(err) => {
                  eprintln!("error: {err}");
                  eprint_context(&err.context);
                }
              };
            }
            Ok(_) => {}
            Err(err) => {
              eprintln!("error: {err}");
              std::process::exit(1);
            }
          }
        }
      }
    }
  }
}

fn run_file_source(
  engine: &Engine,
  source: Rc<FileSource>,
) -> Result<Context, RunError> {
  let exprs = match Parser::new(Lexer::new(source)).parse() {
    Ok(exprs) => exprs,
    Err(err) => {
      eprintln!("error: {err}");
      std::process::exit(1);
    }
  };

  engine.run(Context::new(), exprs)
}

fn print_context(context: &Context) {
  println!("stack:");

  core::iter::repeat("  ")
    .zip(context.stack())
    .for_each(|(sep, x)| println!("{sep}{x}"));
}

fn eprint_context(context: &Context) {
  eprintln!("stack:");

  core::iter::repeat("  ")
    .zip(context.stack())
    .for_each(|(sep, x)| eprintln!("{sep}{x}"));
}

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
  #[command(subcommand)]
  subcommand: Subcommand,

  /// Whether to run in a sandbox variant of the enabled standard modules.
  #[arg(short, long)]
  #[cfg(feature = "stack-std")]
  sandbox: bool,

  /// Enable all standard module.
  #[arg(long)]
  #[cfg(feature = "stack-std")]
  enable_all: bool,
  /// Enable the string standard module.
  #[arg(long)]
  #[cfg(feature = "stack-std")]
  enable_str: bool,
  /// Enable the file-system standard module.
  #[arg(long)]
  #[cfg(feature = "stack-std")]
  enable_fs: bool,
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

    /// Whether to watch the file and re-run it if there are changes.
    #[arg(short, long)]
    watch: bool,
  },
}
