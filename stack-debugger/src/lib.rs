pub mod module;

use eframe::egui::{
  text::LayoutJob, Align, Color32, FontSelection, RichText, Style,
};
use itertools::Itertools;
use stack_core::{
  expr::display_fn_scope,
  journal::{Journal, JournalOp, JournalScope},
  prelude::*,
};

pub enum IOHookEvent {
  Print(String),
  Marker(usize),
  GoTo(usize),
  Note(usize, String),
}

pub fn append_to_job(text: RichText, layout_job: &mut LayoutJob) {
  text.append_to(
    layout_job,
    &Style::default(),
    FontSelection::Default,
    Align::Center,
  )
}

pub fn append_string(text: String, layout_job: &mut LayoutJob) {
  append_to_job(RichText::new(text), layout_job)
}

const GREEN: &str = "#16C60C";
const RED: &str = "#E74856";
const BLUE: &str = "#3B78FF";
const YELLOW: &str = "#C19C00";

pub fn paint_expr(expr: &Expr, layout_job: &mut LayoutJob) {
  let green = Color32::from_hex(GREEN).unwrap();
  let blue = Color32::from_hex(BLUE).unwrap();
  let yellow = Color32::from_hex(YELLOW).unwrap();

  match &expr.kind {
    ExprKind::Nil => {
      append_to_job(RichText::new("nil").color(green), layout_job)
    }
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
      append_to_job(RichText::new("["), layout_job);

      for (sep, x) in core::iter::once("")
        .chain(core::iter::repeat(" "))
        .zip(x.iter())
      {
        append_to_job(RichText::new(sep), layout_job);
        paint_expr(x, layout_job);
      }

      append_to_job(RichText::new("]"), layout_job);
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

    ExprKind::Function { scope, body } => {
      // append_to_job(RichText::new(x.to_string()).color(yellow), layout_job)
      append_to_job(RichText::new("("), layout_job);

      let sep = if body.is_empty() { "" } else { " " };
      append_to_job(
        RichText::new(format!("{}{sep}", display_fn_scope(scope))).color(blue),
        layout_job,
      );

      for (sep, x) in core::iter::once("")
        .chain(core::iter::repeat(" "))
        .zip(body.iter())
      {
        append_to_job(RichText::new(sep), layout_job);
        paint_expr(x, layout_job);
      }

      append_to_job(RichText::new(")"), layout_job);
    }

    ExprKind::SExpr { call, body } => {
      append_to_job(RichText::new("("), layout_job);

      let sep = if body.is_empty() { "" } else { " " };
      append_to_job(
        RichText::new(call.as_str().to_string()).color(blue),
        layout_job,
      );
      append_string(sep.to_owned(), layout_job);

      for (sep, x) in core::iter::once("")
        .chain(core::iter::repeat(" "))
        .zip(body.iter())
      {
        append_to_job(RichText::new(sep), layout_job);
        paint_expr(x, layout_job);
      }

      append_to_job(RichText::new(")"), layout_job);
    }
    ExprKind::Underscore => append_string("_".to_string(), layout_job),
  }
}

pub fn paint_op(op: &JournalOp, layout_job: &mut LayoutJob) {
  let green = Color32::from_hex(GREEN).unwrap();
  let red = Color32::from_hex(RED).unwrap();
  let yellow = Color32::from_hex(YELLOW).unwrap();

  match op {
    JournalOp::Call(expr) => append_to_job(
      RichText::new(format!("get({})", string_with_quotes(expr))).color(yellow),
      layout_job,
    ),
    JournalOp::SCall(expr) => append_to_job(
      RichText::new(format!("{}", string_with_quotes(expr))).color(yellow),
      layout_job,
    ),
    JournalOp::FnCall(expr) => append_to_job(
      RichText::new(format!("{}", string_with_quotes(expr))).color(yellow),
      layout_job,
    ),
    JournalOp::Push(expr) => append_to_job(
      RichText::new(format!("push({})", string_with_quotes(expr))).color(green),
      layout_job,
    ),
    JournalOp::Pop(expr) => append_to_job(
      RichText::new(format!("pop({})", string_with_quotes(expr))).color(red),
      layout_job,
    ),
    JournalOp::ScopedFnStart(_) => {
      append_to_job(RichText::new("fn start"), layout_job);
    }
    JournalOp::ScopelessFnStart => {
      append_to_job(RichText::new("fn! start"), layout_job);
    }
    JournalOp::FnEnd(..) => {
      append_to_job(RichText::new("fn end"), layout_job);
    }
    _ => {}
  }
}

pub fn paint_scope(scope: &JournalScope, layout_job: &mut LayoutJob) {
  for (key, value) in scope.iter().sorted_by_key(|(a, _)| a.as_str()) {
    append_to_job(RichText::new(format!("{}: ", key)), layout_job);
    paint_expr(value, layout_job);
    append_to_job(RichText::new("\n"), layout_job);
  }
}

pub fn paint_journal(journal: &Journal, layout_job: &mut LayoutJob) {
  let green = Color32::from_hex(GREEN).unwrap();
  let red = Color32::from_hex(RED).unwrap();
  let yellow = Color32::from_hex(YELLOW).unwrap();

  if !journal.entries().is_empty() {
    append_to_job(
      RichText::new("Stack History (most recent first):\n")
        .color(Color32::WHITE),
      layout_job,
    );
  }

  for entry in journal.entries().iter().rev().take(journal.entries().len()) {
    let bullet_symbol = match entry.scoped {
      true => format!("{}*", "  ".repeat(entry.scope_level)),
      false => {
        format!("{}!", "  ".repeat(entry.scope_level))
      }
    };

    append_to_job(
      RichText::new(format!(" {} ", bullet_symbol)).monospace(),
      layout_job,
    );

    for (i, op) in entry.ops.iter().enumerate() {
      if i != 0 {
        append_to_job(RichText::new(" ").monospace(), layout_job);
      }

      match op {
        JournalOp::Call(x) => {
          append_to_job(RichText::new(x.to_string()).monospace(), layout_job);
        }
        JournalOp::SCall(x) => {
          append_to_job(
            RichText::new(x.to_string()).color(yellow).monospace(),
            layout_job,
          );
        }
        JournalOp::FnCall(x) => {
          append_to_job(
            RichText::new(x.to_string()).color(yellow).monospace(),
            layout_job,
          );
        }
        JournalOp::Push(x) => {
          append_to_job(
            RichText::new(x.to_string()).color(green).monospace(),
            layout_job,
          );
        }
        JournalOp::Pop(x) => {
          append_to_job(
            RichText::new(x.to_string()).color(red).monospace(),
            layout_job,
          );
        }
        _ => {}
      }
    }
    append_to_job(RichText::new("\n").monospace(), layout_job);
  }
}

pub fn string_with_quotes(expr: &Expr) -> String {
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

    ExprKind::SExpr { call, body } => {
      let mut string = String::from("(");

      let sep = if body.is_empty() { "" } else { " " };
      string.push_str(format!("{}{sep}", call.as_str()).as_str());

      core::iter::once("")
        .chain(core::iter::repeat(" "))
        .zip(body.iter())
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
