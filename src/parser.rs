use crate::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
  Integer(i64),
  Float(f64),
  String(String),
  Symbol(String),
  Call(String),
  Block(Vec<Expr>),
  List(Vec<Expr>),
  Nil,
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
