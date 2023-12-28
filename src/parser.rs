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
