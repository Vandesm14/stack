use std::sync::Arc;

use itertools::Itertools;
use lasso::{Spur, ThreadedRodeo};

use crate::Intrinsic;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lexer {
  pub interner: Arc<ThreadedRodeo<Spur>>,
}

impl Lexer {
  pub fn new() -> Self {
    let interner = ThreadedRodeo::new();

    interner.get_or_intern_static("_");
    interner.get_or_intern_static("+");
    interner.get_or_intern_static("*");
    interner.get_or_intern_static("/");
    interner.get_or_intern_static("=");
    interner.get_or_intern_static("%");
    interner.get_or_intern_static("!");
    interner.get_or_intern_static("&");
    interner.get_or_intern_static("|");
    interner.get_or_intern_static("<");
    interner.get_or_intern_static(">");
    interner.get_or_intern_static("?");
    interner.get_or_intern_static("$");
    interner.get_or_intern_static("-");
    interner.get_or_intern_static("~");
    interner.get_or_intern_static("^");

    interner.get_or_intern_static("nil");
    interner.get_or_intern_static("false");
    interner.get_or_intern_static("true");
    interner.get_or_intern_static("fn");

    for intrinsic in enum_iterator::all::<Intrinsic>() {
      if let Intrinsic::Syscall { arity } = intrinsic {
        if arity > 6 {
          continue;
        }
      }

      interner.get_or_intern_static(intrinsic.as_str());
    }

    Self {
      interner: interner.into(),
    }
  }

  #[inline]
  pub fn lex<'source>(&'source self, source: &'source str) -> Lex {
    Lex {
      lexer: self,
      source: source,
      index: 0,
    }
  }
}

impl Default for Lexer {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lex<'source> {
  lexer: &'source Lexer,
  source: &'source str,
  index: usize,
}

impl<'source> Iterator for Lex<'source> {
  type Item = Token;

