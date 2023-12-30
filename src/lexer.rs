use core::fmt;

use lasso::Spur;

use crate::interner::{interned, interner};

/// Converts a [`Lexer`] into a lazy <code>[Vec]\<[Token]\></code>.
///
/// This is useful in contexts where back-tracking or look-ahead is required,
/// since it collects [`Token`]s from the [`Lexer`] as needed.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct TokenVec<'source> {
  lexer: Lexer<'source>,
  tokens: Vec<Token>,
  eoi: Option<usize>,
}

impl<'source> TokenVec<'source> {
  /// Creates a new [`TokenVec`].
  ///
  /// Prefer [`TokenVec::reuse`] where possible.
  #[inline]
  pub const fn new(lexer: Lexer<'source>) -> Self {
    Self {
      lexer,
      tokens: Vec::new(),
      eoi: None,
    }
  }

  /// Creates a [`TokenVec`] by re-using the allocations of an existing one.
  #[inline]
  pub fn reuse(&mut self, lexer: Lexer<'source>) {
    self.lexer = lexer;
    self.tokens.clear();
  }

  /// Returns a [`Token`] at the index.
  ///
  /// If the index is out of bounds, this returns the [`Token`] at the end index.
  pub fn token(&mut self, mut index: usize) -> Token {
    // Clamp the upper bound to the end of input index, if smaller.
    index = index.min(self.eoi.unwrap_or(usize::MAX));

    match self.tokens.get(index) {
      Some(token) => *token,
      None => loop {
        let token = self.lexer.next();
        self.tokens.push(token);

        if token.kind == TokenKind::Eoi {
          self.eoi = Some(self.tokens.len() - 1);
          break token;
        }

        if self.tokens.len() - 1 == index {
          break token;
        }
      },
    }
  }
}

/// Converts a source code <code>&[str]</code> into a stream of [`Token`]s.
///
/// This produces a concrete [`Token`] stream, such that every character in the
/// source code is represented by a [`Token`], and none are disregarded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Lexer<'source> {
  pub source: &'source str,
  cursor: usize,
}

impl<'source> Lexer<'source> {
  /// Creates a new [`Lexer`].
  #[inline]
  pub fn new(source: &'source str) -> Self {
    // Skip the UTF-8 BOM if present.
    let start = source
      .as_bytes()
      .starts_with(b"\xef\xbb\xbf")
      .then_some(3)
      .unwrap_or(0);

    Self {
      source,
      cursor: start,
    }
  }

