use core::fmt;

use crate::Token;

#[derive(Debug, Clone, Default)]
pub enum Expr {
  Integer(i64),
  Float(f64),

  String(String),
  Boolean(bool),

  Call(String),

  /// A block is lazy. It only gets evaluated when it's called.
  /// This is useful for things like if statements.
  /// `(1 2 3)` is a block
  Block(Vec<Expr>),

  /// Lists are eager. They get evaluated before being pushed to the stack.
  /// `[1 2 3]` is a list
  List(Vec<Expr>),

  #[default]
  Nil,
}

impl fmt::Display for Expr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Expr::Integer(i) => write!(f, "{}", i),
      Expr::Float(float) => write!(f, "{}", float),

      Expr::String(s) => write!(f, "\"{}\"", s),
      Expr::Boolean(b) => write!(f, "{}", b),

      Expr::Call(s) => write!(f, "{}", s),

      Expr::Block(b) => write!(
        f,
        "({})",
        b.iter()
          .map(|e| e.to_string())
          .collect::<Vec<String>>()
          .join(" ")
      ),
      Expr::List(l) => write!(
        f,
        "[{}]",
        l.iter()
          .map(|e| e.to_string())
          .collect::<Vec<String>>()
          .join(" ")
      ),

      Expr::Nil => write!(f, "nil"),
    }
  }
}

impl From<Option<Expr>> for Expr {
  fn from(value: Option<Expr>) -> Self {
    match value {
      Option::None => Expr::Nil,
      Option::Some(value) => value,
    }
  }
}

impl PartialEq for Expr {
  fn eq(&self, other: &Expr) -> bool {
    match (self, other) {
      // Same types
      (Expr::Float(a), Expr::Float(b)) => a == b,
      (Expr::Integer(a), Expr::Integer(b)) => a == b,

      (Expr::String(a), Expr::String(b)) => a == b,
      (Expr::Boolean(a), Expr::Boolean(b)) => a == b,

      (Expr::Call(a), Expr::Call(b)) => a == b,

      (Expr::Block(a), Expr::Block(b)) => a == b,
      (Expr::List(a), Expr::List(b)) => a == b,

      (Expr::Nil, Expr::Nil) => true,

      // Different types
      (Expr::Float(a), Expr::Integer(b)) => {
        let b = *b as f64;
        a == &b
      }
      (Expr::Integer(a), Expr::Float(b)) => {
        let a = *a as f64;
        &a == b
      }

      (Expr::Integer(a), Expr::Boolean(b)) => {
        let a = *a != 0;
        a == *b
      }
      (Expr::Boolean(a), Expr::Integer(b)) => {
        let b = *b != 0;
        *a == b
      }
      _ => false,
    }
  }
}

impl PartialOrd for Expr {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    match (self, other) {
      // Same types
      (Expr::Integer(a), Expr::Integer(b)) => a.partial_cmp(b),
      (Expr::Float(a), Expr::Float(b)) => a.partial_cmp(b),

      (Expr::String(a), Expr::String(b)) => a.partial_cmp(b),
      (Expr::Boolean(a), Expr::Boolean(b)) => a.partial_cmp(b),

      (Expr::Call(a), Expr::Call(b)) => a.partial_cmp(b),

      (Expr::Block(a), Expr::Block(b)) => a.partial_cmp(b),
      (Expr::List(a), Expr::List(b)) => a.partial_cmp(b),

      (Expr::Nil, Expr::Nil) => Some(std::cmp::Ordering::Equal),

      // Different types
      (Expr::Float(a), Expr::Integer(b)) => {
        let b = *b as f64;
        a.partial_cmp(&b)
      }
      (Expr::Integer(a), Expr::Float(b)) => {
        let a = *a as f64;
        a.partial_cmp(b)
      }

      _ => None,
    }
  }
}

impl Expr {
  pub fn is_truthy(&self) -> bool {
    match self {
      Expr::Nil => false,
      Expr::Boolean(b) => *b,
      Expr::Integer(i) => *i != 0,
      Expr::Float(f) => *f != 0.0,
      _ => true,
    }
  }

  pub fn is_nil(&self) -> bool {
    matches!(self, Expr::Nil)
  }

