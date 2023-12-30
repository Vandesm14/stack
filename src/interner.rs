use std::sync::OnceLock;

use lasso::{Spur, ThreadedRodeo};

static INTERNER: OnceLock<ThreadedRodeo<Spur>> = OnceLock::new();
static INTERNED: OnceLock<Interned> = OnceLock::new();

/// See [`interned`].
#[allow(non_snake_case)]
pub struct Interned {
  pub PLUS: Spur,
  pub MINUS: Spur,
  pub ASTERISK: Spur,
  pub SLASH: Spur,
  pub PERCENT: Spur,
  pub EQUALS: Spur,
  pub EXCLAMATION: Spur,
  pub EXCLAMATION_EQUALS: Spur,
  pub OPEN_ANGLE_BRACKET: Spur,
  pub CLOSE_ANGLE_BRACKET: Spur,
  pub NIL: Spur,
  pub FALSE: Spur,
  pub TRUE: Spur,
  pub AND: Spur,
  pub OR: Spur,
  pub FN: Spur,
}

/// Provides access to the static, thread-safe interner.
#[inline]
pub fn interner() -> &'static ThreadedRodeo<Spur> {
  INTERNER.get_or_init(|| ThreadedRodeo::new())
}

/// Provides access to a static struct that contains commonly interned slices.
#[inline]
pub fn interned() -> &'static Interned {
  INTERNED.get_or_init(|| {
    let interner = interner();

    // TODO: Add more commonly used slices.
    Interned {
      PLUS: interner.get_or_intern_static("+"),
      MINUS: interner.get_or_intern_static("-"),
      ASTERISK: interner.get_or_intern_static("*"),
      SLASH: interner.get_or_intern_static("/"),
      PERCENT: interner.get_or_intern_static("/"),
      EQUALS: interner.get_or_intern_static("="),
      EXCLAMATION: interner.get_or_intern_static("!"),
      EXCLAMATION_EQUALS: interner.get_or_intern_static("!="),
      OPEN_ANGLE_BRACKET: interner.get_or_intern_static("<"),
      CLOSE_ANGLE_BRACKET: interner.get_or_intern_static(">"),
      NIL: interner.get_or_intern_static("nil"),
      FALSE: interner.get_or_intern_static("false"),
      TRUE: interner.get_or_intern_static("true"),
      AND: interner.get_or_intern_static("and"),
      OR: interner.get_or_intern_static("or"),
      FN: interner.get_or_intern_static("fn"),
    }
  })
}
