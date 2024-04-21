use core::fmt;
use std::{
  io::{self, prelude::Write, Read},
  path::PathBuf,
  rc::Rc,
};

use clap::Parser;
use crossterm::{
  cursor::{self, MoveTo},
  style::Print,
  terminal, QueueableCommand,
};
use notify::{
  Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};
use stack::prelude::*;

fn main() {
  let cli = Cli::parse();

  let new_context = || Context::new().with_journal(cli.journal);

  let mut engine = Engine::new();
  let mut context = new_context();

  #[cfg(feature = "stack-std")]
  {
    if cli.enable_all || cli.enable_str {
      engine.add_module(stack_std::str::module());
    }

    if cli.enable_all || cli.enable_fs {
      engine.add_module(stack_std::fs::module(cli.sandbox));
    }

    if cli.enable_all || cli.enable_scope {
      engine.add_module(stack_std::scope::module());
    }
  }

  match cli.subcommand {
    Subcommand::Stdin => {
      let mut stdin = std::io::stdin();
      let mut source = String::new();

      ok_or_exit(stdin.read_to_string(&mut source));

      let source = Rc::new(Source::new("stdin", source));
      let mut lexer = Lexer::new(source);
      let exprs = ok_or_exit(parse(&mut lexer));

      context = ok_or_exit(engine.run(context, exprs));
      print_stack(&context);
    }
    Subcommand::Repl => {
      let mut repl = Reedline::create();
      let prompt = DefaultPrompt::new(
        DefaultPromptSegment::Empty,
        DefaultPromptSegment::Empty,
      );

      loop {
        let signal = ok_or_exit(repl.read_line(&prompt));

        match signal {
          Signal::CtrlC | Signal::CtrlD => {
            println!("aborted");
            break;
          }
          Signal::Success(line) => {
            if line.starts_with(':') {
              match &line.as_str()[1..] {
                "exit" => break,
                "clear" => {
                  ok_or_exit(repl.clear_screen());
                }
                "reset" => {
                  context = new_context();
                  println!("Reset context");
                }
                command => eprintln!("error: unknown command '{command}'"),
              }
            } else {
              let source = Rc::new(Source::new("repl", line));
              let mut lexer = Lexer::new(source);
              let exprs = ok_or_exit(parse(&mut lexer));

              context = match engine.run(context, exprs) {
                Ok(context) => {
                  print_stack(&context);
                  context
                }
                Err(e) => {
                  eprintln!("error: {e}");
                  eprint_stack(&e.context);
                  e.context
                }
              }
            }
          }
        }
      }
    }
    Subcommand::Run { input, watch } => {
      if !watch {
        let source = Rc::new(ok_or_exit(Source::from_path(input)));
        let mut lexer = Lexer::new(source);
        let exprs = ok_or_exit(parse(&mut lexer));

        context = ok_or_exit(engine.run(context, exprs));
        print_stack(&context);
      } else {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher =
          ok_or_exit(RecommendedWatcher::new(tx, Config::default()));
        ok_or_exit(watcher.watch(&input, RecursiveMode::NonRecursive));

        let run_file = |input| {
          let context = new_context();

          let source = match Source::from_path(input).map(Rc::new) {
            Ok(source) => source,
            Err(e) => {
              eprintln!("error: {e}");
              return context;
            }
          };

          let mut lexer = Lexer::new(source);

          let exprs = match parse(&mut lexer) {
            Ok(exprs) => exprs,
            Err(e) => {
              eprintln!("error: {e}");
              return context;
            }
          };

          match engine.run(context, exprs) {
            Ok(context) => {
              print_stack(&context);
              context
            }
            Err(e) => {
              eprintln!("error: {e}");
              eprint_stack(&e.context);
              e.context
            }
          }
        };

        ok_or_exit(clear_screen());
        run_file(&input);

        for event in rx {
          if let Event {
            kind: EventKind::Modify(_),
            ..
          } = ok_or_exit(event)
          {
            ok_or_exit(clear_screen());
            run_file(&input);
          }
        }
      }
    }
  }
}

fn ok_or_exit<T, E>(result: Result<T, E>) -> T
where
  E: fmt::Display,
{
  match result {
    Ok(x) => x,
    Err(e) => {
      eprintln!("error: {e}");
      std::process::exit(1);
    }
  }
}

fn print_stack(context: &Context) {
  print!("stack:");

  core::iter::repeat(" ")
    .zip(context.stack())
    .for_each(|(sep, x)| print!("{sep}{x:#}"));

  println!()
}

fn eprint_stack(context: &Context) {
  eprint!("stack:");

  core::iter::repeat(" ")
    .zip(context.stack())
    .for_each(|(sep, x)| eprint!("{sep}{x:#}"));

  eprintln!()
}

fn clear_screen() -> io::Result<()> {
  let mut stdout = std::io::stdout();

  stdout.queue(cursor::Hide)?;
  let (_, num_lines) = terminal::size()?;
  for _ in 0..2 * num_lines {
    stdout.queue(Print("\n"))?;
  }
  stdout.queue(MoveTo(0, 0))?;
  stdout.queue(cursor::Show)?;

  stdout.flush()
}

#[derive(Debug, Clone, PartialEq, Eq, Default, clap::Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
  #[command(subcommand)]
  subcommand: Subcommand,

  /// Whether to enable stack journaling.
  #[arg(short, long)]
  journal: bool,

  /// Whether to run a sandbox variant of the enabled standard modules.
  #[arg(short, long)]
  #[cfg(feature = "stack-std")]
  sandbox: bool,

  /// Enable all standard modules.
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
  /// Enable the scope standard module.
  #[arg(long)]
  #[cfg(feature = "stack-std")]
  enable_scope: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, clap::Subcommand)]
