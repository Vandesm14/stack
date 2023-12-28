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

////////////////////////////////////////////////////////////////////////////////

// use lasso::Spur;

// use crate::Context;

// #[derive(Debug, Clone, PartialEq)]
// pub enum Token {
//   Integer(i64),
//   Float(f64),

//   String(Spur),

//   NoEval,
//   Call(Spur),

//   ParenStart,
//   ParenEnd,

//   CurlyStart,
//   CurlyEnd,

//   Nil,
// }

// #[derive(Debug, Clone, Copy)]
// enum State {
//   Start,
//   String,
//   StringEscape,
//   Integer,
//   Float,
//   Call,
//   Comment,
//   NegSign,
// }

// fn is_symbol_start(c: char) -> bool {
//   matches!(c, 'a'..='z' | 'A'..='Z' | '_' | '+' | '*' | '/' | '=' | '%' | '!' | '&' | '|' | '<' | '>' | '?' | '$' | '-' | '~' | '^')
// }

// fn is_symbol(c: char) -> bool {
//   // Numbers can be part of a symbol, but not the first character
//   is_symbol_start(c) || c.is_ascii_digit()
// }

// pub fn lex(context: &mut Context, input: &str) -> Vec<Token> {
//   let mut state = State::Start;
//   let mut tokens: Vec<Token> = vec![];

//   let input = input.chars().collect::<Vec<_>>();
//   let mut i = 0;

//   let mut accumulator = String::new();

//   loop {
//     if i > input.len() {
//       break;
//     }

//     let new_state: State = if i < input.len() {
//       let c = input[i];
//       match state {
//         State::Start => {
//           i += 1;

//           match c {
//             // Match a double-quote
//             '"' => State::String,

//             '-' => State::NegSign,

//             // Match a digit (including negative sign)
//             '0'..='9' => {
//               accumulator.push(c);
//               State::Integer
//             }

//             // Match a call
//             c if is_symbol_start(c) => {
//               accumulator.push(c);
//               State::Call
//             }

//             // Match a comment
//             ';' => State::Comment,

//             // Match a start paren
//             '(' => {
//               tokens.push(Token::ParenStart);
//               State::Start
//             }
//             // Match an end paren
//             ')' => {
//               tokens.push(Token::ParenEnd);
//               State::Start
//             }

//             // Ignore square brackets
//             '[' | ']' => State::Start,

//             // Match a noeval symbol
//             '\'' => {
//               tokens.push(Token::NoEval);
//               State::Start
//             }

//             // Ignore whitespace
//             c if c.is_whitespace() => State::Start,

//             // Match a start curly bracket
//             '{' => {
//               tokens.push(Token::CurlyStart);
//               State::Start
//             }

//             // Match an end curly bracket
//             '}' => {
//               tokens.push(Token::CurlyEnd);
//               State::Start
//             }

//             // Error on everything else
//             _ => {
//               eprintln!("Error: Unexpected character: {}", c);
//               break;
//             }
//           }
//         }
//         State::String => match c {
//           '"' => {
//             i += 1;
//             State::Start
//           }
//           '\\' => {
//             i += 1;
//             State::StringEscape
//           }
//           _ => {
//             accumulator.push(c);
//             i += 1;
//             State::String
//           }
//         },
//         State::StringEscape => match c {
//           'n' => {
//             i += 1;
//             accumulator.push('\n');
//             State::String
//           }
//           'r' => {
//             i += 1;
//             accumulator.push('\r');
//             State::String
//           }
//           't' => {
//             i += 1;
//             accumulator.push('\t');
//             State::String
//           }
//           _ => {
//             eprintln!("Error: Unexpected character: {}", c);
//             break;
//           }
//         },
//         State::Integer => match c {
//           '0'..='9' => {
//             accumulator.push(c);
//             i += 1;
//             State::Integer
//           }
//           '.' => {
//             accumulator.push(c);
//             i += 1;
//             State::Float
//           }
//           _ => State::Start,
//         },
//         State::Float => match c {
//           '0'..='9' => {
//             accumulator.push(c);
//             i += 1;
//             State::Float
//           }
//           _ => State::Start,
//         },
//         State::Call => match c {
//           c if is_symbol(c) => {
//             accumulator.push(c);
//             i += 1;
//             State::Call
//           }
//           _ => State::Start,
//         },
//         State::NegSign => match c {
//           '0'..='9' => {
//             accumulator.push('-');
//             accumulator.push(c);
//             i += 1;
//             State::Integer
//           }
//           _ => State::Start,
//         },
//         State::Comment => match c {
//           '\n' => {
//             i += 1;
//             State::Start
//           }
//           _ => {
//             i += 1;
//             State::Comment
//           }
//         },
//       }
//     } else {
//       // Evaluates at the end of input (sets the state to Start to trigger an eval)
//       i += 1;
//       State::Start
//     };

//     match (state, new_state) {
//       (State::String, State::Start) => {
//         tokens.push(Token::String(context.intern(accumulator.clone())));
//         accumulator.clear();
//       }
//       (State::Integer, State::Start) => {
//         tokens.push(Token::Integer(accumulator.parse::<i64>().unwrap()));
//         accumulator.clear();
//       }
//       (State::Float, State::Start) => {
//         tokens.push(Token::Float(accumulator.parse::<f64>().unwrap()));
//         accumulator.clear();
//       }
//       (State::NegSign, State::Start) => {
//         tokens.push(Token::Call(context.intern("-")));
//         accumulator.clear();
//       }
//       (State::Call, State::Start) => {
//         let call = accumulator.clone();