  fn next(&mut self) -> Option<Self::Item> {
    let mut state = State::Start;
    let mut start = self.index;

    loop {
      let c = self.source.chars().nth(self.index).unwrap_or('\0');

      match state {
        State::Start => match c {
          '\0' => {
            if self.index != self.source.len() {
              let token = Token {
                kind: TokenKind::Invalid,
                span: Span {
                  start: self.index,
                  end: self.index + 1,
                },
              };

              self.index += 1;
              break Some(token);
            } else {
              break None;
            }
          }
          '0'..='9' => {
            state = State::Integer;
          }
          'a'..='z' | 'A'..='Z' | '_' => {
            state = State::Ident;
          }
          '"' => {
            state = State::String;
            start += 1;
          }
          '(' => {
            self.index += 1;

            break Some(Token {
              kind: TokenKind::ParenStart,
              span: Span {
                start,
                end: self.index,
              },
            });
          }
          ')' => {
            self.index += 1;

            break Some(Token {
              kind: TokenKind::ParenEnd,
              span: Span {
                start,
                end: self.index,
              },
            });
          }
          '{' => {
            self.index += 1;

            break Some(Token {
              kind: TokenKind::CurlyStart,
              span: Span {
                start,
                end: self.index,
              },
            });
          }
          '}' => {
            self.index += 1;

            break Some(Token {
              kind: TokenKind::CurlyEnd,
              span: Span {
                start,
                end: self.index,
              },
            });
          }
          '\'' => {
            self.index += 1;

            break Some(Token {
              kind: TokenKind::Apostrophe,
              span: Span {
                start,
                end: self.index,
              },
            });
          }
          '-' => {
            state = State::Hyphen;
          }
          '!' => {
            state = State::Exclamation;
          }
          '+' | '*' | '/' | '=' | '%' | '&' | '|' | '<' | '>' | '?' | '$'
          | '~' | '^' => {
            let mut tmp = [0u8; 4];
            let s = c.encode_utf8(&mut tmp);

            let interned = self.lexer.interner.get_or_intern(s);
            self.index += 1;

            break Some(Token {
              kind: TokenKind::Ident(interned),
              span: Span {
                start,
                end: self.index,
              },
            });
          }
          ';' => {
            state = State::Comment;
          }
          // TODO: Maybe check that these are opened and closed correctly.
          '[' | ']' => {
            start += 1;
          }
          c if c.is_whitespace() => {
            start += 1;
          }
          _ => {
            state = State::Invalid;
          }
        },
        State::Invalid => match c {
          c if c.is_whitespace() => {
            break Some(Token {
              kind: TokenKind::Invalid,
              span: Span {
                start,
                end: self.index,
              },
            })
          }
          _ => {}
        },
        State::Comment => match c {
          '\n' => {
            state = State::Start;
            start = self.index + 1;
          }
          _ => {}
        },
        State::Integer => match c {
          '0'..='9' => {}
          '.' => {
            state = State::Float;
          }
          _ => {
            let slice = &self.source[start..self.index];
            // If this panics, it's a bug.
            let parsed = match slice.parse() {
              Ok(parsed) => parsed,
              Err(err) => panic!(
                "{start}..{} {err}: {}",
                self.index,
                slice.escape_debug().join("")
              ),
            };

            break Some(Token {
              kind: TokenKind::Integer(parsed),
              span: Span {
                start,
                end: self.index,
              },
            });
          }
        },
        // TODO: Add nan, inf, and -inf for floats.
        State::Float => match c {
          '0'..='9' => {}
          _ => {
            let slice = &self.source[start..self.index];
            // If this panics, it's a bug.
            let parsed = match slice.parse() {
              Ok(parsed) => parsed,
              Err(err) => panic!(
                "{start}..{} {err}: {}",
                self.index,
                slice.escape_debug().join("")
              ),
            };

            break Some(Token {
              kind: TokenKind::Float(parsed),
              span: Span {
                start,
                end: self.index,
              },
            });
          }
        },
        State::String => match c {
          '\\' => {
            state = State::StringBackslash;
          }
          '"' => {
            let slice = &self.source[start..self.index];
            let interned = self.lexer.interner.get_or_intern(slice);

            self.index += 1;

            break Some(Token {
              kind: TokenKind::String(interned),
              span: Span {
                start,
                end: self.index,
              },
            });
          }
          _ => {}
        },
        State::StringBackslash => match c {
          'n' | 'r' | 't' | '0' => {
            state = State::String;
          }
          _ => {
            break Some(Token {
              kind: TokenKind::Invalid,
              span: Span {
                start,
                end: self.index,
              },
            })
          }
        },
        State::Ident => match c {
          'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '/' => {}
          _ => {
            let slice = &self.source[start..self.index];
            let interned = self.lexer.interner.get_or_intern(slice);

            let kind = match interned {
              interned
                if interned
                  == self.lexer.interner.get_or_intern_static("false") =>
              {
                TokenKind::Boolean(false)
              }
              interned
                if interned
                  == self.lexer.interner.get_or_intern_static("true") =>
              {
                TokenKind::Boolean(true)
              }
              interned
                if interned
                  == self.lexer.interner.get_or_intern_static("nil") =>
              {
                TokenKind::Nil
              }
              interned
                if interned
                  == self.lexer.interner.get_or_intern_static("fn") =>
              {
                TokenKind::Fn
              }
              interned => TokenKind::Ident(interned),
            };

            break Some(Token {
              kind,
              span: Span {
                start,
                end: self.index,
              },
            });
          }
        },
        State::Hyphen => match c {
          '0'..='9' => {
            state = State::Integer;
          }
          _ => {
            let interned = self.lexer.interner.get_or_intern_static("-");

            break Some(Token {
              kind: TokenKind::Ident(interned),
              span: Span {
                start,
                end: self.index,
              },
            });
          }
        },
        State::Exclamation => match c {
          '=' => {
            let interned = self.lexer.interner.get_or_intern_static("!=");
            self.index += 1;

            break Some(Token {
              kind: TokenKind::Ident(interned),
              span: Span {
                start,
                end: self.index,
              },
            });
          }
          _ => {
            let interned = self.lexer.interner.get_or_intern_static("!");

            break Some(Token {
              kind: TokenKind::Ident(interned),
              span: Span {
                start,
                end: self.index,
              },
            });
          }
        },
      }

      self.index += 1;
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Token {
  pub kind: TokenKind,
  pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum TokenKind {
  Invalid,

  Boolean(bool),
  Integer(i64),
  Float(f64),
  String(Spur),

  Ident(Spur),

  Apostrophe,

  ParenStart,
  ParenEnd,
  CurlyStart,
  CurlyEnd,

  Nil,
  Fn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
  /// Inclusive.
  pub start: usize,
  /// Exclusive.
  pub end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum State {
  Start,
  Invalid,

  Comment,

  Integer,
  Float,
  String,
  StringBackslash,

  Ident,

  Hyphen,
  Exclamation,
}

#[cfg(test)]
mod test {
  use super::*;

  use test_case::test_case;

  // TODO: Test invalid cases.

  #[test_case("false" => vec![Token { kind: TokenKind::Boolean(false), span: Span { start: 0, end: 5 } }] ; "boolean false")]
  #[test_case("true" => vec![Token { kind: TokenKind::Boolean(true), span: Span { start: 0, end: 4 } }] ; "boolean true")]
  #[test_case("0" => vec![Token { kind: TokenKind::Integer(0), span: Span { start: 0, end: 1 } }] ; "integer 0")]
  #[test_case("123" => vec![Token { kind: TokenKind::Integer(123), span: Span { start: 0, end: 3 } }] ; "integer 123")]
  #[test_case("-123" => vec![Token { kind: TokenKind::Integer(-123), span: Span { start: 0, end: 4 } }] ; "integer negative 123")]
  #[test_case("0." => vec![Token { kind: TokenKind::Float(0.0), span: Span { start: 0, end: 2 } }] ; "float 0.")]
  #[test_case("0.0" => vec![Token { kind: TokenKind::Float(0.0), span: Span { start: 0, end: 3 } }] ; "float 0.0")]
  #[test_case("-0." => vec![Token { kind: TokenKind::Float(-0.0), span: Span { start: 0, end: 3 } }] ; "float negative 0.")]
  #[test_case("-0.0" => vec![Token { kind: TokenKind::Float(-0.0), span: Span { start: 0, end: 4 } }] ; "float negative 0.0")]
  #[test_case("123.0" => vec![Token { kind: TokenKind::Float(123.0), span: Span { start: 0, end: 5 } }] ; "float 123.0")]
  #[test_case("0.123" => vec![Token { kind: TokenKind::Float(0.123), span: Span { start: 0, end: 5 } }] ; "float 0.123")]
  #[test_case("-123.0" => vec![Token { kind: TokenKind::Float(-123.0), span: Span { start: 0, end: 6 } }] ; "float negative 123.0")]
  #[test_case("-0.123" => vec![Token { kind: TokenKind::Float(-0.123), span: Span { start: 0, end: 6 } }] ; "float negative 0.123")]
  fn lex(source: &str) -> Vec<Token> {
    let lexer = Lexer::new();
    lexer.lex(source).collect()
  }

  #[test]
  fn lex_string() {
    let source = "\"Hello, World!\" \"Hey\nthere!\"";

    let lexer = Lexer::new();
    let tokens: Vec<_> = lexer.lex(source).collect();

    let expected = vec![
      Token {
        kind: TokenKind::String(
          lexer.interner.get_or_intern_static("Hello, World!"),
        ),
        span: Span { start: 1, end: 15 },
      },
      Token {
        kind: TokenKind::String(
          lexer.interner.get_or_intern_static("Hey\nthere!"),
        ),
        span: Span { start: 17, end: 28 },
      },
    ];

    for (expected, actual) in expected.into_iter().zip(tokens.into_iter()) {
      assert_eq!(expected, actual);
    }
  }

  #[test]
  fn lex_ident() {
    let source = "nil fn";

    let lexer = Lexer::new();
    let tokens: Vec<_> = lexer.lex(source).collect();

    let expected = vec![
      Token {
        kind: TokenKind::Nil,
        span: Span { start: 0, end: 3 },
      },
      Token {
        kind: TokenKind::Fn,
        span: Span { start: 4, end: 6 },
      },
    ];

    for (expected, actual) in expected.into_iter().zip(tokens.into_iter()) {
      assert_eq!(expected, actual);
    }
  }

  #[test]
  fn lex_keywords() {
    let source = "hello hello-world";

    let lexer = Lexer::new();
    let tokens: Vec<_> = lexer.lex(source).collect();

    let expected = vec![
      Token {
        kind: TokenKind::Ident(lexer.interner.get_or_intern_static("hello")),
        span: Span { start: 0, end: 5 },
      },
      Token {
        kind: TokenKind::Ident(
          lexer.interner.get_or_intern_static("hello-world"),
        ),
        span: Span { start: 6, end: 17 },
      },
    ];

    for (expected, actual) in expected.into_iter().zip(tokens.into_iter()) {
      assert_eq!(expected, actual);
    }
  }

  #[test]
  fn lex_single_symbols() {
    let source = "'()+-*/!<>=";

    let lexer = Lexer::new();
    let tokens: Vec<_> = lexer.lex(source).collect();

    let expected = vec![
      Token {
        kind: TokenKind::Apostrophe,
        span: Span { start: 0, end: 1 },
      },
      Token {
        kind: TokenKind::ParenStart,
        span: Span { start: 1, end: 2 },
      },
      Token {
        kind: TokenKind::ParenEnd,
        span: Span { start: 2, end: 3 },
      },
      Token {
        kind: TokenKind::Ident(lexer.interner.get_or_intern_static("+")),
        span: Span { start: 3, end: 4 },
      },
      Token {
        kind: TokenKind::Ident(lexer.interner.get_or_intern_static("-")),
        span: Span { start: 4, end: 5 },
      },
      Token {
        kind: TokenKind::Ident(lexer.interner.get_or_intern_static("*")),
        span: Span { start: 5, end: 6 },
      },
      Token {
        kind: TokenKind::Ident(lexer.interner.get_or_intern_static("/")),
        span: Span { start: 6, end: 7 },
      },
      Token {
        kind: TokenKind::Ident(lexer.interner.get_or_intern_static("!")),
        span: Span { start: 7, end: 8 },
      },
      Token {
        kind: TokenKind::Ident(lexer.interner.get_or_intern_static("<")),
        span: Span { start: 8, end: 9 },
      },
      Token {
        kind: TokenKind::Ident(lexer.interner.get_or_intern_static(">")),
        span: Span { start: 9, end: 10 },
      },
      Token {
        kind: TokenKind::Ident(lexer.interner.get_or_intern_static("=")),
        span: Span { start: 10, end: 11 },
      },
    ];

    for (expected, actual) in expected.into_iter().zip(tokens.into_iter()) {
      assert_eq!(expected, actual);
    }
  }

  #[test]
  fn lex_double_symbols() {
    let source = "!=";

    let lexer = Lexer::new();
    let tokens: Vec<_> = lexer.lex(source).collect();

    let expected = vec![Token {
      kind: TokenKind::Ident(lexer.interner.get_or_intern_static("!=")),
      span: Span { start: 0, end: 2 },
    }];

    for (expected, actual) in expected.into_iter().zip(tokens.into_iter()) {
      assert_eq!(expected, actual);
    }
  }
}