  /// Returns the next [`Token`].
  ///
  /// Once the first [`TokenKind::Eoi`] has been returned, it will continue to
  /// return them thereafter, akin to a [`FusedIterator`].
  ///
  /// [`FusedIterator`]: core::iter::FusedIterator
  pub fn next(&mut self) -> Token {
    let mut state = State::Start;
    let mut chars = self.source[self.cursor..].chars();

    let start = self.cursor;

    loop {
      let char = chars.next().unwrap_or('\0');
      let char_width = char.len_utf8();

      match state {
        State::Start => match char {
          '\0' if self.cursor == self.source.len() => {
            break Token {
              kind: TokenKind::Eoi,
              len: (self.cursor - start) as u32,
            };
          }
          ' ' | '\t' | '\r' | '\n' => state = State::Whitespace,
          ';' => state = State::Comment,
          '0'..='9' => state = State::Int,
          '"' => state = State::String,
          'a'..='z'
          | 'A'..='Z'
          | '_'
          | '+'
          | '*'
          | '/'
          | '%'
          | '='
          | '!'
          | '&'
          | '|'
          | '<'
          | '>'
          | '?'
          | '$'
          | '~'
          | '^' => state = State::Ident,
          '\'' => {
            self.cursor += char_width;

            break Token {
              kind: TokenKind::Apostrophe,
              len: (self.cursor - start) as u32,
            };
          }
          '(' => {
            self.cursor += char_width;

            break Token {
              kind: TokenKind::ParenOpen,
              len: (self.cursor - start) as u32,
            };
          }
          ')' => {
            self.cursor += char_width;

            break Token {
              kind: TokenKind::ParenClose,
              len: (self.cursor - start) as u32,
            };
          }
          '{' => {
            self.cursor += char_width;

            break Token {
              kind: TokenKind::CurlyOpen,
              len: (self.cursor - start) as u32,
            };
          }
          '}' => {
            self.cursor += char_width;

            break Token {
              kind: TokenKind::CurlyClose,
              len: (self.cursor - start) as u32,
            };
          }
          '[' => {
            self.cursor += char_width;

            break Token {
              kind: TokenKind::SquareOpen,
              len: (self.cursor - start) as u32,
            };
          }
          ']' => {
            self.cursor += char_width;

            break Token {
              kind: TokenKind::SquareClose,
              len: (self.cursor - start) as u32,
            };
          }
          '-' => state = State::Hyphen,
          _ => state = State::Invalid,
        },
        State::Invalid => match char {
          '\0' | ' ' | '\t' | '\r' | '\n' => {
            break Token {
              kind: TokenKind::Invalid,
              len: (self.cursor - start) as u32,
            };
          }
          _ => {}
        },
        State::Whitespace => match char {
          ' ' | '\t' | '\r' | '\n' => {}
          _ => {
            break Token {
              kind: TokenKind::Whitespace,
              len: (self.cursor - start) as u32,
            };
          }
        },
        State::Comment => match char {
          '\0' | '\n' => {
            break Token {
              kind: TokenKind::Comment,
              len: (self.cursor - start) as u32,
            };
          }
          _ => {}
        },
        State::Int => match char {
          '0'..='9' => {}
          '.' => state = State::Float,
          _ => {
            let slice = &self.source[start..self.cursor];
            let len = (self.cursor - start) as u32;

            break match slice.parse() {
              Ok(value) => Token {
                kind: TokenKind::Integer(value),
                len,
              },
              Err(_) => Token {
                kind: TokenKind::Invalid,
                len,
              },
            };
          }
        },
        State::Float => match char {
          '0'..='9' => {}
          _ => {
            let slice = &self.source[start..self.cursor];
            let len = (self.cursor - start) as u32;

            break match slice.parse() {
              Ok(value) => Token {
                kind: TokenKind::Float(value),
                len,
              },
              Err(_) => Token {
                kind: TokenKind::Invalid,
                len,
              },
            };
          }
        },
        State::String => match char {
          '\0' => {
            break Token {
              kind: TokenKind::Invalid,
              len: (self.cursor - start) as u32,
            };
          }
          '\\' => state = State::StringBackslash,
          '"' => {
            // Since this is a concrete token stream, the quotes are
            // included in the length. However, we only want to
            // intern the inner slice.
            let slice = &self.source[start + 1..self.cursor];
            dbg!(slice);
            let value = interner().get_or_intern(slice);

            self.cursor += 1;

            break Token {
              kind: TokenKind::String(value),
              len: (self.cursor - start) as u32,
            };
          }
          _ => {}
        },
        State::StringBackslash => match char {
          'n' | 'r' | 't' | '0' => state = State::String,
          _ => state = State::StringInvalid,
        },
        State::StringInvalid => match char {
          '\0' | '"' => {
            break Token {
              kind: TokenKind::Invalid,
              len: (self.cursor - start) as u32,
            };
          }
          _ => {}
        },
        State::Ident => match char {
          'a'..='z'
          | 'A'..='Z'
          | '0'..='9'
          | '_'
          | '+'
          | '-'
          | '*'
          | '/'
          | '%'
          | '='
          | '!'
          | '&'
          | '|'
          | '<'
          | '>'
          | '?'
          | '$'
          | '~'
          | '^' => {}
          _ => {
            let slice = &self.source[start..self.cursor];
            let ident = interner().get_or_intern(slice);

            let kind = match ident {
              ident if ident == interned().NIL => TokenKind::Nil,
              ident if ident == interned().FALSE => TokenKind::Boolean(false),
              ident if ident == interned().TRUE => TokenKind::Boolean(true),
              ident if ident == interned().FN => TokenKind::Fn,
              ident => TokenKind::Ident(ident),
            };

            break Token {
              kind,
              len: (self.cursor - start) as u32,
            };
          }
        },
        State::Hyphen => match char {
          '0'..='9' => state = State::Int,
          'a'..='z'
          | 'A'..='Z'
          | '_'
          | '+'
          | '-'
          | '*'
          | '/'
          | '%'
          | '='
          | '!'
          | '&'
          | '|'
          | '<'
          | '>'
          | '?'
          | '$'
          | '~'
          | '^' => state = State::Ident,
          _ => {
            break Token {
              kind: TokenKind::Ident(interned().MINUS),
              len: (self.cursor - start) as u32,
            };
          }
        },
      }

      self.cursor += char_width;
    }
  }
}

