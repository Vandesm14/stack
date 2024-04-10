use core::fmt;

use internment::Intern;

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
      Some(token) => token.clone(),
      None => loop {
        let token = self.lexer.next();
        self.tokens.push(token.clone());

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
    #[allow(clippy::obfuscated_if_else)]
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
  #[allow(clippy::should_implement_trait)]
  // ^ This is fine. If it acts like an iterator, it's an iterator.
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
              span: Span {
                start,
                end: self.cursor,
              },
            };
          }
          // TODO: Remove square brackets from this once they become semantic.
          ' ' | '\t' | '\r' | '\n' | '[' | ']' => state = State::Whitespace,
          ';' => state = State::Comment,
          '0'..='9' => state = State::Int,
          '"' => state = State::String,
          'a'..='z'
          | 'A'..='Z'
          | '_'
          | '+'
          | '*'
          | '/'
          | ':'
          | '%'
          | '!'
          | '='
          | '<'
          | '>'
          | '?' => state = State::Ident,
          '\'' => {
            self.cursor += char_width;

            break Token {
              kind: TokenKind::Apostrophe,
              span: Span {
                start,
                end: self.cursor,
              },
            };
          }
          '(' => {
            self.cursor += char_width;

            break Token {
              kind: TokenKind::ParenOpen,
              span: Span {
                start,
                end: self.cursor,
              },
            };
          }
          ')' => {
            self.cursor += char_width;

            break Token {
              kind: TokenKind::ParenClose,
              span: Span {
                start,
                end: self.cursor,
              },
            };
          }
          '-' => state = State::Hyphen,
          _ => state = State::Invalid,
        },
        State::Invalid => match char {
          '\0' | ' ' | '\t' | '\r' | '\n' | '(' | ')' | '{' | '}' | '['
          | ']' | '"' => {
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
        State::Whitespace => match char {
          ' ' | '\t' | '\r' | '\n' => {}
          _ => {
            break Token {
              kind: TokenKind::Whitespace,
              span: Span {
                start,
                end: self.cursor,
              },
            };
          }
        },
        State::Comment => match char {
          '\0' | '\n' => {
            break Token {
              kind: TokenKind::Comment,
              span: Span {
                start,
                end: self.cursor,
              },
            };
          }
          _ => {}
        },
        State::Int => match char {
          '0'..='9' => {}
          '.' => state = State::Float,
          _ => {
            let slice = &self.source[start..self.cursor];
            let span = Span {
              start,
              end: self.cursor,
            };

            break match slice.parse() {
              Ok(value) => Token {
                kind: TokenKind::Integer(value),
                span,
              },
              Err(_) => Token {
                kind: TokenKind::Invalid,
                span,
              },
            };
          }
        },
        State::Float => match char {
          '0'..='9' => {}
          _ => {
            let slice = &self.source[start..self.cursor];
            let span = Span {
              start,
              end: self.cursor,
            };

            break match slice.parse() {
              Ok(value) => Token {
                kind: TokenKind::Float(value),
                span,
              },
              Err(_) => Token {
                kind: TokenKind::Invalid,
                span,
              },
            };
          }
        },
        State::String => match char {
          '\0' => {
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
            // Since this is a concrete token stream, the quotes are
            // included in the length. However, we only want to
            // intern the inner slice.
            let slice = &self.source[start + 1..self.cursor];
            self.cursor += 1;

            break Token {
              kind: TokenKind::String(slice.into()),
              span: Span {
                start,
                end: self.cursor,
              },
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
              span: Span {
                start,
                end: self.cursor,
              },
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
            let ident = Intern::from_ref(slice);

            let kind = match ident {
              ident if ident == Intern::from_ref("nil") => TokenKind::Nil,
              ident if ident == Intern::from_ref("false") => {
                TokenKind::Boolean(false)
              }
              ident if ident == Intern::from_ref("true") => {
                TokenKind::Boolean(true)
              }
              ident if ident == Intern::from_ref("fn") => TokenKind::Fn,
              ident if ident == Intern::from_ref("fn!") => {
                TokenKind::FnExclamation
              }
              ident => TokenKind::Ident(ident),
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
              kind: TokenKind::Ident(Intern::from_ref("-")),
              span: Span {
                start,
                end: self.cursor,
              },
            };
          }
        },
      }

      self.cursor += char_width;
    }
  }
}

