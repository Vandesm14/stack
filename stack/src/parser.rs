use core::{fmt, str::FromStr};
use std::rc::Rc;

use crate::{
  expr::{Expr, ExprInfo, ExprKind, FnIdent},
  intrinsic::Intrinsic,
  lexer::{Lexer, Span, Token, TokenKind},
  source::Source,
  symbol::Symbol,
};

#[derive(Debug)]
pub struct Parser {
  lexer: Lexer,
}

impl Parser {
  #[inline]
  pub const fn new(lexer: Lexer) -> Self {
    Self { lexer }
  }

  #[inline]
  pub fn source(&self) -> Rc<Source> {
    self.lexer.source()
  }

  pub fn parse(mut self) -> Result<Vec<Expr>, ParseError> {
    let mut exprs = Vec::new();

    loop {
      let next_token = self.lexer.next();

      match self.next(next_token) {
        Ok(Some(expr)) => exprs.push(expr),
        Ok(None) => break Ok(exprs),
        Err(err) => break Err(err),
      }
    }
  }

  fn next(&mut self, token: Token) -> Result<Option<Expr>, ParseError> {
    let source = self.source();

    match token.kind {
      TokenKind::Invalid | TokenKind::RightParen => Err(ParseError {
        source,
        token,
        reason: ParseErrorReason::UnexpectedToken,
      }),

      TokenKind::Eof => Ok(None),
      TokenKind::Whitespace | TokenKind::Comment => {
        let next_token = self.lexer.next();
        self.next(next_token)
      }

      TokenKind::Apostrophe => {
        let next_token = self.lexer.next();

        match next_token.kind {
          TokenKind::Eof => Err(ParseError {
            source,
            token: next_token,
            reason: ParseErrorReason::UnexpectedToken,
          }),
          _ => match self.next(next_token)? {
            Some(expr) => Ok(Some(Expr {
              kind: ExprKind::Lazy(Box::new(expr)),
              info: Some(ExprInfo::Source {
                source,
                span: Span {
                  start: token.span.start,
                  end: next_token.span.end,
                },
              }),
            })),
            None => Err(ParseError {
              source,
              token: next_token,
              reason: ParseErrorReason::UnexpectedToken,
            }),
          },
        }
      }
      TokenKind::LeftParen => {
        let mut list = Vec::new();

        loop {
          let next_token = self.lexer.next();

          match next_token.kind {
            TokenKind::RightParen => {
              break Ok(Some(Expr {
                kind: ExprKind::List(list),
                info: Some(ExprInfo::Source {
                  source,
                  span: Span {
                    start: token.span.start,
                    end: next_token.span.end,
                  },
                }),
              }));
            }
            _ => match self.next(next_token)? {
              Some(expr) => list.push(expr),
              None => {
                break Err(ParseError {
                  source,
                  token: next_token,
                  reason: ParseErrorReason::UnexpectedToken,
                });
              }
            },
          }
        }
      }
      // TODO: Check that there are matching pairs for sanity sake. Removal
      //       would be the better option if possible, since you can use
      //       newlines to achive the same effect in a less cluttered way.
      TokenKind::LeftSquare | TokenKind::RightSquare => {
        let next_token = self.lexer.next();
        self.next(next_token)
      }

      TokenKind::Integer => {
        let slice = &source.source()[token.span.start..token.span.end];
        let integer = slice.parse().map_err(|_| ParseError {
          source,
          token,
          reason: ParseErrorReason::InvalidInteger,
        })?;

        Ok(Some(Expr {
          kind: ExprKind::Integer(integer),
          info: Some(ExprInfo::Source {
            source: self.source(),
            span: token.span,
          }),
        }))
      }
      TokenKind::Float => {
        let slice = &source.source()[token.span.start..token.span.end];
        let float = slice.parse().map_err(|_| ParseError {
          source,
          token,
          reason: ParseErrorReason::InvalidFloat,
        })?;

        Ok(Some(Expr {
          kind: ExprKind::Float(float),
          info: Some(ExprInfo::Source {
            source: self.source(),
            span: token.span,
          }),
        }))
      }
      TokenKind::String => {
        // Discard the quotation marks from the slice.
        let slice = &source.source()[token.span.start + 1..token.span.end - 1];

        Ok(Some(Expr {
          kind: ExprKind::String(
            slice
              .replace("\\n", "\n")
              .replace("\\t", "\t")
              .replace("\\r", "\r")
              .replace("\\0", "\0"),
          ),
          info: Some(ExprInfo::Source {
            source,
            span: token.span,
          }),
        }))
      }
      TokenKind::Symbol => {
        let slice = &source.source()[token.span.start..token.span.end];

        let kind = Intrinsic::from_str(slice)
          .map(ExprKind::Intrinsic)
          .unwrap_or_else(|_| match slice {
            "nil" => ExprKind::Nil,
            "true" => ExprKind::Boolean(true),
            "false" => ExprKind::Boolean(false),
            "fn" => ExprKind::Fn(FnIdent {
              scoped: true,
              ..Default::default()
            }),
            "fn!" => ExprKind::Fn(FnIdent::default()),

            slice => ExprKind::Symbol(Symbol::from_ref(slice)),
          });

        Ok(Some(Expr {
          kind,
          info: Some(ExprInfo::Source {
            source,
            span: token.span,
          }),
        }))
      }
    }
  }
}

