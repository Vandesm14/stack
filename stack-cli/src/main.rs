use core::fmt;
use std::{io::Read, rc::Rc};

use clap::Parser;
use stack::prelude::*;

fn main() {
  let cli = Cli::parse();

  let mut engine = Engine::new();

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
    Subcommand::Repl => todo!(),
    Subcommand::Stdin => {
      let mut stdin = std::io::stdin();
      let mut source = String::new();

      ok_or_exit(stdin.read_to_string(&mut source));

      let source = Rc::new(Source::new("stdin", source));
      let mut lexer = Lexer::new(source);
      let exprs = ok_or_exit(parse(&mut lexer));

      let mut context = Context::new().with_journal(cli.journal);
      context = ok_or_exit(engine.run(context, exprs));

      display_stack(&context);
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

fn display_stack(context: &Context) {
  println!("stack:");

  core::iter::repeat(" ")
    .zip(context.stack())
    .for_each(|(sep, x)| print!("{sep}{x:#}"));

  println!();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::Parser)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::Subcommand)]
enum Subcommand {
  /// Runs a REPL [alias >].
  #[default]
  #[command(alias = ">")]
  Repl,
  /// Runs the code supplied via STDIN [alias -].
  #[command(alias = "-")]
  Stdin,
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