//         tokens.push(match call.as_str() {
//           "nil" => Token::Nil,
//           _ => Token::Call(context.intern(call)),
//         });
//         accumulator.clear();
//       }
//       _ => {}
//     };

//     state = new_state;
//   }

//   tokens
// }

// #[cfg(test)]
// mod tests {
//   use super::*;

//   #[test]
//   fn test_integer() {
//     let mut context = Context::new();

//     let input = "123";
//     let expected = vec![Token::Integer(123)];

//     assert_eq!(lex(&mut context, input), expected);
//   }

//   #[test]
//   fn test_negative_integer() {
//     let mut context = Context::new();

//     let input = "-123";
//     let expected = vec![Token::Integer(-123)];

//     assert_eq!(lex(&mut context, input), expected);
//   }

//   #[test]
//   fn test_float() {
//     let mut context = Context::new();

//     let input = "123.456";
//     let expected = vec![Token::Float(123.456)];

//     assert_eq!(lex(&mut context, input), expected);
//   }

//   #[test]
//   fn test_negative_float() {
//     let mut context = Context::new();

//     let input = "-123.456";
//     let expected = vec![Token::Float(-123.456)];

//     assert_eq!(lex(&mut context, input), expected);
//   }

//   #[test]
//   fn test_string() {
//     let mut context = Context::new();

//     let hello_world = context.intern("Hello, world!");

//     let input = "\"Hello, world!\"";
//     let expected = vec![Token::String(hello_world)];

//     assert_eq!(lex(&mut context, input), expected);
//   }

//   #[test]
//   fn test_call_plus() {
//     let mut context = Context::new();

//     let plus = context.intern("+");

//     let input = "+";
//     let expected = vec![Token::Call(plus)];

//     assert_eq!(lex(&mut context, input), expected);
//   }

//   #[test]
//   fn test_symbol_var() {
//     let mut context = Context::new();

//     let my_var = context.intern("my_var");

//     let input = "my_var";
//     let expected = vec![Token::Call(my_var)];

//     assert_eq!(lex(&mut context, input), expected);
//   }

//   #[test]
//   fn test_nil() {
//     let mut context = Context::new();

//     let input = "nil";
//     let expected = vec![Token::Nil];

//     assert_eq!(lex(&mut context, input), expected);
//   }

//   #[test]
//   fn test_multiple() {
//     let mut context = Context::new();

//     let hello_world = context.intern("Hello, world!");
//     let h3ll0_worl6 = context.intern("h3ll0_worl6");

//     let input = "123 \"Hello, world!\" (h3ll0_worl6) nil";
//     let expected = vec![
//       Token::Integer(123),
//       Token::String(hello_world),
//       Token::ParenStart,
//       Token::Call(h3ll0_worl6),
//       Token::ParenEnd,
//       Token::Nil,
//     ];

//     assert_eq!(lex(&mut context, input), expected);
//   }

//   #[test]
//   fn test_multiple2() {
//     let mut context = Context::new();

//     let a2 = context.intern("a2");
//     let add = context.intern("add");
//     let string_hello = context.intern("string hello");
//     let var = context.intern("var");

//     let input = "1 (a2) 3.0 add \"string hello\" var nil";
//     let expected = vec![
//       Token::Integer(1),
//       Token::ParenStart,
//       Token::Call(a2),
//       Token::ParenEnd,
//       Token::Float(3.0),
//       Token::Call(add),
//       Token::String(string_hello),
//       Token::Call(var),
//       Token::Nil,
//     ];

//     assert_eq!(lex(&mut context, input), expected);
//   }

//   #[test]
//   fn ignore_whitespace() {
//     let mut context = Context::new();

//     let input = "1  \n   2    \n 3";
//     let expected =
//       vec![Token::Integer(1), Token::Integer(2), Token::Integer(3)];

//     assert_eq!(lex(&mut context, input), expected);
//   }

//   #[test]
//   fn ignore_comments() {
//     let mut context = Context::new();

//     let input = "1; this is a comment\n2; this is another comment";
//     let expected = vec![Token::Integer(1), Token::Integer(2)];

//     assert_eq!(lex(&mut context, input), expected);
//   }

//   #[test]
//   fn curly_brackets() {
//     let mut context = Context::new();

//     let input = "1 {2} { 3 }";
//     let expected = vec![
//       Token::Integer(1),
//       Token::CurlyStart,
//       Token::Integer(2),
//       Token::CurlyEnd,
//       Token::CurlyStart,
//       Token::Integer(3),
//       Token::CurlyEnd,
//     ];

//     assert_eq!(lex(&mut context, input), expected);
//   }

//   #[test]
//   fn curly_brackets_outside() {
//     let mut context = Context::new();

//     let a = context.intern("a");
//     let set = context.intern("set");

//     let input = "{2 (a) set}";
//     let expected = vec![
//       Token::CurlyStart,
//       Token::Integer(2),
//       Token::ParenStart,
//       Token::Call(a),
//       Token::ParenEnd,
//       Token::Call(set),
//       Token::CurlyEnd,
//     ];

//     assert_eq!(lex(&mut context, input), expected);
//   }
// }