/// Contains information about a source code token.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Token {
  /// The lexeme kind.
  pub kind: TokenKind,
  /// The [`Span`] in bytes that this token represents.
  pub span: Span,
}

/// [`Token`] lexeme kinds.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
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
  String(String),

  /// Sequence of identifier [`char`]s.
  Ident(Intern<String>),

  /// `'` symbol.
  Apostrophe,
  /// `(` symbol.
  ParenOpen,
  /// `)` symbol.
  ParenClose,
  // /// `[` symbol.
  // SquareOpen,
  // /// `]` symbol.
  // SquareClose,
  /// `nil` keyword.
  Nil,
  /// `fn` keyword.
  Fn,
  /// `fn!` keyword.
  FnExclamation,
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
      Self::String(x) => fmt::Display::fmt(x, f),

      // TODO: Should this display the kind of token too?
      Self::Ident(x) => fmt::Display::fmt(x.as_ref(), f),

      Self::Apostrophe => f.write_str("'"),
      Self::ParenOpen => f.write_str("("),
      Self::ParenClose => f.write_str(")"),
      // Self::SquareOpen => f.write_str("["),
      // Self::SquareClose => f.write_str("]"),
      Self::Nil => f.write_str("nil"),
      Self::Fn => f.write_str("fn"),
      Self::FnExclamation => f.write_str("fn!"),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
  /// The lower bound (inclusive).
  pub start: usize,
  /// The upper bound (exclusive).
  pub end: usize,
}

impl Span {
  pub fn new(start: usize, end: usize) -> Self {
    Span { start, end }
  }
}