#[derive(Clone)]
pub struct ParseError {
  pub source: Rc<Source>,
  pub token: Token,
  pub reason: ParseErrorReason,
}

impl PartialEq for ParseError {
  fn eq(&self, other: &Self) -> bool {
    self.token == other.token
      && self.reason == other.reason
      && self.source.name() == other.source.name()
      && self.source.source() == other.source.source()
  }
}

impl std::error::Error for ParseError {}

impl fmt::Debug for ParseError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("ParseError")
      .field("token", &self.token)
      .field("reason", &self.reason)
      .finish_non_exhaustive()
  }
}

impl fmt::Display for ParseError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // TODO: This should output line and column numbers instead of the span.
    write!(
      f,
      "{} {} '{}' at {}:{}",
      self.reason,
      self.token.kind,
      &self.source.source()[self.token.span.start..self.token.span.end],
      self.source.name(),
      self
        .source
        .location(self.token.span.start)
        .map(|x| x.to_string())
        .unwrap_or_else(|| "?:?".into())
    )
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParseErrorReason {
  UnexpectedToken,
  InvalidInteger,
  InvalidFloat,
}

impl fmt::Display for ParseErrorReason {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::UnexpectedToken => write!(f, "unexpected token"),
      Self::InvalidInteger => write!(f, "invalid integer"),
      Self::InvalidFloat => write!(f, "invalid float"),
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use test_case::case;

  fn fill_info_none(mut expr: Expr) -> Expr {
    expr.info = None;
    expr.kind = match expr.kind {
      ExprKind::Lazy(x) => ExprKind::Lazy(Box::new(fill_info_none(*x))),
      ExprKind::List(x) => {
        ExprKind::List(x.into_iter().map(fill_info_none).collect())
      }
      kind => kind,
    };

    expr
  }

  // TODO: Test that the info is correct as well.

  #[case("" => Ok(vec![]) ; "empty")]
  #[case("123" => Ok(vec![Expr { kind: ExprKind::Integer(123), info: None }]) ; "integer")]
  #[case("-123" => Ok(vec![Expr { kind: ExprKind::Integer(-123), info: None }]) ; "negative integer")]
  #[case("1.2" => Ok(vec![Expr { kind: ExprKind::Float(1.2), info: None }]) ; "float")]
  #[case("-1.2" => Ok(vec![Expr { kind: ExprKind::Float(-1.2), info: None }]) ; "negative float")]
  #[case("123 -1.2" => Ok(vec![Expr { kind: ExprKind::Integer(123), info: None }, Expr { kind: ExprKind::Float(-1.2), info: None }]) ; "integer whitespace negative float")]
  #[case("'123" => Ok(vec![Expr { kind: ExprKind::Lazy(Box::new(Expr { kind: ExprKind::Integer(123), info: None })), info: None }]) ; "lazy integer")]
  #[case("'-123" => Ok(vec![Expr { kind: ExprKind::Lazy(Box::new(Expr { kind: ExprKind::Integer(-123), info: None })), info: None }]) ; "lazy negative integer")]
  #[case("()" => Ok(vec![Expr { kind: ExprKind::List(vec![]), info: None }]) ; "empty list")]
  #[case("'()" => Ok(vec![Expr { kind: ExprKind::Lazy(Box::new(Expr { kind: ExprKind::List(vec![]), info: None })), info: None }]) ; "lazy empty list")]
  #[case("(true)" => Ok(vec![Expr { kind: ExprKind::List(vec![Expr { kind: ExprKind::Boolean(true), info: None }]), info: None }]) ; "list of boolean")]
  #[case("'(false)" => Ok(vec![Expr { kind: ExprKind::Lazy(Box::new(Expr { kind: ExprKind::List(vec![Expr { kind: ExprKind::Boolean(false), info: None }]), info: None })), info: None }]) ; "lazy list of boolean")]
  #[case("(true -123)" => Ok(vec![Expr { kind: ExprKind::List(vec![Expr { kind: ExprKind::Boolean(true), info: None }, Expr { kind: ExprKind::Integer(-123), info: None }]), info: None }]) ; "list of boolean and negative integer")]
  #[case("'(false h-llo)" => Ok(vec![Expr { kind: ExprKind::Lazy(Box::new(Expr { kind: ExprKind::List(vec![Expr { kind: ExprKind::Boolean(false), info: None }, Expr { kind: ExprKind::Symbol(Symbol::from_ref("h-llo")), info: None }]), info: None })), info: None }]) ; "lazy list of boolean and symbol")]
  fn parser(source: &str) -> Result<Vec<Expr>, ParseError> {
    let source = Rc::new(Source::new("", source));
    let lexer = Lexer::new(source);
    let parser = Parser::new(lexer);

    parser
      .parse()
      .map(|x| x.into_iter().map(fill_info_none).collect())
  }
}
