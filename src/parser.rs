use crate::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
  Integer(i64),
  Float(f64),

  String(String),

  Symbol(String),
  Call(String),

  /// A block is lazy. It only gets evaluated when it's called.
  /// This is useful for things like if statements.
  Block(Vec<Expr>),

  /// Lists are eager. They get evaluated before being pushed to the stack.
  List(Vec<Expr>),

  Nil,
}

impl From<Option<Expr>> for Expr {
  fn from(value: Option<Expr>) -> Self {
    match value {
      Option::None => Expr::Nil,
      Option::Some(value) => value,
    }
  }
}

pub fn parse(tokens: Vec<Token>) -> Vec<Expr> {
  let mut blocks: Vec<Vec<Expr>> = vec![Vec::new()];

  for token in tokens {
    match token {
      Token::Integer(i) => blocks.last_mut().unwrap().push(Expr::Integer(i)),
      Token::Float(f) => blocks.last_mut().unwrap().push(Expr::Float(f)),
      Token::String(s) => blocks.last_mut().unwrap().push(Expr::String(s)),
      Token::Symbol(s) => blocks.last_mut().unwrap().push(Expr::Symbol(s)),
      Token::Call(s) => blocks.last_mut().unwrap().push(Expr::Call(s)),
      Token::Nil => blocks.last_mut().unwrap().push(Expr::Nil),

      Token::ParenStart | Token::BracketStart => {
        blocks.push(Vec::new());
      }
      Token::ParenEnd => {
        let block = blocks.pop().unwrap();
        blocks.last_mut().unwrap().push(Expr::Block(block));
      }
      Token::BracketEnd => {
        let block = blocks.pop().unwrap();
        blocks.last_mut().unwrap().push(Expr::List(block));
      }
    };
  }

  if blocks.len() != 1 {
    panic!("Unbalanced blocks: {:?}", blocks);
  }

  blocks.last().unwrap().clone()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn implicit_block() {
    let tokens = crate::lex("(1 2 3)".to_owned());
    let expected = vec![Expr::Block(vec![
      Expr::Integer(1),
      Expr::Integer(2),
      Expr::Integer(3),
    ])];

    assert_eq!(parse(tokens), expected);
  }

  #[test]
  fn block_at_beginning() {
    let tokens = crate::lex("(1 2 3) 4 5 6".to_owned());
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
    let tokens = crate::lex("(1 (2 3) 4)".to_owned());
    let expected = vec![Expr::Block(vec![
      Expr::Integer(1),
      Expr::Block(vec![Expr::Integer(2), Expr::Integer(3)]),
      Expr::Integer(4),
    ])];

    assert_eq!(parse(tokens), expected);
  }

  #[test]
  fn blocks_and_lists() {
    let tokens = crate::lex("(1 [2 3] 4)".to_owned());
    let expected = vec![Expr::Block(vec![
      Expr::Integer(1),
      Expr::List(vec![Expr::Integer(2), Expr::Integer(3)]),
      Expr::Integer(4),
    ])];

    assert_eq!(parse(tokens), expected);
  }
}