impl fmt::Display for Span {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}..{}", self.start, self.end)
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

  #[test_case("" => vec![Token { kind: TokenKind::Eoi, span: Span { start: 0, end: 0 } }] ; "empty")]
  #[test_case(" \t\r\n" => vec![Token { kind: TokenKind::Whitespace, span: Span { start: 0, end: 4 } }, Token { kind: TokenKind::Eoi, span: Span { start: 4, end: 4 } }] ; "whitespace")]
  #[test_case("ßℝ💣" => vec![Token { kind: TokenKind::Invalid, span: Span { start: 0, end: 9 } }, Token { kind: TokenKind::Eoi, span: Span { start: 9, end: 9 } }] ; "invalid eoi")]
  #[test_case("ßℝ💣\n" => vec![Token { kind: TokenKind::Invalid, span: Span { start: 0, end: 9 } }, Token { kind: TokenKind::Whitespace, span: Span { start: 9, end: 10 } }, Token { kind: TokenKind::Eoi, span: Span { start: 10, end: 10 } }] ; "invalid whitespace")]
  #[test_case("; Comment" => vec![Token { kind: TokenKind::Comment, span: Span { start: 0, end: 9 } }, Token { kind: TokenKind::Eoi, span: Span { start: 9, end: 9 }}] ; "comment eoi")]
  #[test_case("; Comment\n" => vec![Token { kind: TokenKind::Comment, span: Span { start: 0, end: 9 } }, Token { kind: TokenKind::Whitespace, span: Span { start: 9, end: 10 } }, Token { kind: TokenKind::Eoi, span: Span { start: 10, end: 10 } }] ; "comment whitespace")]
  #[test_case("123" => vec![Token { kind: TokenKind::Integer(123), span: Span { start: 0, end: 3 } }, Token { kind: TokenKind::Eoi, span: Span { start: 3, end: 3 } }] ; "integer eoi")]
  #[test_case("-123" => vec![Token { kind: TokenKind::Integer(-123), span: Span { start: 0, end: 4 } }, Token { kind: TokenKind::Eoi, span: Span { start: 4, end: 4 } }] ; "negative integer eoi")]
  #[test_case("123." => vec![Token { kind: TokenKind::Float(123.0), span: Span { start: 0, end: 4 } }, Token { kind: TokenKind::Eoi, span: Span { start: 4, end: 4 } }] ; "float without fractional eoi")]
  #[test_case("-123." => vec![Token { kind: TokenKind::Float(-123.0), span: Span { start: 0, end: 5 } }, Token { kind: TokenKind::Eoi, span: Span { start: 5, end: 5 } }] ; "negative float without fractional eoi")]
  #[test_case("123.456" => vec![Token { kind: TokenKind::Float(123.456), span: Span { start: 0, end: 7 } }, Token { kind: TokenKind::Eoi, span: Span { start: 7, end: 7 } }] ; "float with fractional eoi")]
  #[test_case("-123.456" => vec![Token { kind: TokenKind::Float(-123.456), span: Span { start: 0, end: 8 } }, Token { kind: TokenKind::Eoi, span: Span { start: 8, end: 8 } }] ; "negative float with fractional eoi")]
  #[test_case("\"hello\"" => vec![Token { kind: TokenKind::String("hello".into()), span: Span { start: 0, end: 7 } }, Token { kind: TokenKind::Eoi, span: Span { start: 7, end: 7 } }] ; "string eoi")]
  #[test_case("\"he\\tlo\"" => vec![Token { kind: TokenKind::String("he\\tlo".into()), span: Span { start: 0, end: 8 } }, Token { kind: TokenKind::Eoi, span: Span { start: 8, end: 8 } }] ; "string escape eoi")]
  #[test_case("\"hello" => vec![Token { kind: TokenKind::Invalid, span: Span { start: 0, end: 6 } }, Token { kind: TokenKind::Eoi, span: Span { start: 6, end: 6 } }] ; "string missing end quote eoi")]
  #[test_case("\"hello\n" => vec![Token { kind: TokenKind::Invalid, span: Span { start: 0, end: 7 } }, Token { kind: TokenKind::Eoi, span: Span { start: 7, end: 7 } }] ; "string missing end quote whitespace")]
  #[test_case("false" => vec![Token { kind: TokenKind::Boolean(false), span: Span { start: 0, end: 5 } }, Token { kind: TokenKind::Eoi, span: Span { start: 5, end: 5 } }] ; "boolean false eoi")]
  #[test_case("true" => vec![Token { kind: TokenKind::Boolean(true), span: Span { start: 0, end: 4 } }, Token { kind: TokenKind::Eoi, span: Span { start: 4, end: 4 } }] ; "boolean true eoi")]
  #[test_case("hello" => vec![Token { kind: TokenKind::Ident(Intern::from_ref("hello")), span: Span { start: 0, end: 5 } }, Token { kind: TokenKind::Eoi, span: Span { start: 5, end: 5 } }] ; "ident eoi")]
  #[test_case("-hello" => vec![Token { kind: TokenKind::Ident(Intern::from_ref("-hello")), span: Span { start: 0, end: 6 } }, Token { kind: TokenKind::Eoi, span: Span { start: 6, end: 6 } }] ; "ident starting hypen eoi")]
  #[test_case("hey-o" => vec![Token { kind: TokenKind::Ident(Intern::from_ref("hey-o")), span: Span { start: 0, end: 5 } }, Token { kind: TokenKind::Eoi, span: Span { start: 5, end: 5 } }] ; "kebab ident eoi")]
  #[test_case("+" => vec![Token { kind: TokenKind::Ident(Intern::from_ref("+")), span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eoi, span: Span { start: 1, end: 1 } }] ; "plus ident eoi")]
  #[test_case("-" => vec![Token { kind: TokenKind::Ident(Intern::from_ref("-")), span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eoi, span: Span { start: 1, end: 1 } }] ; "minus ident eoi")]
  #[test_case("!=" => vec![Token { kind: TokenKind::Ident(Intern::from_ref("!=")), span: Span { start: 0, end: 2 } }, Token { kind: TokenKind::Eoi, span: Span { start: 2, end: 2 } }] ; "exclamation equals ident eoi")]
  #[test_case("'" => vec![Token { kind: TokenKind::Apostrophe, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eoi, span: Span { start: 1, end: 1 } }] ; "apostrophe eoi")]
  #[test_case("(" => vec![Token { kind: TokenKind::ParenOpen, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eoi, span: Span { start: 1, end: 1 } }] ; "paren open eoi")]
  #[test_case(")" => vec![Token { kind: TokenKind::ParenClose, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eoi, span: Span { start: 1, end: 1 } }] ; "paren close eoi")]
  #[test_case("()" => vec![Token { kind: TokenKind::ParenOpen, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::ParenClose, span: Span { start: 1, end: 2 } }, Token { kind: TokenKind::Eoi, span: Span { start: 2, end: 2 } }] ; "paren open close eoi")]
  // #[test_case("[" => vec![Token { kind: TokenKind::SquareOpen, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eoi, span: Span { start: 1, end: 1 } }] ; "square open eoi")]
  // #[test_case("]" => vec![Token { kind: TokenKind::SquareClose, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::Eoi, span: Span { start: 1, end: 1 } }] ; "square close eoi")]
  // #[test_case("[]" => vec![Token { kind: TokenKind::SquareOpen, span: Span { start: 0, end: 1 } }, Token { kind: TokenKind::SquareClose, span: Span { start: 1, end: 2 } }, Token { kind: TokenKind::Eoi, span: Span { start: 2, end: 2 } }] ; "square open close eoi")]
  fn lexer(source: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::with_capacity(8);

    loop {
      let token = lexer.next();
      tokens.push(token.clone());

      if token.kind == TokenKind::Eoi {
        break;
      }
    }

    tokens
  }

  #[test_case("", 0 => Token { kind: TokenKind::Eoi, span: Span { start: 0, end: 0 } } ; "empty at 0")]
  #[test_case("", 1 => Token { kind: TokenKind::Eoi, span: Span { start: 0, end: 0 } } ; "empty at 1")]
  #[test_case("", 3 => Token { kind: TokenKind::Eoi, span: Span { start: 0, end: 0 } } ; "empty at 3")]
  #[test_case("123", 0 => Token { kind: TokenKind::Integer(123), span: Span { start: 0, end: 3 } } ; "single at 0")]
  #[test_case("123", 1 => Token { kind: TokenKind::Eoi, span: Span { start: 3, end: 3 } } ; "single at 1")]
  #[test_case("123", 3 => Token { kind: TokenKind::Eoi, span: Span { start: 3, end: 3 } } ; "single at 3")]
  #[test_case("hello 123", 0 => Token { kind: TokenKind::Ident(Intern::from_ref("hello")), span: Span { start: 0, end: 5 } } ; "many at 0")]
  #[test_case("hello 123", 1 => Token { kind: TokenKind::Whitespace, span: Span { start: 5, end: 6 } } ; "many at 1")]
  #[test_case("hello 123", 2 => Token { kind: TokenKind::Integer(123), span: Span { start: 6, end: 9 } } ; "many at 2")]
  #[test_case("hello 123", 5 => Token { kind: TokenKind::Eoi, span: Span { start: 9, end: 9 } } ; "many at 4")]
  #[test_case("hello 123", 5 => Token { kind: TokenKind::Eoi, span: Span { start: 9, end: 9 } } ; "many at 5")]
  fn token_vec(source: &str, index: usize) -> Token {
    let lexer = Lexer::new(source);
    let mut token_vec = TokenVec::new(lexer);
    token_vec.token(index)
  }
}
