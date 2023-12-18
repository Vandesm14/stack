#[derive(Debug, Clone, PartialEq)]
pub enum Token {
  Integer(i64),
  Float(f64),
  String(String),
  Symbol(String),
  Nil,
}

#[derive(Debug, Clone, Copy)]
enum State {
  Start,
  String,
  Integer,
  Float,
  Symbol,
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
      let new_state = match state {
        State::Start => match c {
          // Match a double-quote
          '"' => State::String,

          // Match a digit (including negative sign)
          '0'..='9' | '-' => {
            accumulator.push(c);
            State::Integer
          }

          // Match a symbol
          'a'..='z' | 'A'..='Z' | '_' => {
            accumulator.push(c);
            State::Symbol
          }

          // Ignore whitespace
          ' ' => State::Start,

          // Error on everything else
          _ => {
            println!("Error: Unexpected character: {}", c);
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
          'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
            accumulator.push(c);
            State::Symbol
          }
          _ => State::Start,
        },
      };

      println!("Current: {:?}, New: {:?}, Char: {}", state, new_state, c);

      new_state
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
      (State::Symbol, State::Start) => {
        let symbol = accumulator.clone();

        tokens.push(match symbol.as_str() {
          "nil" => Token::Nil,
          _ => Token::Symbol(symbol),
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
    let input = "h3ll0_worl6".to_string();
    let expected = vec![Token::Symbol("h3ll0_worl6".to_string())];

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
    let input = "123 \"Hello, world!\" h3ll0_worl6 nil".to_string();
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
    let inpuit = "1 a2 3.0 add \"string hello\" var nil".to_string();
    let expected = vec![
      Token::Integer(1),
      Token::Symbol("a2".to_string()),
      Token::Float(3.0),
      Token::Symbol("add".to_string()),
      Token::String("string hello".to_string()),
      Token::Symbol("var".to_string()),
      Token::Nil,
    ];

    assert_eq!(parse(inpuit), expected);
  }
}
