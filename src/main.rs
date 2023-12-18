#[derive(Debug, Clone, PartialEq)]
enum Token {
  Integer(usize),
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

fn main() {
  let mut state = State::Start;
  let mut tokens: Vec<Token> = vec![];

  let input = "1 a2 3.0 add \"string hello\" var"
    .chars()
    .collect::<Vec<_>>();
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
        tokens.push(Token::Integer(accumulator.parse::<usize>().unwrap()));
        accumulator.clear();
      }
      (State::Float, State::Start) => {
        tokens.push(Token::Float(accumulator.parse::<f64>().unwrap()));
        accumulator.clear();
      }
      (State::Symbol, State::Start) => {
        tokens.push(Token::Symbol(accumulator.clone()));
        accumulator.clear();
      }
      _ => {}
    };

    state = new_state;
    i += 1;
  }

  println!("{:?}", tokens);
}

// ==================================================
//
// use rustyline::error::ReadlineError;
// use rustyline::{DefaultEditor, Result};

// enum Value {
//   Integer(usize),
//   Float(f64),
//   String(String),
//   Symbol(String),
//   Nil,
// }

// struct Program {
//   stack: Vec<Value>,
// }

// impl Program {
//   fn new() -> Self {
//     Self { stack: vec![] }
//   }
// }

// fn main() -> Result<()> {
//   // `()` can be used when no completer is required
//   let mut rl = DefaultEditor::new()?;

//   loop {
//     let readline = rl.readline(">> ");
//     match readline {
//       Ok(line) => {
//         rl.add_history_entry(line.as_str());
//         println!("Line: {}", line);
//       }
//       Err(ReadlineError::Interrupted) => {
//         println!("CTRL-C");
//         break;
//       }
//       Err(ReadlineError::Eof) => {
//         println!("CTRL-D");
//         break;
//       }
//       Err(err) => {
//         println!("Error: {:?}", err);
//         break;
//       }
//     }
//   }

//   Ok(())
// }
