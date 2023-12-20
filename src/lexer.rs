#[derive(Debug, Clone, PartialEq)]
pub enum Token {
  Integer(i64),
  Float(f64),

  String(String),
  Call(String),

  ParenStart,
  ParenEnd,

  BracketStart,
  BracketEnd,

  Nil,
}

#[derive(Debug, Clone, Copy)]
enum State {
  Start,
  String,
  Integer,
  Float,
  Call,
  Comment,
  NegSign,
}

fn is_symbol_start(c: char) -> bool {
  matches!(c, 'a'..='z' | 'A'..='Z' | '_' | '+' | '*' | '/' | '=' | '%' | '!' | '&' | '|' | '<' | '>' | '?' | '$' | '-' | '~' | '^' | '@')
}

fn is_symbol(c: char) -> bool {
  // Numbers can be part of a symbol, but not the first character
  is_symbol_start(c) || c.is_ascii_digit()
}

pub fn lex(input: &str) -> Vec<Token> {
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

            // Match a start bracket
            '[' => {
              tokens.push(Token::BracketStart);
              State::Start
            }
            // Match an end bracket
            ']' => {
              tokens.push(Token::BracketEnd);
              State::Start
            }

            // Ignore whitespace
            c if c.is_whitespace() => State::Start,

            // Ignore Curly Brackets
            '{' | '}' => State::Start,

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
          _ => {
            accumulator.push(c);
            i += 1;
            State::String
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
        tokens.push(Token::String(accumulator.clone()));
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
        tokens.push(Token::Call('-'.into()));
        accumulator.clear();
      }
      (State::Call, State::Start) => {
        let call = accumulator.clone();

        tokens.push(match call.as_str() {
          "nil" => Token::Nil,
          _ => Token::Call(call),
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
    let input = "123";
    let expected = vec![Token::Integer(123)];

    assert_eq!(lex(input), expected);
  }

  #[test]
  fn test_negative_integer() {
    let input = "-123";
    let expected = vec![Token::Integer(-123)];

    assert_eq!(lex(input), expected);
  }

  #[test]
  fn test_float() {
    let input = "123.456";
    let expected = vec![Token::Float(123.456)];

    assert_eq!(lex(input), expected);
  }

  #[test]
  fn test_negative_float() {
    let input = "-123.456";
    let expected = vec![Token::Float(-123.456)];

    assert_eq!(lex(input), expected);
  }

  #[test]
  fn test_string() {
    let input = "\"Hello, world!\"";
    let expected = vec![Token::String("Hello, world!".to_string())];

    assert_eq!(lex(input), expected);
  }

  #[test]
  fn test_call_plus() {
    let input = "+";
    let expected = vec![Token::Call("+".to_string())];

    assert_eq!(lex(input), expected);
  }

  #[test]
  fn test_symbol_var() {
    let input = "myVar";
    let expected = vec![Token::Call("myVar".to_string())];

    assert_eq!(lex(input), expected);
  }

  #[test]
  fn test_nil() {
    let input = "nil";
    let expected = vec![Token::Nil];

    assert_eq!(lex(input), expected);
  }

  #[test]
  fn test_multiple() {
    let input = "123 \"Hello, world!\" (h3ll0_worl6) nil";
    let expected = vec![
      Token::Integer(123),
      Token::String("Hello, world!".to_string()),
      Token::ParenStart,
      Token::Call("h3ll0_worl6".to_string()),
      Token::ParenEnd,
      Token::Nil,
    ];

    assert_eq!(lex(input), expected);
  }

  #[test]
  fn test_multiple2() {
    let inpuit = "1 (a2) 3.0 add \"string hello\" var nil";
    let expected = vec![
      Token::Integer(1),
      Token::ParenStart,
      Token::Call("a2".to_string()),
      Token::ParenEnd,
      Token::Float(3.0),
      Token::Call("add".to_string()),
      Token::String("string hello".to_string()),
      Token::Call("var".to_string()),
      Token::Nil,
    ];

    assert_eq!(lex(inpuit), expected);
  }

  #[test]
  fn ignore_whitespace() {
    let input = "1  \n   2    \n 3";
    let expected =
      vec![Token::Integer(1), Token::Integer(2), Token::Integer(3)];

    assert_eq!(lex(input), expected);
  }

  #[test]
  fn ignore_comments() {
    let input = "1; this is a comment\n2; this is another comment";
    let expected = vec![Token::Integer(1), Token::Integer(2)];

    assert_eq!(lex(input), expected);
  }

  #[test]
  fn ignore_curly_brackets() {
    let input = "1 {2} { 3 }";
    let expected =
      vec![Token::Integer(1), Token::Integer(2), Token::Integer(3)];

    assert_eq!(lex(input), expected);
  }
}