/// Contains information about a source code token.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Token {
  /// The lexeme kind.
  pub kind: TokenKind,
  /// The number of bytes that this token represents.
  pub len: u32,
}

/// [`Token`] lexeme kinds.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum TokenKind {
  /// Invalid sequence of [`char`]s.
  Invalid,
  /// End of input.
  Eoi,

  /// Sequence of whitespace [`char`]s.
  Whitespace,
  /// Semicolon until the next newline or end of input.
  Comment,

  /// Boolean literal.
  Boolean(bool),
  /// 64-bit signed integer literal.
  Integer(i64),
  /// 64-bit floating-point literal.
  Float(f64),
  /// Sequence of [`char`]s delimited by double-quotes (`"`).
  String(Spur),

  /// Sequence of identifier [`char`]s.
  Ident(Spur),

  /// `'` symbol.
  Apostrophe,
  /// `(` symbol.
  ParenOpen,
  /// `)` symbol.
  ParenClose,
  /// `{` symbol.
  CurlyOpen,
  /// `}` symbol.
  CurlyClose,
  /// `[` symbol.
  SquareOpen,
  /// `]` symbol.
  SquareClose,

  /// `nil` keyword.
  Nil,
  /// `fn` keyword.
  Fn,
}

impl fmt::Display for TokenKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Invalid => f.write_str("invalid"),
      Self::Eoi => f.write_str("end of input"),

      Self::Whitespace => f.write_str("whitespace"),
      Self::Comment => f.write_str("comment"),

      // TODO: Should this display the kind of token too?
      Self::Boolean(x) => fmt::Display::fmt(x, f),
      // TODO: Should this display the kind of token too?
      Self::Integer(x) => fmt::Display::fmt(x, f),
      // TODO: Should this display the kind of token too?
      Self::Float(x) => fmt::Display::fmt(x, f),
      // TODO: Should this display the kind of token too?
      Self::String(x) => fmt::Display::fmt(interner().resolve(x), f),

      // TODO: Should this display the kind of token too?
      Self::Ident(x) => fmt::Display::fmt(interner().resolve(x), f),

      Self::Apostrophe => f.write_str("'"),
      Self::ParenOpen => f.write_str("("),
      Self::ParenClose => f.write_str(")"),
      Self::CurlyOpen => f.write_str("{"),
      Self::CurlyClose => f.write_str("}"),
      Self::SquareOpen => f.write_str("["),
      Self::SquareClose => f.write_str("]"),

      Self::Nil => f.write_str("nil"),
      Self::Fn => f.write_str("fn"),
    }
  }
}

enum State {
  Start,
  Invalid,
  Whitespace,
  Comment,
  Int,
  Float,
  String,
  StringBackslash,
  StringInvalid,
  Ident,
  Hyphen,
}

#[cfg(test)]
mod test {
  use super::*;

  use test_case::test_case;