enum Subcommand {
  /// Runs a REPL [alias >].
  #[default]
  #[command(alias = ">")]
  Repl,
  /// Runs the code supplied via STDIN [alias -].
  #[command(alias = "-")]
  Stdin,
  /// Runs the code from an input file path.
  Run {
    /// The input file path.
    input: PathBuf,

    /// Whether to watch the file and re-run it if there are changes.
    #[arg(short, long)]
    watch: bool,
  },
}

////////////////////////////////////////////////////////////////////////////////

// use std::{
//   io::{self, Write},
//   path::{Path, PathBuf},
//   rc::Rc,
// };

// use clap::Parser as _;
// use notify::{RecommendedWatcher, Watcher as _};
// use stack::prelude::*;

// fn main() {
//   let mut cli = Cli::parse();

//   cli.enable_str = cli.enable_all || cli.enable_str;
//   cli.enable_fs = cli.enable_all || cli.enable_fs;
//   cli.enable_scope = cli.enable_all || cli.enable_scope;

//   match cli.subcommand {
//     Subcommand::Run { path, watch } => {
//       // Clear screen and reset cursor position to the top-left.
//       const ANSI: &[u8; 10] = b"\x1b[2J\x1b[1;1H";

//       io::stdout().write_all(ANSI).unwrap();
//       io::stderr().write_all(ANSI).unwrap();

//       let mut engine = Engine::new();

//       #[cfg(feature = "stack-std")]
//       {
//         if cli.enable_str {
//           engine.add_module(stack_std::str::module());
//         }

//         if cli.enable_fs {
//           engine.add_module(stack_std::fs::module(cli.sandbox));
//         }

//         if cli.enable_scope {
//           engine.add_module(stack_std::scope::module());
//         }
//       }

//       let source = match Source::from_path(path.clone()) {
//         Ok(source) => Rc::new(source),
//         Err(_) => {
//           eprintln!("error: unable to read file '{}'", path.display());
//           std::process::exit(1);
//         }
//       };

//       match run_file_source(&engine, source.clone(), cli.journal) {
//         Ok(context) => print_context(&context),
//         Err(err) => {
//           eprintln!("error: {err}");
//           eprint_context(&err.context);
//         }
//       };

//       if watch {
//         let (tx, rx) = std::sync::mpsc::channel();

//         let mut watcher =
//           RecommendedWatcher::new(tx, notify::Config::default()).unwrap();

//         watcher
//           .watch(
//             Path::new(source.name()),
//             notify::RecursiveMode::NonRecursive,
//           )
//           .unwrap();

//         for event in rx {
//           match event {
//             Ok(notify::Event {
//               kind: notify::EventKind::Modify(_),
//               ..
//             }) => {
//               io::stdout().write_all(ANSI).unwrap();
//               io::stderr().write_all(ANSI).unwrap();

//               let source = match Source::from_path(path.clone()) {
//                 Ok(source) => Rc::new(source),
//                 Err(_) => {
//                   eprintln!("error: unable to read file '{}'", path.display());
//                   std::process::exit(1);
//                 }
//               };

//               match run_file_source(&engine, source, cli.journal) {
//                 Ok(context) => print_context(&context),
//                 Err(err) => {
//                   eprintln!("error: {err}");
//                   eprint_context(&err.context);
//                 }
//               };
//             }
//             Ok(_) => {}
//             Err(err) => {
//               eprintln!("error: {err}");
//               std::process::exit(1);
//             }
//           }
//         }
//       }
//     }
//   }
// }

// fn run_file_source(
//   engine: &Engine,
//   source: Rc<Source>,
//   journal: bool,
// ) -> Result<Context, RunError> {
//   let mut lexer = Lexer::new(source);
//   let exprs = match stack::parser::parse(&mut lexer) {
//     Ok(exprs) => exprs,
//     Err(err) => {
//       eprintln!("error: {err}");
//       std::process::exit(1);
//     }
//   };

//   let context = Context::new();
//   let context = if journal {
//     context.add_journal()
//   } else {
//     context
//   };

//   engine.run(context, exprs)
// }

// fn print_context(context: &Context) {
//   println!("{}", context);
// }

// fn eprint_context(context: &Context) {
//   eprintln!("{}", context);
// }

// #[derive(clap::Parser)]
// #[command(author, version, about, long_about = None)]
// struct Cli {
//   #[command(subcommand)]
//   subcommand: Subcommand,

//   /// Whether to run a sandbox variant of the enabled standard modules.
//   #[arg(short, long)]
//   #[cfg(feature = "stack-std")]
//   sandbox: bool,

//   /// Whether to enable stack journaling.
//   #[arg(short, long)]
//   journal: bool,

//   /// Enable all standard modules.
//   #[arg(long)]
//   #[cfg(feature = "stack-std")]
//   enable_all: bool,
//   /// Enable the string standard module.
//   #[arg(long)]
//   #[cfg(feature = "stack-std")]
//   enable_str: bool,
//   /// Enable the file-system standard module.
//   #[arg(long)]
//   #[cfg(feature = "stack-std")]
//   enable_fs: bool,
//   /// Enable the scope standard module.
//   #[arg(long)]
//   #[cfg(feature = "stack-std")]
//   enable_scope: bool,
// }

// #[derive(clap::Subcommand)]
// enum Subcommand {
//   /// Run a file.
//   Run {
//     /// The input file path.
//     path: PathBuf,

//     /// Whether to watch the file and re-run it if there are changes.
//     #[arg(short, long)]
//     watch: bool,
//   },
// }
