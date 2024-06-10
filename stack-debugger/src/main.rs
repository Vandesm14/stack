use core::{fmt, num::NonZeroUsize};
use std::{
  ops::Add,
  path::PathBuf,
  sync::{mpsc, Arc},
  time::Duration,
};

use clap::Parser;
use eframe::egui::{self, text::LayoutJob, Color32, RichText, ScrollArea};
use notify::{
  Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use stack_core::prelude::*;
use stack_debugger::*;

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

  let (print_tx, print_rx) = mpsc::channel();
  let debug_tx = print_tx.clone();

  let mut engine = Engine::new().with_debug_hook(Some(Arc::new(move |s| {
    debug_tx.send(IOHookEvent::Print(s)).unwrap()
  })));
  engine.add_module(module::module(print_tx));

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
    print_rx,

    context,
    engine,
    input: cli.input.clone(),

    error: None,
    prints: Vec::new(),
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
  print_rx: mpsc::Receiver<IOHookEvent>,

  context: Context,
  engine: Engine,
  input: PathBuf,

  error: Option<String>,
  prints: Vec<IOHookEvent>,
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

    self.prints.drain(..);
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

    self.context.journal_mut().as_mut().unwrap().commit();

    self.index = self.stack_ops_len().saturating_sub(1);
    self.prints.extend(self.print_rx.try_iter().map(|evt| {
      if let IOHookEvent::GoTo(index) = evt {
        self.index = index;
      }

      evt
    }));
  }

  fn stack_ops_len(&self) -> usize {
    self.context.journal().as_ref().unwrap().all_entries().len()
  }
}

impl eframe::App for DebuggerApp {
  fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
    if self.do_reload.try_iter().last().is_some() {
      self.reload();
    }

    egui::SidePanel::left("io_hooks").show(ctx, |ui| {
      ui.heading("Contents");
      ui.add_space(10.0);

      if self.prints.is_empty() {
        let mut layout_job = LayoutJob::default();
        append_to_job(RichText::new("Use "), &mut layout_job);
        append_to_job(RichText::new("debug").code(), &mut layout_job);
        append_to_job(RichText::new(", "), &mut layout_job);
        append_to_job(RichText::new("dbg:mark").code(), &mut layout_job);
        append_to_job(RichText::new(", or "), &mut layout_job);
        append_to_job(RichText::new("dbg:note").code(), &mut layout_job);
        append_to_job(
          RichText::new(" to create jump locations"),
          &mut layout_job,
        );
        ui.label(layout_job);
      } else {
        let row_height = ui.text_style_height(&egui::TextStyle::Body);

        egui::ScrollArea::vertical().show_rows(
          ui,
          row_height,
          self.prints.len(),
          |ui, index| {
            for text in self.prints.get(index).unwrap_or_default() {
              match text {
                IOHookEvent::Print(text) => {
                  ui.label(text);
                }
                IOHookEvent::Marker(op) => {
                  if ui.link(format!("mark at {op}")).clicked() {
                    self.index = *op;
                  }
                }
                IOHookEvent::Note(op, text) => {
                  if ui
                    .link(format!("note at {op}"))
                    .on_hover_text(text)
                    .clicked()
                  {
                    self.index = *op;
                  }
                }
                IOHookEvent::GoTo(op) => {
                  if ui.link(format!("goto at {op}")).clicked() {
                    self.index = *op;
                  }
                }
              };
            }
          },
        );
      }
    });

    egui::CentralPanel::default().show(ctx, |ui| {
      ctx.set_pixels_per_point(1.2);

      if !self.prints.is_empty() {
        ui.add_space(10.0);
      }

      if let Some(err) = &self.error {
        ui.label(format!("Error: {err}"));
      }

      ui.horizontal(|ui| {
        if ui.button("Reload").clicked() {
          self.reload();
        }

        if ui.button("<|").clicked() {
          self.index = 0;
        }
        if ui.button("<").clicked() {
          self.index = self.index.saturating_sub(1);
        }

        if ui.button(">").clicked() {
          self.index = self
            .index
            .add(1)
            .min(self.stack_ops_len().saturating_sub(1));
        }
        if ui.button("|>").clicked() {
          self.index = self.stack_ops_len().saturating_sub(1);
        }
      });

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
        ui.spacing_mut().slider_width = ui.available_width() - 80.0;
        ui.add(
          egui::Slider::new(&mut self.index, 0..=max)
            .clamp_to_range(true)
            .text("ops"),
        )
      });

      let mut layout_job = LayoutJob::default();
      if let Some(entry) = entry {
        if let Some(first) = entry.ops.first() {
          if let Some(expr) = first.expr() {
            if let Some(info) = expr.info.clone() {
              if let Some((start_loc, end_loc)) = info
                .source
                .location(info.span.start)
                .zip(info.source.location(info.span.end))
              {
                const SURROUNDING_LINES: usize = 7;

                let start =
                  start_loc.line.get().saturating_sub(SURROUNDING_LINES);
                let end =
                  start_loc.line.get().saturating_add(SURROUNDING_LINES);

                ui.add_space(5.0);
                ui.label(RichText::new(info.source.name()).monospace());
                ui.add_space(5.0);

                for line in start..end {
                  if let Some(line_str) = NonZeroUsize::new(line)
                    .and_then(|line| info.source.line(line))
                  {
                    let mut text =
                      RichText::new(format!("{}: ", line)).monospace();

                    if line == start_loc.line.get() {
                      text = text.color(Color32::YELLOW);
                    }

                    append_to_job(text, &mut layout_job);

                    line_str.char_indices().for_each(|(i, c)| {
                      let mut text = RichText::new(c).monospace();
                      // TODO: properly support multiline exprs
                      if line >= start_loc.line.into()
                        && line <= end_loc.line.into()
                      {
                        if (i + 1) >= start_loc.column.into()
                          && (i + 1) < end_loc.column.into()
                        {
                          text = text
                            .color(Color32::BLACK)
                            .background_color(Color32::YELLOW);
                        } else {
                          text = text.color(Color32::YELLOW);
                        }
                      }

                      append_to_job(text, &mut layout_job);
                    });
                  }
                }
              }
            }
          }
        }
      }
      ui.label(layout_job);

      ScrollArea::vertical().show(ui, |ui| {
        ui.monospace(
          self
            .context
            .journal()
            .clone()
            .unwrap()
            .trim_to(self.index)
            .to_string(),
        );
      });
    });

    ctx.request_repaint_after(Duration::from_secs_f32(1.0 / 15.0));
  }
}
