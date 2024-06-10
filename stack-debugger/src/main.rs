use core::fmt;
use std::{
  env,
  path::{self, PathBuf},
  sync::mpsc,
  time::Duration,
};

use clap::Parser;
use eframe::egui::{
  self, text::LayoutJob, Align, Color32, FontSelection, Hyperlink, RichText,
  Style, Visuals,
};
use notify::{
  Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use stack_core::{journal::JournalOp, prelude::*};

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
    index: 0,
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
    Box::new(move |_| Box::new(debugger_app)),
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
  index: usize,
}

impl DebuggerApp {
  fn reload(&mut self) {
    // TODO: Clear screen when we reload
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

    self.index = self.stack_ops_len().saturating_sub(1);
  }

  fn stack_ops_len(&self) -> usize {
    self.context.journal().as_ref().unwrap().all_entries().len()
  }
}

fn append_to_job(text: RichText, layout_job: &mut LayoutJob) {
  text.append_to(
    layout_job,
    &Style::default(),
    FontSelection::Default,
    Align::Center,
  )
}

const GREEN: &str = "#16C60C";
const RED: &str = "#E74856";
const BLUE: &str = "#3B78FF";
const YELLOW: &str = "#C19C00";

fn paint_expr(expr: &Expr, layout_job: &mut LayoutJob) {
  let green = Color32::from_hex(GREEN).unwrap();
  let red = Color32::from_hex(RED).unwrap();
  let blue = Color32::from_hex(BLUE).unwrap();
  let yellow = Color32::from_hex(YELLOW).unwrap();

  match &expr.kind {
    ExprKind::Nil => {
      append_to_job(RichText::new("nil").color(green), layout_job)
    }
    ExprKind::Error(x) => append_to_job(
      RichText::new(format!("error({})", x)).color(red),
      layout_job,
    ),
    ExprKind::Boolean(x) => {
      append_to_job(RichText::new(x.to_string()).color(green), layout_job)
    }
    ExprKind::Integer(x) => {
      append_to_job(RichText::new(x.to_string()).color(blue), layout_job)
    }
    ExprKind::Float(x) => {
      append_to_job(RichText::new(x.to_string()).color(blue), layout_job)
    }
    ExprKind::String(x) => {
      append_to_job(RichText::new(format!("\"{x}\"")).color(green), layout_job)
    }

    ExprKind::Symbol(x) => {
      append_to_job(RichText::new(x.to_string()).color(blue), layout_job)
    }

    ExprKind::Lazy(x) => {
      append_to_job(RichText::new("'").color(yellow), layout_job);
      paint_expr(x, layout_job)
    }
    ExprKind::List(x) => {
      append_to_job(RichText::new("("), layout_job);

      for (sep, x) in core::iter::once("")
        .chain(core::iter::repeat(" "))
        .zip(x.iter())
      {
        append_to_job(RichText::new(sep), layout_job);
        paint_expr(x, layout_job);
      }

      append_to_job(RichText::new(")"), layout_job);
    }
    ExprKind::Record(x) => {
      append_to_job(RichText::new("{"), layout_job);

      for (sep, (key, value)) in core::iter::once("")
        .chain(core::iter::repeat(", "))
        .zip(x.iter())
      {
        let key: Expr = ExprKind::Symbol(*key).into();
        append_to_job(RichText::new(sep), layout_job);
        paint_expr(&key, layout_job);
        append_to_job(RichText::new(": "), layout_job);
        paint_expr(value, layout_job);
      }

      append_to_job(RichText::new("}"), layout_job);
    }

    ExprKind::Fn(x) => {
      append_to_job(RichText::new(x.to_string()).color(yellow), layout_job)
    }
  }
}

fn string_with_quotes(expr: &Expr) -> String {
  match &expr.kind {
    ExprKind::String(x) => format!("\"{x}\""),

    ExprKind::Lazy(x) => string_with_quotes(x),

    ExprKind::List(x) => {
      let mut string = String::from("(");
      core::iter::once("")
        .chain(core::iter::repeat(" "))
        .zip(x.iter())
        .for_each(|(sep, x)| {
          string.push_str(&format!("{sep}{}", string_with_quotes(x)))
        });
      string.push(')');

      string
    }

    ExprKind::Record(x) => {
      let mut string = String::from("{");
      core::iter::once("")
        .chain(core::iter::repeat(" "))
        .zip(x.iter())
        .for_each(|(sep, (key, value))| {
          string.push_str(&format!("{sep}{key}: {}", string_with_quotes(value)))
        });
      string.push('}');

      string
    }

    kind => kind.to_string(),
  }
}

