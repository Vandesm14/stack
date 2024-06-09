use core::fmt;
use std::{
  io::{self, prelude::Write, Read},
  path::{Path, PathBuf},
  sync::mpsc,
  time::Duration,
};

use clap::Parser;
use crossterm::{
  cursor::{self, MoveTo},
  style::Print,
  terminal, QueueableCommand,
};
use eframe::egui::{
  self, text::LayoutJob, Align, Color32, FontSelection, RichText, Style,
};
use notify::{
  Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};
use stack_core::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Default, clap::Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
  /// The input file path.
  input: PathBuf,

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

pub fn main() {
  let cli = Cli::parse();

  let context = Context::new().with_journal(None);
  let mut engine = Engine::new().with_debug_hook(Some(|s| eprintln!("{s}")));

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

  let (tx, rx) = mpsc::channel();

  let mut debugger_app = DebuggerApp {
    do_reload: rx,
    context,
    engine,
    input: cli.input.clone(),
    error: None,
  };

  // Run the program once in the beginning
  debugger_app.reload();

  std::thread::spawn(move || {
    let (watcher_tx, watcher_rx) = mpsc::channel();

    let mut watcher =
      ok_or_exit(RecommendedWatcher::new(watcher_tx, Config::default()));
    ok_or_exit(watcher.watch(&cli.input, RecursiveMode::NonRecursive));

    for event in watcher_rx {
      if let Event {
        kind: EventKind::Modify(_),
        ..
      } = ok_or_exit(event)
      {
        tx.send(()).unwrap();
      }
    }
  });

  let native_options = eframe::NativeOptions::default();
  eframe::run_native(
    "Stack Debugger",
    native_options,
    Box::new(move |cc| Box::new(debugger_app)),
  )
  .unwrap();
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

pub struct DebuggerApp {
  do_reload: mpsc::Receiver<()>,
  context: Context,
  engine: Engine,
  input: PathBuf,

  error: Option<String>,
}

impl DebuggerApp {
  fn reload(&mut self) {
    let mut context = Context::new().with_journal(None);

    let source = match Source::from_path(&self.input) {
      Ok(source) => source,
      Err(e) => return self.error = Some(e.to_string()),
    };

    context.add_source(source.clone());

    let mut lexer = Lexer::new(source);

    let exprs = match parse(&mut lexer) {
      Ok(exprs) => exprs,
      Err(e) => return self.error = Some(e.to_string()),
    };

    match self.engine.run(context, exprs) {
      Ok(context) => {
        self.context = context;
        self.error = None
      }
      Err(err) => {
        self.error = Some(err.to_string().clone());
        self.context = err.context;
      }
    }
  }
}

impl eframe::App for DebuggerApp {
  fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
    if self.do_reload.try_iter().last().is_some() {
      self.reload();
    }

    egui::CentralPanel::default().show(ctx, |ui| {
      if let Some(err) = &self.error {
        ui.label(format!("Error: {err}"));
      }

      ui.label(format!(
        "Stack: {}",
        self.context.stack().iter().enumerate().fold(
          String::new(),
          |mut str, (i, expr)| {
            if i == 0 {
              str.push_str(&format!("{}", expr));
            } else {
              str.push_str(&format!(", {}", expr));
            }

            str
          },
        )
      ));
    });

    ctx.request_repaint_after(Duration::from_millis(300));
  }
}
