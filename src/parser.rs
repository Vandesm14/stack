#[derive(Debug, Clone, PartialEq)]
pub enum Token {
  Integer(i64),
  Float(f64),
  String(String),
  Symbol(String),
  Call(String),
  Nil,
}

#[derive(Debug, Clone, Copy)]
enum State {
  Start,
  String,
  Integer,
  Float,
  Symbol,
  Call,
  NegSign,
}

fn is_symbol_start(c: char) -> bool {
  matches!(c, 'a'..='z' | 'A'..='Z' | '_' | '+' | '*' | '/' | '=' | '%')
}

fn is_symbol(c: char) -> bool {
  // Numbers can be part of a symbol, but not the first character
  is_symbol_start(c) || c.is_ascii_digit()
}

pub fn parse(input: String) -> Vec<Token> {
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
        State::Start => match c {
          // Match a double-quote
          '"' => State::String,

          '-' => State::NegSign,

          // Match a digit (including negative sign)
          '0'..='9' => {
            accumulator.push(c);
            State::Integer
          }

          // Match a symbol
          '\'' => State::Symbol,

          // Match a call
          c if is_symbol_start(c) => {
            accumulator.push(c);
            State::Call
          }

          // Ignore whitespace
          ' ' => State::Start,

          // Error on everything else
          _ => {
            eprintln!("Error: Unexpected character: {}", c);
            break;
          }
        },
        State::String => match c {
          '"' => State::Start,
          _ => {
            accumulator.push(c);
            State::String
          }
        },
        State::Integer => match c {
          '0'..='9' => {
            accumulator.push(c);
            State::Integer
          }
          '.' => {
            accumulator.push(c);
            State::Float
          }
          _ => State::Start,
        },
        State::Float => match c {
          '0'..='9' => {
            accumulator.push(c);
            State::Float
          }
          _ => State::Start,
        },
        State::Symbol => match c {
          c if is_symbol(c) => {
            accumulator.push(c);
            State::Symbol
          }
          _ => State::Start,
        },
        State::Call => match c {
          c if is_symbol(c) => {
            accumulator.push(c);
            State::Call
          }
          _ => State::Start,
        },
        State::NegSign => match c {
          '0'..='9' => {
            accumulator.push('-');
            accumulator.push(c);
            State::Integer
          }
          _ => State::Start,
        },
      }
    } else {
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
        tokens.push(Token::Symbol('-'.into()));
        accumulator.clear();
      }
      (State::Symbol, State::Start) => {
        let symbol = accumulator.clone();

        tokens.push(match symbol.as_str() {
          "nil" => Token::Nil,
          _ => Token::Symbol(symbol),
        });
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
    i += 1;
  }

  tokens
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_integer() {
    let input = "123".to_string();
    let expected = vec![Token::Integer(123)];

    assert_eq!(parse(input), expected);
  }

  #[test]
  fn test_negative_integer() {
    let input = "-123".to_string();
    let expected = vec![Token::Integer(-123)];

    assert_eq!(parse(input), expected);
  }

  #[test]
  fn test_float() {
    let input = "123.456".to_string();
    let expected = vec![Token::Float(123.456)];

    assert_eq!(parse(input), expected);
  }

  #[test]
  fn test_negative_float() {
    let input = "-123.456".to_string();
    let expected = vec![Token::Float(-123.456)];

    assert_eq!(parse(input), expected);
  }

  #[test]
  fn test_string() {
    let input = "\"Hello, world!\"".to_string();
    let expected = vec![Token::String("Hello, world!".to_string())];

    assert_eq!(parse(input), expected);
  }

  #[test]
  fn test_symbol() {
    let input = "'h3ll0_worl6".to_string();
    let expected = vec![Token::Symbol("h3ll0_worl6".to_string())];

    assert_eq!(parse(input), expected);
  }

  #[test]
  fn test_symbol_simple() {
    let input = "'myVar".to_string();
    let expected = vec![Token::Symbol("myVar".to_string())];

    assert_eq!(parse(input), expected);
  }

  #[test]
  fn test_call_plus() {
    let input = "+".to_string();
    let expected = vec![Token::Call("+".to_string())];

    assert_eq!(parse(input), expected);
  }

  #[test]
  fn test_symbol_var() {
    let input = "myVar".to_string();
    let expected = vec![Token::Call("myVar".to_string())];

    assert_eq!(parse(input), expected);
  }

  #[test]
  fn test_nil() {
    let input = "nil".to_string();
    let expected = vec![Token::Nil];

    assert_eq!(parse(input), expected);
  }

  #[test]
  fn test_multiple() {
    let input = "123 \"Hello, world!\" 'h3ll0_worl6 nil".to_string();
    let expected = vec![
      Token::Integer(123),
      Token::String("Hello, world!".to_string()),
      Token::Symbol("h3ll0_worl6".to_string()),
      Token::Nil,
    ];

    assert_eq!(parse(input), expected);
  }

  #[test]
  fn test_multiple2() {
    let inpuit = "1 'a2 3.0 add \"string hello\" var nil".to_string();
    let expected = vec![
      Token::Integer(1),
      Token::Symbol("a2".to_string()),
      Token::Float(3.0),
      Token::Call("add".to_string()),
      Token::String("string hello".to_string()),
      Token::Call("var".to_string()),
      Token::Nil,
    ];

    assert_eq!(parse(inpuit), expected);
  }
}
