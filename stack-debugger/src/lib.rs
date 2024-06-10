pub mod module;

use eframe::egui::{
  text::LayoutJob, Align, Color32, FontSelection, RichText, Style,
};
use stack_core::{journal::JournalOp, prelude::*};

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

const GREEN: &str = "#16C60C";
const RED: &str = "#E74856";
const BLUE: &str = "#3B78FF";
const YELLOW: &str = "#C19C00";

pub fn paint_expr(expr: &Expr, layout_job: &mut LayoutJob) {
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

pub fn paint_op(op: &JournalOp, layout_job: &mut LayoutJob) {
  let green = Color32::from_hex(GREEN).unwrap();
  let red = Color32::from_hex(RED).unwrap();
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
    JournalOp::FnStart(scoped) => {
      append_to_job(
        RichText::new(format!(
          "scope: fn{}(start)",
          if *scoped { "" } else { "!" }
        )),
        layout_job,
      );
    }
    JournalOp::FnEnd => {
      append_to_job(RichText::new("scope: fn(end)"), layout_job);
    }
    _ => {}
  }
}