  pub fn type_of(&self) -> String {
    match self {
      Expr::Integer(_) => "integer".to_owned(),
      Expr::Float(_) => "float".to_owned(),

      Expr::String(_) => "string".to_owned(),
      Expr::Boolean(_) => "boolean".to_owned(),

      Expr::Call(_) => "call".to_owned(),

      Expr::Block(_) => "block".to_owned(),
      Expr::List(_) => "list".to_owned(),

      Expr::Nil => "nil".to_owned(),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListMode {
  Paren,
  Bracket,
}

pub fn parse(tokens: Vec<Token>) -> Vec<Expr> {
  let mut blocks: Vec<Vec<Expr>> = vec![Vec::new()];
  let mut list_mode: Vec<ListMode> = Vec::new();

  for token in tokens {
    match token {
      Token::Integer(i) => blocks.last_mut().unwrap().push(Expr::Integer(i)),
      Token::Float(f) => blocks.last_mut().unwrap().push(Expr::Float(f)),
      Token::String(s) => blocks.last_mut().unwrap().push(Expr::String(s)),
      Token::Call(s) => match s.as_str() {
        "true" => blocks.last_mut().unwrap().push(Expr::Boolean(true)),
        "false" => blocks.last_mut().unwrap().push(Expr::Boolean(false)),
        _ => blocks.last_mut().unwrap().push(Expr::Call(s)),
      },
      Token::Nil => blocks.last_mut().unwrap().push(Expr::Nil),

      Token::ParenStart | Token::BracketStart => {
        blocks.push(Vec::new());

        match token {
          Token::ParenStart => list_mode.push(ListMode::Paren),
          Token::BracketStart => list_mode.push(ListMode::Bracket),
          _ => {}
        }
      }
      Token::ParenEnd => {
        if let Some(ListMode::Paren) = list_mode.pop() {
          let block = blocks.pop().unwrap();
          blocks.last_mut().unwrap().push(Expr::Block(block));
        } else {
          eprintln!("Mismatched brackets");
          return vec![];
        }
      }
      Token::BracketEnd => {
        if let Some(ListMode::Bracket) = list_mode.pop() {
          let block = blocks.pop().unwrap();
          blocks.last_mut().unwrap().push(Expr::List(block));
        } else {
          eprintln!("Mismatched brackets");
          return vec![];
        }
      }
    };
  }

  if blocks.len() != 1 {
    eprintln!("Unbalanced blocks: {:?}", blocks);
    return vec![];
  }

  blocks.last().unwrap().clone()
}

#[cfg(test)]
mod tests {
  use super::*;

  mod parsing {
    use super::*;

    #[test]
    fn implicit_block() {
      let tokens = crate::lex("(1 2 3)");
      let expected = vec![Expr::Block(vec![
        Expr::Integer(1),
        Expr::Integer(2),
        Expr::Integer(3),
      ])];

      assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn block_at_beginning() {
      let tokens = crate::lex("(1 2 3) 4 5 6");
      let expected = vec![
        Expr::Block(vec![Expr::Integer(1), Expr::Integer(2), Expr::Integer(3)]),
        Expr::Integer(4),
        Expr::Integer(5),
        Expr::Integer(6),
      ];

      assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn nested_blocks() {
      let tokens = crate::lex("(1 (2 3) 4)");
      let expected = vec![Expr::Block(vec![
        Expr::Integer(1),
        Expr::Block(vec![Expr::Integer(2), Expr::Integer(3)]),
        Expr::Integer(4),
      ])];

      assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn blocks_and_lists() {
      let tokens = crate::lex("(1 [2 3] 4)");
      let expected = vec![Expr::Block(vec![
        Expr::Integer(1),
        Expr::List(vec![Expr::Integer(2), Expr::Integer(3)]),
        Expr::Integer(4),
      ])];

      assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn fail_for_only_start_paren() {
      let tokens = crate::lex("(");
      let exprs = parse(tokens);
      assert_eq!(exprs, vec![]);
    }

    #[test]
    fn fail_for_only_end_paren() {
      let tokens = crate::lex(")");
      let exprs = parse(tokens);
      assert_eq!(exprs, vec![]);
    }

    #[test]
    fn fail_for_mismatched_parens() {
      let tokens = crate::lex("(1 2 3]");
      let exprs = parse(tokens);
      assert_eq!(exprs, vec![]);
    }

    #[test]
    fn booleans() {
      let tokens = crate::lex("true false");
      let expected = vec![Expr::Boolean(true), Expr::Boolean(false)];

      assert_eq!(parse(tokens), expected);
    }
  }

  mod equality {
    use super::*;

    mod same_types {
      use super::*;

      #[test]
      fn integer_to_integer() {
        let a = Expr::Integer(1);
        let b = Expr::Integer(1);
        assert_eq!(a, b);

        let a = Expr::Integer(1);
        let b = Expr::Integer(2);
        assert_ne!(a, b);
      }

      #[test]
      fn float_to_float() {
        let a = Expr::Float(1.0);
        let b = Expr::Float(1.0);
        assert_eq!(a, b);

        let a = Expr::Float(1.0);
        let b = Expr::Float(1.1);
        assert_ne!(a, b);
      }

      #[test]
      fn string_to_string() {
        let a = Expr::String("hello".to_owned());
        let b = Expr::String("hello".to_owned());
        assert_eq!(a, b);

        let a = Expr::String("hello".to_owned());
        let b = Expr::String("world".to_owned());
        assert_ne!(a, b);
      }

      #[test]
      fn boolean_to_boolean() {
        let a = Expr::Boolean(true);
        let b = Expr::Boolean(true);
        assert_eq!(a, b);

        let a = Expr::Boolean(true);
        let b = Expr::Boolean(false);
        assert_ne!(a, b);
      }

      #[test]
      fn call_to_call() {
        let a = Expr::Call("hello".to_owned());
        let b = Expr::Call("hello".to_owned());
        assert_eq!(a, b);

        let a = Expr::Call("hello".to_owned());
        let b = Expr::Call("world".to_owned());
        assert_ne!(a, b);
      }

      #[test]
      fn block_to_block() {
        let a = Expr::Block(vec![Expr::Integer(1), Expr::Integer(2)]);
        let b = Expr::Block(vec![Expr::Integer(1), Expr::Integer(2)]);
        assert_eq!(a, b);

        let a = Expr::Block(vec![Expr::Integer(1), Expr::Integer(2)]);
        let b = Expr::Block(vec![Expr::Integer(1), Expr::Integer(3)]);
        assert_ne!(a, b);
      }

      #[test]
      fn list_to_list() {
        let a = Expr::List(vec![Expr::Integer(1), Expr::Integer(2)]);
        let b = Expr::List(vec![Expr::Integer(1), Expr::Integer(2)]);
        assert_eq!(a, b);

        let a = Expr::List(vec![Expr::Integer(1), Expr::Integer(2)]);
        let b = Expr::List(vec![Expr::Integer(1), Expr::Integer(3)]);
        assert_ne!(a, b);
      }

      #[test]
      fn nil_to_nil() {
        let a = Expr::Nil;
        let b = Expr::Nil;
        assert_eq!(a, b);
      }
    }

    mod different_types {
      use super::*;

      #[test]
      fn integer_to_float() {
        let a = Expr::Integer(1);
        let b = Expr::Float(1.0);
        assert_eq!(a, b);

        let a = Expr::Integer(1);
        let b = Expr::Float(1.1);
        assert_ne!(a, b);
      }

      #[test]
      fn float_to_integer() {
        let a = Expr::Float(1.0);
        let b = Expr::Integer(1);
        assert_eq!(a, b);

        let a = Expr::Float(1.1);
        let b = Expr::Integer(1);
        assert_ne!(a, b);
      }

      #[test]
      fn integer_to_boolean() {
        let a = Expr::Integer(1);
        let b = Expr::Boolean(true);
        assert_eq!(a, b);

        let a = Expr::Integer(0);
        let b = Expr::Boolean(false);
        assert_eq!(a, b);
      }

      #[test]
      fn boolean_to_integer() {
        let a = Expr::Boolean(true);
        let b = Expr::Integer(1);
        assert_eq!(a, b);

        let a = Expr::Boolean(false);
        let b = Expr::Integer(0);
        assert_eq!(a, b);
      }
    }
  }

  mod ordering {
    use super::*;

    mod same_types {
      use super::*;

      #[test]
      fn integer_to_integer() {
        let a = Expr::Integer(1);
        let b = Expr::Integer(1);
        assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Equal));

        let a = Expr::Integer(1);
        let b = Expr::Integer(2);
        assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Less));

        let a = Expr::Integer(2);
        let b = Expr::Integer(1);
        assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Greater));
      }

      #[test]
      fn float_to_float() {
        let a = Expr::Float(1.0);
        let b = Expr::Float(1.0);
        assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Equal));

        let a = Expr::Float(1.0);
        let b = Expr::Float(1.1);
        assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Less));

        let a = Expr::Float(1.1);
        let b = Expr::Float(1.0);
        assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Greater));
      }
    }

    mod different_types {
      use super::*;

      #[test]
      fn integer_to_float() {
        let a = Expr::Integer(1);
        let b = Expr::Float(1.0);
        assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Equal));

        let a = Expr::Integer(1);
        let b = Expr::Float(1.1);
        assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Less));

        let a = Expr::Integer(2);
        let b = Expr::Float(1.0);
        assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Greater));
      }

      #[test]
      fn float_to_integer() {
        let a = Expr::Float(1.0);
        let b = Expr::Integer(1);
        assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Equal));

        let a = Expr::Float(1.1);
        let b = Expr::Integer(1);
        assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Greater));

        let a = Expr::Float(1.0);
        let b = Expr::Integer(2);
        assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Less));
      }
    }
  }
}
