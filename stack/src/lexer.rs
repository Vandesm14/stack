use core::fmt;
use std::rc::Rc;

use crate::source::Source;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token {
  pub kind: TokenKind,
  pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
  /// The lower byte bound (inclusive).
  pub start: usize,
  /// The upper byte bound (exclusive).
  pub end: usize,
}

impl fmt::Display for Span {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}..{}", self.start, self.end)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
  Invalid,
  Eof,

  Integer,
  Float,
  String,
  Symbol,

  Plus,
  Minus,
  Asterisk,
  Slash,
  Percent,
  Apostrophe,
  LeftParen,
  RightParen,
  LeftSquare,
  RightSquare,

  Fn,
  FnExclamation,
}

impl fmt::Display for TokenKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Invalid => write!(f, "invalid characters"),
      Self::Eof => write!(f, "end of file"),

      Self::Integer => write!(f, "an integer literal"),
      Self::Float => write!(f, "a float literal"),
      Self::String => write!(f, "a string literal"),
      Self::Symbol => write!(f, "a symbol"),

      Self::Plus => write!(f, "+"),
      Self::Minus => write!(f, "-"),
      Self::Asterisk => write!(f, "*"),
      Self::Slash => write!(f, "/"),
      Self::Percent => write!(f, "%"),
      Self::Apostrophe => write!(f, "'"),
      Self::LeftParen => write!(f, "("),
      Self::RightParen => write!(f, ")"),
      Self::LeftSquare => write!(f, "["),
      Self::RightSquare => write!(f, "]"),

      Self::Fn => write!(f, "fn"),
      Self::FnExclamation => write!(f, "fn!"),
    }
  }
}

/// Converts a [`Source`] into a stream of [`Token`]s.
///
/// This **does not** implement the [`Copy`] trait, since each *instance* keeps
/// track of its own cursor.
#[derive(Clone)]
pub struct Lexer {
  source: Rc<dyn Source>,
  cursor: usize,
}

impl Lexer {
  /// Creates a [`Lexer`] from a [`Source`].
  #[inline]
  pub fn new(source: Rc<dyn Source>) -> Self {
    Self {
      // Skip the UTF-8 BOM, if present.
      #[allow(clippy::obfuscated_if_else)]
      cursor: source
        .content()
        .as_bytes()
        .starts_with(b"\xef\xbb\xbf")
        .then_some(3)
        .unwrap_or(0),
      source,
    }
  }

  #[inline]
  pub fn source(&self) -> Rc<dyn Source> {
    self.source.clone()
  }

