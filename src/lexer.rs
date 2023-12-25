use lasso::Spur;

use crate::Context;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
  Integer(i64),
  Float(f64),

  String(Spur),

  NoEval,
  Call(Spur),

  ParenStart,
  ParenEnd,

  CurlyStart,
  CurlyEnd,

  Nil,
}

#[derive(Debug, Clone, Copy)]
enum State {
  Start,
  String,
  StringEscape,
  Integer,
  Float,
  Call,
  Comment,
  NegSign,
}

fn is_symbol_start(c: char) -> bool {
  matches!(c, 'a'..='z' | 'A'..='Z' | '_' | '+' | '*' | '/' | '=' | '%' | '!' | '&' | '|' | '<' | '>' | '?' | '$' | '-' | '~' | '^')
}

fn is_symbol(c: char) -> bool {
  // Numbers can be part of a symbol, but not the first character
  is_symbol_start(c) || c.is_ascii_digit()
}

pub fn lex(context: &mut Context, input: &str) -> Vec<Token> {
  let mut state = State::Start;
  let mut tokens: Vec<Token> = vec![];

  let input = input.chars().collect::<Vec<_>>();
  let mut i = 0;

  let mut accumulator = String::new();

  loop {
    if i > input.len() {
      break;
    }

    let new_state: State = if i < input.len() {
      let c = input[i];
      match state {
        State::Start => {
          i += 1;

          match c {
            // Match a double-quote
            '"' => State::String,

            '-' => State::NegSign,

            // Match a digit (including negative sign)
            '0'..='9' => {
              accumulator.push(c);
              State::Integer
            }

            // Match a call
            c if is_symbol_start(c) => {
              accumulator.push(c);
              State::Call
            }

            // Match a comment
            ';' => State::Comment,

            // Match a start paren
            '(' => {
              tokens.push(Token::ParenStart);
              State::Start
            }
            // Match an end paren
            ')' => {
              tokens.push(Token::ParenEnd);
              State::Start
            }

            // Ignore square brackets
            '[' | ']' => State::Start,

            // Match a noeval symbol
            '\'' => {
              tokens.push(Token::NoEval);
              State::Start
            }

            // Ignore whitespace
            c if c.is_whitespace() => State::Start,

            // Match a start curly bracket
            '{' => {
              tokens.push(Token::CurlyStart);
              State::Start
            }

            // Match an end curly bracket
            '}' => {
              tokens.push(Token::CurlyEnd);
              State::Start
            }

            // Error on everything else
            _ => {
              eprintln!("Error: Unexpected character: {}", c);
              break;
            }
          }
        }
        State::String => match c {
          '"' => {
            i += 1;
            State::Start
          }
          '\\' => {
            i += 1;
            State::StringEscape
          }
          _ => {
            accumulator.push(c);
            i += 1;
            State::String
          }
        },
        State::StringEscape => match c {
          'n' => {
            i += 1;
            accumulator.push('\n');
            State::String
          }
          'r' => {
            i += 1;
            accumulator.push('\r');
            State::String
          }
          't' => {
            i += 1;
            accumulator.push('\t');
            State::String
          }
          _ => {
            eprintln!("Error: Unexpected character: {}", c);
            break;
          }
        },
        State::Integer => match c {
          '0'..='9' => {
            accumulator.push(c);
            i += 1;
            State::Integer
          }
          '.' => {
            accumulator.push(c);
            i += 1;
            State::Float
          }
          _ => State::Start,
        },
        State::Float => match c {
          '0'..='9' => {
            accumulator.push(c);
            i += 1;
            State::Float
          }
          _ => State::Start,
        },
        State::Call => match c {
          c if is_symbol(c) => {
            accumulator.push(c);
            i += 1;
            State::Call
          }
          _ => State::Start,
        },
        State::NegSign => match c {
          '0'..='9' => {
            accumulator.push('-');
            accumulator.push(c);
            i += 1;
            State::Integer
          }
          _ => State::Start,
        },
        State::Comment => match c {
          '\n' => {
            i += 1;
            State::Start
          }
          _ => {
            i += 1;
            State::Comment
          }
        },
      }
    } else {
      // Evaluates at the end of input (sets the state to Start to trigger an eval)
      i += 1;
      State::Start
    };

    match (state, new_state) {
      (State::String, State::Start) => {
        tokens.push(Token::String(context.intern(accumulator.clone())));
        accumulator.clear();
      }
      (State::Integer, State::Start) => {
        tokens.push(Token::Integer(accumulator.parse::<i64>().unwrap()));
        accumulator.clear();
      }
      (State::Float, State::Start) => {
        tokens.push(Token::Float(accumulator.parse::<f64>().unwrap()));
        accumulator.clear();
      }
      (State::NegSign, State::Start) => {
        tokens.push(Token::Call(context.intern("-")));
        accumulator.clear();
      }
      (State::Call, State::Start) => {
        let call = accumulator.clone();

        tokens.push(match call.as_str() {
          "nil" => Token::Nil,
          _ => Token::Call(context.intern(call)),
        });
        accumulator.clear();
      }
      _ => {}
    };

    state = new_state;
  }

  tokens
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_integer() {
    let mut context = Context::new();

    let input = "123";
    let expected = vec![Token::Integer(123)];

    assert_eq!(lex(&mut context, input), expected);
  }

  #[test]
  fn test_negative_integer() {
    let mut context = Context::new();

    let input = "-123";
    let expected = vec![Token::Integer(-123)];

    assert_eq!(lex(&mut context, input), expected);
  }

  #[test]
  fn test_float() {
    let mut context = Context::new();

    let input = "123.456";
    let expected = vec![Token::Float(123.456)];

    assert_eq!(lex(&mut context, input), expected);
  }

  #[test]
  fn test_negative_float() {
    let mut context = Context::new();

    let input = "-123.456";
    let expected = vec![Token::Float(-123.456)];

    assert_eq!(lex(&mut context, input), expected);
  }

  #[test]
  fn test_string() {
    let mut context = Context::new();

    let hello_world = context.intern("Hello, world!");

    let input = "\"Hello, world!\"";
    let expected = vec![Token::String(hello_world)];

    assert_eq!(lex(&mut context, input), expected);
  }

  #[test]
  fn test_call_plus() {
    let mut context = Context::new();

    let plus = context.intern("+");

    let input = "+";
    let expected = vec![Token::Call(plus)];

    assert_eq!(lex(&mut context, input), expected);
  }

  #[test]
  fn test_symbol_var() {
    let mut context = Context::new();

    let my_var = context.intern("my_var");

    let input = "my_var";
    let expected = vec![Token::Call(my_var)];

    assert_eq!(lex(&mut context, input), expected);
  }

  #[test]
  fn test_nil() {
    let mut context = Context::new();

    let input = "nil";
    let expected = vec![Token::Nil];

    assert_eq!(lex(&mut context, input), expected);
  }

  #[test]
  fn test_multiple() {
    let mut context = Context::new();

    let hello_world = context.intern("Hello, world!");
    let h3ll0_worl6 = context.intern("h3ll0_worl6");

    let input = "123 \"Hello, world!\" (h3ll0_worl6) nil";
    let expected = vec![
      Token::Integer(123),
      Token::String(hello_world),
      Token::ParenStart,
      Token::Call(h3ll0_worl6),
      Token::ParenEnd,
      Token::Nil,
    ];

    assert_eq!(lex(&mut context, input), expected);
  }

  #[test]
  fn test_multiple2() {
    let mut context = Context::new();

    let a2 = context.intern("a2");
    let add = context.intern("add");
    let string_hello = context.intern("string hello");
    let var = context.intern("var");

    let input = "1 (a2) 3.0 add \"string hello\" var nil";
    let expected = vec![
      Token::Integer(1),
      Token::ParenStart,
      Token::Call(a2),
      Token::ParenEnd,
      Token::Float(3.0),
      Token::Call(add),
      Token::String(string_hello),
      Token::Call(var),
      Token::Nil,
    ];

    assert_eq!(lex(&mut context, input), expected);
  }

  #[test]
  fn ignore_whitespace() {
    let mut context = Context::new();

    let input = "1  \n   2    \n 3";
    let expected =
      vec![Token::Integer(1), Token::Integer(2), Token::Integer(3)];

    assert_eq!(lex(&mut context, input), expected);
  }

  #[test]
  fn ignore_comments() {
    let mut context = Context::new();

    let input = "1; this is a comment\n2; this is another comment";
    let expected = vec![Token::Integer(1), Token::Integer(2)];

    assert_eq!(lex(&mut context, input), expected);
  }

  #[test]
  fn curly_brackets() {
    let mut context = Context::new();

    let input = "1 {2} { 3 }";
    let expected = vec![
      Token::Integer(1),
      Token::CurlyStart,
      Token::Integer(2),
      Token::CurlyEnd,
      Token::CurlyStart,
      Token::Integer(3),
      Token::CurlyEnd,
    ];

    assert_eq!(lex(&mut context, input), expected);
  }

  #[test]
  fn curly_brackets_outside() {
    let mut context = Context::new();

    let a = context.intern("a");
    let set = context.intern("set");

    let input = "{2 (a) set}";
    let expected = vec![
      Token::CurlyStart,
      Token::Integer(2),
      Token::ParenStart,
      Token::Call(a),
      Token::ParenEnd,
      Token::Call(set),
      Token::CurlyEnd,
    ];

    assert_eq!(lex(&mut context, input), expected);
  }
}
