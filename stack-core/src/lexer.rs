use core::{fmt, ops::Range};

use crate::source::Source;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
  pub kind: TokenKind,
  pub span: Span,
}

impl fmt::Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.kind)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
  /// The lower byte bound (inclusive).
  pub start: usize,
  /// The upper byte bound (exclusive).
  pub end: usize,
}

impl Span {
  /// Returns the <code>[Range]\<[usize]\></code> of this [`Span`].
  #[inline]
  pub const fn to_range(self) -> Range<usize> {
    Range {
      start: self.start,
      end: self.end,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
  Invalid,
  Eof,

  Apostrophe,
  LeftParen,
  RightParen,
  // LeftSquare,
  // RightSquare,
  Integer,
  Float,
  String,
  Symbol,
}

impl fmt::Display for TokenKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Invalid => write!(f, "invalid characters"),
      Self::Eof => write!(f, "end of file"),

      Self::Apostrophe => write!(f, "'"),
      Self::LeftParen => write!(f, "("),
      Self::RightParen => write!(f, ")"),
      // Self::LeftSquare => write!(f, "["),
      // Self::RightSquare => write!(f, "]"),
      Self::Integer => write!(f, "an integer literal"),
      Self::Float => write!(f, "a float literal"),
      Self::String => write!(f, "a string literal"),
      Self::Symbol => write!(f, "a symbol literal"),
    }
  }
}

/// Converts a <code>&[str]</code> into a stream of [`Token`]s.
#[derive(Debug)]
pub struct Lexer {
  source: Source,
  cursor: usize,
  peeked: Option<Token>,
}

impl Lexer {
  /// Creates a [`Lexer`] from a [`Source`].
  pub fn new(source: Source) -> Self {
    Self {
      // Skip the UTF-8 BOM, if present.
      #[allow(clippy::obfuscated_if_else)]
      cursor: source
        .source()
        .as_bytes()
        .starts_with(b"\xef\xbb\xbf")
        .then_some(3)
        .unwrap_or(0),
      source,
      peeked: None,
    }
  }

  /// Returns a clone of the [`Source`].
  #[inline]
  pub fn source(&self) -> Source {
    self.source.clone()
  }

  /// Returns the next [`Token`] in the stream without consuming it.
  #[inline]
  pub fn peek(&mut self) -> Token {
    match self.peeked {
      Some(token) => token,
      None => {
        let token = self.next();
        self.peeked = Some(token);
        token
      }
    }
  }

