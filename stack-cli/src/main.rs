use std::{
  io::Read,
  path::{Path, PathBuf},
  sync::Arc,
};

use clap::Parser;
use codespan_reporting::{
  diagnostic::{Diagnostic, Label},
  files::SimpleFiles,
  term::{
    self,
    termcolor::{ColorChoice, StandardStream},
  },
};
use notify::{
  Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};
use stack_cli::{
  clear_screen, eprint_stack, ok_or_exit, print_stack, server::listen,
};
use stack_core::prelude::*;

fn main() {
  let cli = Cli::parse();

  let new_context = || {
    if cli.journal {
      Context::new().with_journal(Some(cli.journal_length.unwrap_or(20)))
    } else {
      Context::new()
    }
  };

  let mut engine =
    Engine::new().with_debug_hook(Some(Arc::new(|s| eprintln!("{s}"))));
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

      let source = Source::new("stdin", source);
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
              let source = Source::new("repl", line);
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
        let source = ok_or_exit(Source::from_path(input));
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
          let mut context = new_context();

          let source = match Source::from_path(input) {
            Ok(source) => source,
            Err(e) => {
              eprintln!("error: {e}");
              return context;
            }
          };

          context.add_source(source.clone());

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
              if let Some(journal) = context.journal() {
                eprintln!("{:#}", journal);
              }

              context
            }
            Err(e) => {
              if let Some(info) = &e.expr.info {
                let span = info.span;
                let span = span.start..span.end;

                let mut files = SimpleFiles::new();
                let mut file_id = 0;
                for (name, source) in e.context.sources() {
                  let id = files.add(name, source.source());

                  if info.source.name() == name.as_str() {
                    file_id = id;
                  }
                }

                let diagnostic = Diagnostic::error()
                  .with_message(e.clone().to_string())
                  .with_labels(vec![Label::primary(file_id, span)
                    .with_message("error occurs here")]);

                let writer = StandardStream::stderr(ColorChoice::Always);
                let config = codespan_reporting::term::Config::default();

                // TODO: Should we do anything for this error or can we just unwrap?
                let _ =
                  term::emit(&mut writer.lock(), &config, &files, &diagnostic);
              }

              eprint_stack(&e.context);
              if let Some(journal) = e.context.journal() {
                eprintln!("{}", journal);
              }

              e.context
            }
          }
        };

        ok_or_exit(clear_screen());
        let context = run_file(&input);

        ok_or_exit(context.sources().try_for_each(|source| {
          watcher
            .watch(Path::new(source.0.as_str()), RecursiveMode::NonRecursive)
        }));

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
    Subcommand::Serve => listen(),
  }
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

  /// Provide a max size for the journal
  #[arg(long, alias = "jl")]
  journal_length: Option<usize>,

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

  // TODO: add host and port as options
  Serve,
}