  #[test_case("" => vec![Token { kind: TokenKind::Eoi, len: 0 }] ; "empty")]
  #[test_case(" \t\r\n" => vec![Token { kind: TokenKind::Whitespace, len: 4 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "whitespace")]
  #[test_case("ßℝ💣" => vec![Token { kind: TokenKind::Invalid, len: 9 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "invalid eoi")]
  #[test_case("ßℝ💣\n" => vec![Token { kind: TokenKind::Invalid, len: 9 }, Token { kind: TokenKind::Whitespace, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "invalid whitespace")]
  #[test_case("; Comment" => vec![Token { kind: TokenKind::Comment, len: 9 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "comment eoi")]
  #[test_case("; Comment\n" => vec![Token { kind: TokenKind::Comment, len: 9 }, Token { kind: TokenKind::Whitespace, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "comment whitespace")]
  #[test_case("123" => vec![Token { kind: TokenKind::Integer(123), len: 3 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "integer eoi")]
  #[test_case("123\n" => vec![Token { kind: TokenKind::Integer(123), len: 3 }, Token { kind: TokenKind::Whitespace, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "integer whitespace")]
  #[test_case("-123" => vec![Token { kind: TokenKind::Integer(-123), len: 4 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "negative integer eoi")]
  #[test_case("-123\n" => vec![Token { kind: TokenKind::Integer(-123), len: 4 }, Token { kind: TokenKind::Whitespace, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "negative integer whitespace")]
  #[test_case("123." => vec![Token { kind: TokenKind::Float(123.0), len: 4 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "float without fractional eoi")]
  #[test_case("123.\n" => vec![Token { kind: TokenKind::Float(123.0), len: 4 }, Token { kind: TokenKind::Whitespace, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "float without fractional whitespace")]
  #[test_case("-123." => vec![Token { kind: TokenKind::Float(-123.0), len: 5 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "negative float without fractional eoi")]
  #[test_case("-123.\n" => vec![Token { kind: TokenKind::Float(-123.0), len: 5 }, Token { kind: TokenKind::Whitespace, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "negative float without fractional whitespace")]
  #[test_case("123.456" => vec![Token { kind: TokenKind::Float(123.456), len: 7 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "float with fractional eoi")]
  #[test_case("123.456\n" => vec![Token { kind: TokenKind::Float(123.456), len: 7 }, Token { kind: TokenKind::Whitespace, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "float with fractional whitespace")]
  #[test_case("-123.456" => vec![Token { kind: TokenKind::Float(-123.456), len: 8 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "negative float with fractional eoi")]
  #[test_case("-123.456\n" => vec![Token { kind: TokenKind::Float(-123.456), len: 8 }, Token { kind: TokenKind::Whitespace, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "negative float with fractional whitespace")]
  #[test_case("\"hello\"" => vec![Token { kind: TokenKind::String(interner().get_or_intern_static("hello")), len: 7 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "string eoi")]
  #[test_case("\"hello\"\n" => vec![Token { kind: TokenKind::String(interner().get_or_intern_static("hello")), len: 7 }, Token { kind: TokenKind::Whitespace, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "string whitespace")]
  #[test_case("\"he\\tlo\"" => vec![Token { kind: TokenKind::String(interner().get_or_intern_static("he\\tlo")), len: 8 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "string escape eoi")]
  #[test_case("\"he\\tlo\"\n" => vec![Token { kind: TokenKind::String(interner().get_or_intern_static("he\\tlo")), len: 8 }, Token { kind: TokenKind::Whitespace, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "string escape whitespace")]
  #[test_case("\"hello" => vec![Token { kind: TokenKind::Invalid, len: 6 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "string missing end quote eoi")]
  #[test_case("\"hello\n" => vec![Token { kind: TokenKind::Invalid, len: 7 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "string missing end quote whitespace")]
  #[test_case("false" => vec![Token { kind: TokenKind::Boolean(false), len: 5 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "boolean false eoi")]
  #[test_case("false\n" => vec![Token { kind: TokenKind::Boolean(false), len: 5 }, Token { kind: TokenKind::Whitespace, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "boolean false whitespace")]
  #[test_case("true" => vec![Token { kind: TokenKind::Boolean(true), len: 4 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "boolean true eoi")]
  #[test_case("true\n" => vec![Token { kind: TokenKind::Boolean(true), len: 4 }, Token { kind: TokenKind::Whitespace, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "boolean true whitespace")]
  #[test_case("hello" => vec![Token { kind: TokenKind::Ident(interner().get_or_intern_static("hello")), len: 5 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "ident eoi")]
  #[test_case("hello\n" => vec![Token { kind: TokenKind::Ident(interner().get_or_intern_static("hello")), len: 5 }, Token { kind: TokenKind::Whitespace, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "ident whitespace")]
  #[test_case("-hello" => vec![Token { kind: TokenKind::Ident(interner().get_or_intern_static("-hello")), len: 6 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "ident starting hypen eoi")]
  #[test_case("-hello\n" => vec![Token { kind: TokenKind::Ident(interner().get_or_intern_static("-hello")), len: 6 }, Token { kind: TokenKind::Whitespace, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "ident starting hypen whitespace")]
  #[test_case("hey-o" => vec![Token { kind: TokenKind::Ident(interner().get_or_intern_static("hey-o")), len: 5 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "kebab ident eoi")]
  #[test_case("hey-o\n" => vec![Token { kind: TokenKind::Ident(interner().get_or_intern_static("hey-o")), len: 5 }, Token { kind: TokenKind::Whitespace, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "kebab ident whitespace")]
  #[test_case("+" => vec![Token { kind: TokenKind::Ident(interned().PLUS), len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "plus ident eoi")]
  #[test_case("+\n" => vec![Token { kind: TokenKind::Ident(interned().PLUS), len: 1 }, Token { kind: TokenKind::Whitespace, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "plus ident whitespace")]
  #[test_case("-" => vec![Token { kind: TokenKind::Ident(interned().MINUS), len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "minus ident eoi")]
  #[test_case("-\n" => vec![Token { kind: TokenKind::Ident(interned().MINUS), len: 1 }, Token { kind: TokenKind::Whitespace, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "minus ident whitespace")]
  #[test_case("!=" => vec![Token { kind: TokenKind::Ident(interned().EXCLAMATION_EQUALS), len: 2 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "exclamation equals ident eoi")]
  #[test_case("!=\n" => vec![Token { kind: TokenKind::Ident(interned().EXCLAMATION_EQUALS), len: 2 }, Token { kind: TokenKind::Whitespace, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "exclamation equals ident whitespace")]
  #[test_case("'" => vec![Token { kind: TokenKind::Apostrophe, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "apostrophe eoi")]
  #[test_case("'\n" => vec![Token { kind: TokenKind::Apostrophe, len: 1 }, Token { kind: TokenKind::Whitespace, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "apostrophe whitespace")]
  #[test_case("(" => vec![Token { kind: TokenKind::ParenOpen, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "paren open eoi")]
  #[test_case("(\n" => vec![Token { kind: TokenKind::ParenOpen, len: 1 }, Token { kind: TokenKind::Whitespace, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "paren open whitespace")]
  #[test_case(")" => vec![Token { kind: TokenKind::ParenClose, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "paren close eoi")]
  #[test_case(")\n" => vec![Token { kind: TokenKind::ParenClose, len: 1 }, Token { kind: TokenKind::Whitespace, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "paren close whitespace")]
  #[test_case("{" => vec![Token { kind: TokenKind::CurlyOpen, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "curly open eoi")]
  #[test_case("{\n" => vec![Token { kind: TokenKind::CurlyOpen, len: 1 }, Token { kind: TokenKind::Whitespace, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "curly open whitespace")]
  #[test_case("}" => vec![Token { kind: TokenKind::CurlyClose, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "curly close eoi")]
  #[test_case("}\n" => vec![Token { kind: TokenKind::CurlyClose, len: 1 }, Token { kind: TokenKind::Whitespace, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "curly close whitespace")]
  #[test_case("[" => vec![Token { kind: TokenKind::SquareOpen, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "square open eoi")]
  #[test_case("[\n" => vec![Token { kind: TokenKind::SquareOpen, len: 1 }, Token { kind: TokenKind::Whitespace, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "square open whitespace")]
  #[test_case("]" => vec![Token { kind: TokenKind::SquareClose, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "square close eoi")]
  #[test_case("]\n" => vec![Token { kind: TokenKind::SquareClose, len: 1 }, Token { kind: TokenKind::Whitespace, len: 1 }, Token { kind: TokenKind::Eoi, len: 0 }] ; "square close whitespace")]
  fn lexer(source: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::with_capacity(8);

    loop {
      let token = lexer.next();
      tokens.push(token);

      if token.kind == TokenKind::Eoi {
        break;
      }
    }

    tokens
  }