fn paint_op(op: &JournalOp, layout_job: &mut LayoutJob) {
  let green = Color32::from_hex(GREEN).unwrap();
  let red = Color32::from_hex(RED).unwrap();
  let blue = Color32::from_hex(BLUE).unwrap();
  let yellow = Color32::from_hex(YELLOW).unwrap();

  match op {
    JournalOp::Call(expr) => {
      // append_to_job(RichText::new("call(").color(yellow), layout_job);
      // paint_expr(expr, layout_job);
      // append_to_job(RichText::new(")").color(yellow), layout_job)
      append_to_job(
        RichText::new(format!("scope({})", string_with_quotes(expr)))
          .color(yellow),
        layout_job,
      )
    }
    JournalOp::FnCall(expr) => {
      // append_to_job(RichText::new("fn(").color(yellow), layout_job);
      // paint_expr(expr, layout_job);
      // append_to_job(RichText::new(")").color(yellow), layout_job)
      append_to_job(
        RichText::new(format!("fn({})", string_with_quotes(expr)))
          .color(yellow),
        layout_job,
      )
    }
    JournalOp::Push(expr) => {
      // append_to_job(RichText::new("push(").color(green), layout_job);
      // paint_expr(expr, layout_job);
      // append_to_job(RichText::new(")").color(green), layout_job)
      append_to_job(
        RichText::new(format!("push({})", string_with_quotes(expr)))
          .color(green),
        layout_job,
      )
    }
    JournalOp::Pop(expr) => {
      // append_to_job(RichText::new("pop(").color(red), layout_job);
      // paint_expr(expr, layout_job);
      // append_to_job(RichText::new(")").color(red), layout_job)
      append_to_job(
        RichText::new(format!("pop({})", string_with_quotes(expr))).color(red),
        layout_job,
      )
    }
    _ => {}
  }
}

impl eframe::App for DebuggerApp {
  fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
    if self.do_reload.try_iter().last().is_some() {
      self.reload();
    }

    egui::CentralPanel::default().show(ctx, |ui| {
      ctx.set_pixels_per_point(1.2);

      if let Some(err) = &self.error {
        ui.label(format!("Error: {err}"));
      }

      let mut entries = self.context.journal().as_ref().unwrap().all_entries();
      entries.reverse();

      let entry = entries.get(self.index);

      let mut layout_job = LayoutJob::default();
      append_to_job(
        RichText::new("Stack: ").strong().color(Color32::WHITE),
        &mut layout_job,
      );
      self
        .context
        .journal()
        .as_ref()
        .unwrap()
        .construct_to(entry.map(|entry| entry.index).unwrap_or_default())
        .iter()
        .enumerate()
        .for_each(|(i, expr)| {
          if i != 0 {
            append_to_job(RichText::new(", "), &mut layout_job);
          }
          paint_expr(expr, &mut layout_job)
        });
      ui.label(layout_job);

      let mut layout_job = LayoutJob::default();
      append_to_job(
        RichText::new("Commit: ").strong().color(Color32::WHITE),
        &mut layout_job,
      );
      if let Some(entry) = entry {
        append_to_job(
          RichText::new(format!("Scope Level {}; ", entry.scope,)),
          &mut layout_job,
        );

        core::iter::once("")
          .chain(core::iter::repeat(", "))
          .zip(entry.ops.iter())
          .for_each(|(sep, op)| {
            append_to_job(RichText::new(sep), &mut layout_job);
            paint_op(op, &mut layout_job);
          });
      }
      ui.label(layout_job);

      let mut layout_job = LayoutJob::default();
      append_to_job(
        RichText::new("Location: ").strong().color(Color32::WHITE),
        &mut layout_job,
      );
      if let Some(entry) = entry {
        if let Some(first) = entry.ops.first() {
          if let Some(expr) = first.expr() {
            if let Some(info) = expr.info.clone() {
              if let Some(location) = info.source.location(info.span.start) {
                append_to_job(
                  RichText::new(format!("{}:{}", info.source.name(), location)),
                  &mut layout_job,
                );
              }
            }
          }
        }
      }
      ui.label(layout_job);

      let max = self.stack_ops_len().saturating_sub(1);
      ui.horizontal(|ui| {
        ui.spacing_mut().slider_width = ui.available_width() - 100.0;
        ui.add(
          egui::Slider::new(&mut self.index, 0..=max)
            .clamp_to_range(true)
            .text("ops"),
        )
      });

      // TODO: Print out the entire journal like we are in `stack-cli` (nested list)
    });

    ctx.request_repaint_after(Duration::from_millis(300));
  }
}
