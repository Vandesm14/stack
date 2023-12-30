use crate::{Expr, Lexer, TokenKind, TokenVec};

#[derive(Debug, Clone, PartialEq)]
pub struct Parser<'source> {
  tokens: TokenVec<'source>,
  cursor: usize,
}

impl<'source> Parser<'source> {
  /// Creates a new [`Parser`].
  ///
  /// Prefer [`Parser::reuse`] where possible.
  #[inline]
  pub const fn new(lexer: Lexer<'source>) -> Self {
    Self {
      tokens: TokenVec::new(lexer),
      cursor: 0,
    }
  }

  /// Creates a [`Parser`] by re-using the allocations of an existing one.
  #[inline]
  pub fn reuse(&mut self, lexer: Lexer<'source>) {
    self.tokens.reuse(lexer);
  }

  #[inline]
  pub fn parse(&mut self) -> Vec<Expr> {
    let mut exprs = Vec::new();

    while let Some(expr) = self.parse_expr() {
      exprs.push(expr);
    }

    exprs
  }

  fn parse_expr(&mut self) -> Option<Expr> {
    loop {
      let token = self.tokens.token(self.cursor);
      self.cursor += 1;

      match token.kind {
        TokenKind::Invalid => break Some(Expr::Invalid),
        TokenKind::Eoi => break None,

        TokenKind::Whitespace | TokenKind::Comment => continue,

        TokenKind::Nil => break Some(Expr::Nil),
        TokenKind::Boolean(x) => break Some(Expr::Boolean(x)),
        TokenKind::Integer(x) => break Some(Expr::Integer(x)),
        TokenKind::Float(x) => break Some(Expr::Float(x)),
        TokenKind::String(x) => break Some(Expr::String(x)),

        TokenKind::Ident(x) => break Some(Expr::Call(x)),

        TokenKind::Apostrophe => {
          break self.parse_expr().map(Box::new).map(Expr::Lazy);
        }

        TokenKind::ParenOpen => {
          break Some(
            self.parse_list().map(Expr::List).unwrap_or(Expr::Invalid),
          )
        }
        TokenKind::ParenClose => break Some(Expr::Invalid),

        // TODO: Maybe construct a scope similar to how lists work?
        TokenKind::CurlyOpen => break Some(Expr::ScopePush),
        TokenKind::CurlyClose => break Some(Expr::ScopePop),
        // TODO: Maybe check that these match correctly?
        TokenKind::SquareOpen | TokenKind::SquareClose => continue,

        TokenKind::Fn => break Some(Expr::FnScope(None)),
      }
    }
  }

  fn parse_list(&mut self) -> Option<Vec<Expr>> {
    let mut list = Vec::new();

    loop {
      match self.tokens.token(self.cursor).kind {
        TokenKind::Eoi => break None,
        TokenKind::ParenClose => {
          self.cursor += 1;
          break Some(list);
        }
        _ => match self.parse_expr() {
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
  use crate::{interner::interner, Lexer};

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
  // TODO: This is not checked currently.
  // #[test_case("(]" => vec![Expr::Invalid] ; "invalid block 2")]
  #[test_case("(}" => vec![Expr::Invalid] ; "invalid block 3")]
  #[test_case(
    "false true"
    => vec![Expr::Boolean(false), Expr::Boolean(true)]
    ; "boolean"
  )]
  #[test_case(
    "{1 'var set}"
    => vec![
      Expr::ScopePush,
      Expr::Integer(1),
      Expr::Lazy(Expr::Call(interner().get_or_intern_static("var")).into()),
      Expr::Call(interner().get_or_intern_static("set")),
      Expr::ScopePop,
    ]
    ; "scope"
  )]
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
  fn parse(source: &str) -> Vec<Expr> {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    parser.parse()
  }
}