  /// Returns the next [`Token`] in the stream.
  ///
  /// Once the first [`TokenKind::Eof`] has been returned, it will continue to
  /// return them thereafter, akin to a [`FusedIterator`].
  ///
  /// [`FusedIterator`]: core::iter::FusedIterator
  #[allow(clippy::should_implement_trait)]
  pub fn next(&mut self) -> Token {
    let source = self.source.content();

    let mut state = State::Start;
    let mut chars = source[self.cursor..].chars();
    let mut start = self.cursor;

    loop {
      let c = chars.next().unwrap_or('\0');
      let c_len = c.len_utf8();

      match state {
        State::Start => match c {
          '\0' if self.cursor == source.len() => {
            break Token {
              kind: TokenKind::Eof,
              span: Span {
                start: self.cursor,
                end: self.cursor,
              },
            };
          }
          '+' => {
            self.cursor += c_len;

            break Token {
              kind: TokenKind::Plus,
              span: Span {
                start,
                end: self.cursor,
              },
            };
          }
          '*' => {
            self.cursor += c_len;

            break Token {
              kind: TokenKind::Asterisk,
              span: Span {
                start,
                end: self.cursor,
              },
            };
          }
          '/' => {
            self.cursor += c_len;

            break Token {
              kind: TokenKind::Slash,
              span: Span {
                start,
                end: self.cursor,
              },
            };
          }
          '%' => {
            self.cursor += c_len;

            break Token {
              kind: TokenKind::Percent,
              span: Span {
                start,
                end: self.cursor,
              },
            };
          }
          '\'' => {
            self.cursor += c_len;

            break Token {
              kind: TokenKind::Apostrophe,
              span: Span {
                start,
                end: self.cursor,
              },
            };
          }
          '(' => {
            self.cursor += c_len;

            break Token {
              kind: TokenKind::LeftParen,
              span: Span {
                start,
                end: self.cursor,
              },
            };
          }
          ')' => {
            self.cursor += c_len;

            break Token {
              kind: TokenKind::RightParen,
              span: Span {
                start,
                end: self.cursor,
              },
            };
          }
          '[' => {
            self.cursor += c_len;

            break Token {
              kind: TokenKind::LeftSquare,
              span: Span {
                start,
                end: self.cursor,
              },
            };
          }
          ']' => {
            self.cursor += c_len;

            break Token {
              kind: TokenKind::RightSquare,
              span: Span {
                start,
                end: self.cursor,
              },
            };
          }
          ';' => state = State::Comment,
          '-' => state = State::Minus,
          '0'..='9' => state = State::Integer,
          // NOTE: If this is modified, remember to change the other instances
          //       in the other State matches.
          '_'
          | '\\'
          | ':'
          | '!'
          | '='
          | '<'
          | '>'
          | '?'
          | 'a'..='z'
          | 'A'..='Z' => state = State::Symbol,
          ' ' | '\n' | '\t' | '\r' => start = self.cursor + c_len,
          _ => state = State::Invalid,
        },
        State::Invalid => match c {
          '\0' | ' ' | '\n' | '\t' | '\r' | '(' | ')' | '[' | ']' | '"' => {
            break Token {
              kind: TokenKind::Invalid,
              span: Span {
                start,
                end: self.cursor,
              },
            };
          }
          _ => {}
        },
        State::Comment => match c {
          '\0' => {
            state = State::Start;
            self.cursor -= c_len;
          }
          '\n' => state = State::Start,
          _ => {}
        },
        State::Minus => match c {
          '0'..='9' => state = State::Integer,
          '_'
          | '+'
          | '-'
          | '*'
          | '\\'
          | '%'
          | ':'
          | '!'
          | '='
          | '<'
          | '>'
          | '?'
          | 'a'..='z'
          | 'A'..='Z' => state = State::Symbol,
          _ => {
            break Token {
              kind: TokenKind::Minus,
              span: Span {
                start,
                end: self.cursor,
              },
            };
          }
        },
        State::Integer => match c {
          '0'..='9' => {}
          '.' => state = State::Float,
          _ => {
            break Token {
              kind: TokenKind::Integer,
              span: Span {
                start,
                end: self.cursor,
              },
            };
          }
        },
        State::Float => match c {
          '0'..='9' => {}
          _ => {
            break Token {
              kind: TokenKind::Float,
              span: Span {
                start,
                end: self.cursor,
              },
            };
          }
        },
        State::Symbol => match c {
          '_'
          | '+'
          | '-'
          | '*'
          | '\\'
          | '%'
          | ':'
          | '!'
          | '='
          | '<'
          | '>'
          | '?'
          | 'a'..='z'
          | 'A'..='Z'
          | '0'..='9' => {}
          _ => {
            let slice = &source[start..self.cursor];
            let kind = match slice {
              "fn" => TokenKind::Fn,
              "fn!" => TokenKind::FnExclamation,
              _ => TokenKind::Symbol,
            };

            break Token {
              kind,
              span: Span {
                start,
                end: self.cursor,
              },
            };
          }
        },
      }

      self.cursor += c_len;
    }
  }
}

enum State {
  Start,
  Invalid,
  Comment,
  Minus,
  Integer,
  Float,
  Symbol,
}

#[cfg(test)]
mod test {
  use crate::source::test::TestSource;

  use super::*;
  use test_case::case;

