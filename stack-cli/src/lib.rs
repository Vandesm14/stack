use core::fmt;
use std::io::{self, prelude::Write};

use crossterm::{
  cursor::{self, MoveTo},
  style::Print,
  terminal, QueueableCommand,
};
use stack_core::prelude::*;

pub mod server;

pub fn ok_or_exit<T, E>(result: Result<T, E>) -> T
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

pub fn print_stack(context: &Context) {
  print!("stack:");

  core::iter::repeat(" ")
    .zip(context.stack())
    .for_each(|(sep, x)| print!("{sep}{x:#}"));

  println!()
}

pub fn eprint_stack(context: &Context) {
  eprint!("stack:");

  core::iter::repeat(" ")
    .zip(context.stack())
    .for_each(|(sep, x)| eprint!("{sep}{x:#}"));

  eprintln!()
}

pub fn clear_screen() -> io::Result<()> {
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