  #[test_case("", 0 => Token { kind: TokenKind::Eoi, len: 0 } ; "empty at 0")]
  #[test_case("", 1 => Token { kind: TokenKind::Eoi, len: 0 } ; "empty at 1")]
  #[test_case("", 3 => Token { kind: TokenKind::Eoi, len: 0 } ; "empty at 3")]
  #[test_case("123", 0 => Token { kind: TokenKind::Integer(123), len: 3 } ; "single at 0")]
  #[test_case("123", 1 => Token { kind: TokenKind::Eoi, len: 0 } ; "single at 1")]
  #[test_case("123", 3 => Token { kind: TokenKind::Eoi, len: 0 } ; "single at 3")]
  #[test_case("hello 123", 0 => Token { kind: TokenKind::Ident(interner().get_or_intern_static("hello")), len: 5 } ; "many at 0")]
  #[test_case("hello 123", 1 => Token { kind: TokenKind::Whitespace, len: 1 } ; "many at 1")]
  #[test_case("hello 123", 2 => Token { kind: TokenKind::Integer(123), len: 3 } ; "many at 2")]
  #[test_case("hello 123", 5 => Token { kind: TokenKind::Eoi, len: 0 } ; "many at 4")]
  #[test_case("hello 123", 5 => Token { kind: TokenKind::Eoi, len: 0 } ; "many at 5")]
  fn token_vec(source: &str, index: usize) -> Token {
    let lexer = Lexer::new(source);
    let mut token_vec = TokenVec::new(lexer);
    token_vec.token(index)
  }
}