  #[case("" => vec![Token { kind: TokenKind::Eof, span: Span { start: 0, end: 0 } }] ; "empty")]
  #[case(" \t\r\n" => vec![Token { kind: TokenKind::Eof, span: Span { start: 4, end: 4 } }] ; "whitespace only")]
  #[case("; Comment" => vec![Token { kind: TokenKind::Eof, span: Span { start: 9, end: 9 } }] ; "comment")]
  #[case("; Comment\n" => vec![Token { kind: TokenKind::Eof, span: Span { start: 10, end: 10 } }] ; "comment whitespace")]
  #[case("+" => vec![Token { kind: TokenKind::Plus, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "plus only")]
  #[case("+\n" => vec![Token { kind: TokenKind::Plus, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 2, end: 2 } }] ; "plus whitespace only")]
  #[case("-" => vec![Token { kind: TokenKind::Minus, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "minus only")]
  #[case("-\n" => vec![Token { kind: TokenKind::Minus, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 2, end: 2 } }] ; "minus whitespace only")]
  #[case("*" => vec![Token { kind: TokenKind::Asterisk, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "asterisk only")]
  #[case("*\n" => vec![Token { kind: TokenKind::Asterisk, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 2, end: 2 } }] ; "asterisk whitespace only")]
  #[case("/" => vec![Token { kind: TokenKind::Slash, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "slash only")]
  #[case("/\n" => vec![Token { kind: TokenKind::Slash, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 2, end: 2 } }] ; "slash whitespace only")]
  #[case("%" => vec![Token { kind: TokenKind::Percent, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "percent only")]
  #[case("%\n" => vec![Token { kind: TokenKind::Percent, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 2, end: 2 } }] ; "percent whitespace only")]
  #[case("'" => vec![Token { kind: TokenKind::Apostrophe, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "apostrophe")]
  #[case("'\n" => vec![Token { kind: TokenKind::Apostrophe, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 2, end: 2 } }] ; "apostrophe whitespace")]
  #[case("(" => vec![Token { kind: TokenKind::LeftParen, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "left paren")]
  #[case("(\n" => vec![Token { kind: TokenKind::LeftParen, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 2, end: 2 } }] ; "left paren whitespace")]
  #[case(")" => vec![Token { kind: TokenKind::RightParen, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "right paren")]
  #[case(")\n" => vec![Token { kind: TokenKind::RightParen, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 2, end: 2 } }] ; "right paren whitespace")]
  #[case("[" => vec![Token { kind: TokenKind::LeftSquare, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "left square")]
  #[case("[\n" => vec![Token { kind: TokenKind::LeftSquare, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 2, end: 2 } }] ; "left square whitespace")]
  #[case("]" => vec![Token { kind: TokenKind::RightSquare, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "right square")]
  #[case("]\n" => vec![Token { kind: TokenKind::RightSquare, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 2, end: 2 } }] ; "right square whitespace")]
  #[case("123" => vec![Token { kind: TokenKind::Integer, span: Span { start: 0, end: 3 } }, Token { kind: TokenKind::Eof, span: Span { start: 3, end: 3 } }] ; "integer")]
  #[case("123\n" => vec![Token { kind: TokenKind::Integer, span: Span { start: 0, end: 3 } }, Token { kind: TokenKind::Eof, span: Span { start: 4, end: 4 } }] ; "integer whitespace")]
  #[case("-123" => vec![Token { kind: TokenKind::Integer, span: Span { start: 0, end: 4 } }, Token { kind: TokenKind::Eof, span: Span { start: 4, end: 4 } }] ; "negative integer")]
  #[case("-123\n" => vec![Token { kind: TokenKind::Integer, span: Span { start: 0, end: 4 } }, Token { kind: TokenKind::Eof, span: Span { start: 5, end: 5 } }] ; "negative integer whitespace")]
  #[case("1.2" => vec![Token { kind: TokenKind::Float, span: Span { start: 0, end: 3 } }, Token { kind: TokenKind::Eof, span: Span { start: 3, end: 3 } }] ; "float")]
  #[case("1.2\n" => vec![Token { kind: TokenKind::Float, span: Span { start: 0, end: 3 } }, Token { kind: TokenKind::Eof, span: Span { start: 4, end: 4 } }] ; "float whitespace")]
  #[case("-1.2" => vec![Token { kind: TokenKind::Float, span: Span { start: 0, end: 4 } }, Token { kind: TokenKind::Eof, span: Span { start: 4, end: 4 } }] ; "negative float")]
  #[case("-1.2\n" => vec![Token { kind: TokenKind::Float, span: Span { start: 0, end: 4 } }, Token { kind: TokenKind::Eof, span: Span { start: 5, end: 5 } }] ; "negative float whitespace")]
  #[case("hello" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 5 } }, Token { kind: TokenKind::Eof, span: Span { start: 5, end: 5 } }] ; "symbol")]
  #[case("hello\n" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 5 } }, Token { kind: TokenKind::Eof, span: Span { start: 6, end: 6 } }] ; "symbol whitespace")]
  #[case("h3l10" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 5 } }, Token { kind: TokenKind::Eof, span: Span { start: 5, end: 5 } }] ; "alphanumeric symbol")]
  #[case("h3l10\n" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 5 } }, Token { kind: TokenKind::Eof, span: Span { start: 6, end: 6 } }] ; "alphanumeric symbol whitespace")]
  #[case("he_lo" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 5 } }, Token { kind: TokenKind::Eof, span: Span { start: 5, end: 5 } }] ; "underscore symbol")]
  #[case("he_lo\n" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 5 } }, Token { kind: TokenKind::Eof, span: Span { start: 6, end: 6 } }] ; "underscore symbol whitespace")]
  #[case("he-lo" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 5 } }, Token { kind: TokenKind::Eof, span: Span { start: 5, end: 5 } }] ; "hypen symbol")]
  #[case("he-lo\n" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 5 } }, Token { kind: TokenKind::Eof, span: Span { start: 6, end: 6 } }] ; "hypen symbol whitespace")]
  #[case("_" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "underscore symbol only")]
  #[case("_\n" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 2, end: 2 } }] ; "underscore symbol whitespace only")]
  #[case(":" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "colon symbol only")]
  #[case(":\n" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 2, end: 2 } }] ; "colon symbol whitespace only")]
  #[case("!" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "exclamation symbol only")]
  #[case("!\n" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 2, end: 2 } }] ; "exclamation symbol whitespace only")]
  #[case("=" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "equals symbol only")]
  #[case("=\n" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 2, end: 2 } }] ; "equals symbol whitespace only")]
  #[case("<" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "left angle symbol only")]
  #[case("<\n" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 2, end: 2 } }] ; "left angle symbol whitespace only")]
  #[case(">" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "right angle symbol only")]
  #[case(">\n" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 2, end: 2 } }] ; "right angle symbol whitespace only")]
  #[case("?" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "question symbol only")]
  #[case("?\n" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 2, end: 2 } }] ; "question symbol whitespace only")]
  #[case("nil" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 3 } }, Token { kind: TokenKind::Eof, span: Span { start: 3, end: 3 } }] ; "nil")]
  #[case("nil\n" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 3 } }, Token { kind: TokenKind::Eof, span: Span { start: 4, end: 4 } }] ; "nil whitespace")]
  #[case("fn" => vec![Token { kind: TokenKind::Fn, span: Span { start: 0, end: 2 } }, Token { kind: TokenKind::Eof, span: Span { start: 2, end: 2 } }] ; "fn_")]
  #[case("fn\n" => vec![Token { kind: TokenKind::Fn, span: Span { start: 0, end: 2 } }, Token { kind: TokenKind::Eof, span: Span { start: 3, end: 3 } }] ; "fn whitespace")]
  #[case("fn!" => vec![Token { kind: TokenKind::FnExclamation, span: Span { start: 0, end: 3 } }, Token { kind: TokenKind::Eof, span: Span { start: 3, end: 3 } }] ; "fn exclamation")]
  #[case("fn!\n" => vec![Token { kind: TokenKind::FnExclamation, span: Span { start: 0, end: 3 } }, Token { kind: TokenKind::Eof, span: Span { start: 4, end: 4 } }] ; "fn exclamation whitespace")]
  fn lexer(source: &str) -> Vec<Token> {
    let source = Rc::new(TestSource::new(source));
    let mut lexer = Lexer::new(source);

    let mut tokens = Vec::new();

    loop {
      let token = lexer.next();
      tokens.push(token);

      if token.kind == TokenKind::Eof {
        break tokens;
      }
    }
  }
}
