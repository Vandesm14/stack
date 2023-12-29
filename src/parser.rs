use core::iter::Peekable;

use crate::{Expr, Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parser;

impl Parser {
  pub fn parse<I>(&self, tokens: &mut I) -> Vec<Expr>
  where
    I: Iterator<Item = Token>,
  {
    let mut tokens = tokens.peekable();
    let mut exprs = Vec::new();

    while let Some(expr) = self.parse_expr(&mut tokens) {
      exprs.push(expr);
    }

    exprs
  }

  fn parse_expr<I>(&self, tokens: &mut Peekable<I>) -> Option<Expr>
  where
    I: Iterator<Item = Token>,
  {
    let token = tokens.next()?;

    match token.kind {
      TokenKind::Invalid => Some(Expr::Invalid),

      TokenKind::Nil => Some(Expr::Nil),
      TokenKind::Boolean(x) => Some(Expr::Boolean(x)),
      TokenKind::Integer(x) => Some(Expr::Integer(x)),
      TokenKind::Float(x) => Some(Expr::Float(x)),
      TokenKind::String(x) => Some(Expr::String(x)),

      TokenKind::Ident(x) => Some(Expr::Call(x)),

      TokenKind::Apostrophe => {
        self.parse_expr(tokens).map(Box::new).map(Expr::Lazy)
      }

      TokenKind::ParenStart => Some(
        self
          .parse_list(tokens)
          .map(Expr::List)
          .unwrap_or(Expr::Invalid),
      ),
      TokenKind::ParenEnd => Some(Expr::Invalid),

      // TODO: Maybe construct a scope similar to how lists work?
      TokenKind::CurlyStart => Some(Expr::ScopePush),
      TokenKind::CurlyEnd => Some(Expr::ScopePop),

      TokenKind::Fn => Some(Expr::FnScope(None)),
    }
  }

  fn parse_list<I>(&self, tokens: &mut Peekable<I>) -> Option<Vec<Expr>>
  where
    I: Iterator<Item = Token>,
  {
    let mut list = Vec::new();

    loop {
      match tokens.peek().map(|token| token.kind) {
        None => break None,
        Some(TokenKind::ParenEnd) => {
          // If this panics, it's a bug.
          tokens.next().unwrap();
          break Some(list);
        }
        Some(_) => match self.parse_expr(tokens) {
          Some(expr) => list.push(expr),
          None => {
            list.push(Expr::Invalid);
            break Some(list);
          }
        },
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::Lexer;

  use super::*;

  use test_case::test_case;

  #[test_case(
    "(1 2 3)"
    => vec![Expr::List(vec![
      Expr::Integer(1),
      Expr::Integer(2),
      Expr::Integer(3),
    ])]
    ; "block"
  )]
  #[test_case(
    "(1 2 3) 4 5 6"
    => vec![
      Expr::List(vec![
        Expr::Integer(1),
        Expr::Integer(2),
        Expr::Integer(3),
      ]),
      Expr::Integer(4),
      Expr::Integer(5),
      Expr::Integer(6),
    ]
    ; "block before exprs"
  )]
  #[test_case(
    "1 2 3 (4 5 6)"
    => vec![
      Expr::Integer(1),
      Expr::Integer(2),
      Expr::Integer(3),
      Expr::List(vec![
        Expr::Integer(4),
        Expr::Integer(5),
        Expr::Integer(6),
      ]),
    ]
    ; "block after exprs"
  )]
  #[test_case(
    "1 2 (3 4 5) 6"
    => vec![
      Expr::Integer(1),
      Expr::Integer(2),
      Expr::List(vec![
        Expr::Integer(3),
        Expr::Integer(4),
        Expr::Integer(5),
      ]),
      Expr::Integer(6),
    ]
    ; "block between exprs"
  )]
  #[test_case(
    "(1 (2 3) 4)"
    => vec![Expr::List(vec![
      Expr::Integer(1),
      Expr::List(vec![
        Expr::Integer(2),
        Expr::Integer(3),
      ]),
      Expr::Integer(4),
    ])]
    ; "nested blocks"
  )]
  #[test_case("(" => vec![Expr::Invalid] ; "invalid block 0")]
  #[test_case(")" => vec![Expr::Invalid] ; "invalid block 1")]
  #[test_case("(]" => vec![Expr::Invalid] ; "invalid block 2")]
  #[test_case("(}" => vec![Expr::Invalid] ; "invalid block 3")]
  #[test_case(
    "false true"
    => vec![Expr::Boolean(false), Expr::Boolean(true)]
    ; "boolean"
  )]
  // TODO: Implement a nice way to test with Spurs.
  // #[test_case(
  //   "{1 'var set}"
  //   => vec![
  //     Expr::ScopePush,
  //     Expr::Integer(1),
  //     Expr::Lazy(Expr::Call("var".into()).into()),
  //     Expr::Call("set".into()),
  //     Expr::ScopePop,
  //   ]
  //   ; "scope"
  // )]
  #[test_case(
    "'(1 2 3)"
    => vec![Expr::Lazy(Expr::List(vec![
      Expr::Integer(1),
      Expr::Integer(2),
      Expr::Integer(3),
    ]).into())]
    ; "lazy block"
  )]
  #[test_case(
    "'(1 '(2) 3)"
    => vec![Expr::Lazy(Expr::List(vec![
      Expr::Integer(1),
      Expr::Lazy(Expr::List(vec![Expr::Integer(2)]).into()),
      Expr::Integer(3),
    ]).into())]
    ; "lazy nested blocks"
  )]
  #[test_case(
    "''1"
    => vec![Expr::Lazy(Expr::Lazy(Expr::Integer(1).into()).into())]
    ; "lazy lazy expr"
  )]
  fn parse(code: impl AsRef<str>) -> Vec<Expr> {
    let lexer = Lexer::new();
    let mut tokens = lexer.lex(code.as_ref());
    Parser.parse(&mut tokens)
  }
}