  /// Returns the next [`Token`] in the stream.
  ///
  /// Once the first [`TokenKind::Eof`] has been returned, it will continue to
  /// return them thereafter, akin to a [`FusedIterator`].
  ///
  /// [`FusedIterator`]: core::iter::FusedIterator
  #[allow(clippy::should_implement_trait)]
  pub fn next(&mut self) -> Token {
    if let Some(token) = self.peeked.take() {
      return token;
    }

    let source = self.source.source();

    let mut state = State::Start;
    let mut start = self.cursor;
    let mut chars = source[self.cursor..].chars();

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
          // '[' => {
          //   self.cursor += c_len;

          //   break Token {
          //     kind: TokenKind::LeftSquare,
          //     span: Span {
          //       start,
          //       end: self.cursor,
          //     },
          //   };
          // }
          // ']' => {
          //   self.cursor += c_len;

          //   break Token {
          //     kind: TokenKind::RightSquare,
          //     span: Span {
          //       start,
          //       end: self.cursor,
          //     },
          //   };
          // }
          ';' => state = State::Comment,
          '-' => state = State::Minus,
          '0'..='9' => state = State::Integer,
          '"' => state = State::String,
          // NOTE: If this is modified, remember to change the other instances
          //       in the other State matches.
          '_'
          | '+'
          | '*'
          | '/'
          | '%'
          | ':'
          | '!'
          | '='
          | '<'
          | '>'
          | '?'
          | 'a'..='z'
          | 'A'..='Z' => state = State::Symbol,
          // TODO: Square brackets should be checked in the parsing step.
          ' ' | '\n' | '\t' | '\r' | '[' | ']' => start = self.cursor + c_len,
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
            start = self.cursor;
            self.cursor -= c_len;
          }
          '\n' => {
            state = State::Start;
            start = self.cursor + c_len;
          }
          _ => {}
        },
        State::Minus => match c {
          '0'..='9' => state = State::Integer,
          '_'
          | '+'
          | '-'
          | '*'
          | '/'
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
              kind: TokenKind::Symbol,
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
        State::String => match c {
          '\0' if self.cursor == source.len() => {
            break Token {
              kind: TokenKind::Invalid,
              span: Span {
                start,
                end: self.cursor,
              },
            };
          }
          '\n' => {
            break Token {
              kind: TokenKind::Invalid,
              span: Span {
                start,
                end: self.cursor,
              },
            };
          }
          '\\' => state = State::StringBackslash,
          '"' => {
            self.cursor += c_len;

            break Token {
              kind: TokenKind::String,
              span: Span {
                start,
                end: self.cursor,
              },
            };
          }
          _ => {}
        },
        State::StringBackslash => match c {
          '\0' | '\n' => {
            break Token {
              kind: TokenKind::Invalid,
              span: Span {
                start,
                end: self.cursor,
              },
            };
          }
          _ => state = State::String,
        },
        State::Symbol => match c {
          '_'
          | '+'
          | '-'
          | '*'
          | '/'
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
            break Token {
              kind: TokenKind::Symbol,
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
  String,
  StringBackslash,
  Symbol,
}

#[cfg(test)]
mod test {
  use super::*;
  use test_case::case;

  #[case("" => vec![Token { kind: TokenKind::Eof, span: Span { start: 0, end: 0 } }] ; "eof")]
  #[case(" \n\t\r" => vec![Token { kind: TokenKind::Eof, span: Span { start: 4, end: 4 } }] ; "whitespace eof")]
  #[case("; comment" => vec![Token { kind: TokenKind::Eof, span: Span { start: 9, end: 9 } }] ; "comment eof")]
  #[case("; comment\n" => vec![Token { kind: TokenKind::Eof, span: Span { start: 10, end: 10 } }] ; "comment whitespace eof")]
  #[case("+" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "plus only")]
  #[case("-" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "minus only")]
  #[case("*" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "asterisk only")]
  #[case("/" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "slash only")]
  #[case("%" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "percent only")]
  #[case("+a" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 2 } }, Token { kind: TokenKind::Eof, span: Span { start: 2, end: 2 } }] ; "plus symbol only")]
  #[case("-a" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 2 } }, Token { kind: TokenKind::Eof, span: Span { start: 2, end: 2 } }] ; "minus symbol only")]
  #[case("*a" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 2 } }, Token { kind: TokenKind::Eof, span: Span { start: 2, end: 2 } }] ; "asterisk symbol only")]
  #[case("/a" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 2 } }, Token { kind: TokenKind::Eof, span: Span { start: 2, end: 2 } }] ; "slash symbol only")]
  #[case("%a" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 2 } }, Token { kind: TokenKind::Eof, span: Span { start: 2, end: 2 } }] ; "percent symbol only")]
  #[case("'" => vec![Token { kind: TokenKind::Apostrophe, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "apostrophe")]
  #[case("(" => vec![Token { kind: TokenKind::LeftParen, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "left paren")]
  #[case(")" => vec![Token { kind: TokenKind::RightParen, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "right paren")]
  // #[case("[" => vec![Token { kind: TokenKind::LeftSquare, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "left square")]
  // #[case("]" => vec![Token { kind: TokenKind::RightSquare, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "right square")]
  #[case("123" => vec![Token { kind: TokenKind::Integer, span: Span { start: 0, end: 3 } }, Token { kind: TokenKind::Eof, span: Span { start: 3, end: 3 } }] ; "integer")]
  #[case("-123" => vec![Token { kind: TokenKind::Integer, span: Span { start: 0, end: 4 } }, Token { kind: TokenKind::Eof, span: Span { start: 4, end: 4 } }] ; "negative integer")]
  #[case("1.2" => vec![Token { kind: TokenKind::Float, span: Span { start: 0, end: 3 } }, Token { kind: TokenKind::Eof, span: Span { start: 3, end: 3 } }] ; "float")]
  #[case("-1.2" => vec![Token { kind: TokenKind::Float, span: Span { start: 0, end: 4 } }, Token { kind: TokenKind::Eof, span: Span { start: 4, end: 4 } }] ; "negative float")]
  #[case("hello" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 5 } }, Token { kind: TokenKind::Eof, span: Span { start: 5, end: 5 } }] ; "symbol")]
  #[case("h3l10" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 5 } }, Token { kind: TokenKind::Eof, span: Span { start: 5, end: 5 } }] ; "alphanumeric symbol")]
  #[case("he_lo" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 5 } }, Token { kind: TokenKind::Eof, span: Span { start: 5, end: 5 } }] ; "underscore symbol")]
  #[case("he-lo" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 5 } }, Token { kind: TokenKind::Eof, span: Span { start: 5, end: 5 } }] ; "hypen symbol")]
  #[case("_" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "underscore symbol only")]
  #[case(":" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "colon symbol only")]
  #[case("!" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "exclamation symbol only")]
  #[case("=" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "equals symbol only")]
  #[case("<" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "left angle symbol only")]
  #[case(">" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "right angle symbol only")]
  #[case("?" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eof, span: Span { start: 1, end: 1 } }] ; "question symbol only")]
  #[case("nil" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 3 } }, Token { kind: TokenKind::Eof, span: Span { start: 3, end: 3 } }] ; "nil")]
  #[case("fn" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 2 } }, Token { kind: TokenKind::Eof, span: Span { start: 2, end: 2 } }] ; "fn_")]
  #[case("fn!" => vec![Token { kind: TokenKind::Symbol, span: Span { start: 0, end: 3 } }, Token { kind: TokenKind::Eof, span: Span { start: 3, end: 3 } }] ; "fn exclamation")]
  #[case("\"hello\"" => vec![Token { kind: TokenKind::String, span: Span { start: 0, end: 7 } }, Token { kind: TokenKind::Eof, span: Span { start: 7, end: 7 } }] ; "string")]
  fn lexer(source: &str) -> Vec<Token> {
    let source = Source::new("", source);
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

  #[test]
  fn peek() {
    let source = "1 2";
    let source = Source::new("", source);
    let mut lexer = Lexer::new(source);

    assert_eq!(
      lexer.next(),
      Token {
        kind: TokenKind::Integer,
        span: Span { start: 0, end: 1 }
      }
    );
    assert_eq!(
      lexer.peek(),
      Token {
        kind: TokenKind::Integer,
        span: Span { start: 2, end: 3 }
      }
    );
    assert_eq!(
      lexer.next(),
      Token {
        kind: TokenKind::Integer,
        span: Span { start: 2, end: 3 }
      }
    );
    assert_eq!(
      lexer.next(),
      Token {
        kind: TokenKind::Eof,
        span: Span { start: 3, end: 3 }
      }
    );
  }
}
