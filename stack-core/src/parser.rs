use core::fmt;

use crate::{
  expr::{Expr, ExprInfo, ExprKind, FnIdent},
  lexer::{Lexer, Span, Token, TokenKind},
  source::{Location, Source},
  symbol::Symbol,
};

pub fn parse(lexer: &mut Lexer) -> Result<Vec<Expr>, ParseError> {
  let mut exprs = Vec::new();

  loop {
    let token = lexer.peek();

    match token.kind {
      TokenKind::Eof => break Ok(exprs),
      _ => exprs.push(parse_expr(lexer)?),
    }
  }
}

fn parse_expr(lexer: &mut Lexer) -> Result<Expr, ParseError> {
  let source = lexer.source();
  let token = lexer.next();

  match token.kind {
    TokenKind::Invalid | TokenKind::Eof | TokenKind::RightParen => {
      Err(ParseError {
        source,
        kind: ParseErrorKind::UnexpectedToken(token),
      })
    }

    TokenKind::Apostrophe => {
      let next_token = lexer.peek();
      let expr = parse_expr(lexer)?;

      Ok(Expr {
        kind: ExprKind::Lazy(Box::new(expr)),
        info: Some(ExprInfo {
          source,
          span: Span {
            start: token.span.start,
            end: next_token.span.end,
          },
        }),
      })
    }
    TokenKind::LeftParen => {
      let (list, end_span) = parse_list(lexer)?;

      Ok(Expr {
        kind: ExprKind::List(list),
        info: Some(ExprInfo {
          source,
          span: Span {
            start: token.span.start,
            end: end_span.end,
          },
        }),
      })
    }

    TokenKind::Integer => {
      let slice = &source.source()[token.span.start..token.span.end];
      let literal = slice.parse().map_err(|_| ParseError {
        source: source.clone(),
        kind: ParseErrorKind::InvalidLiteral(token),
      })?;

      Ok(Expr {
        kind: ExprKind::Integer(literal),
        info: Some(ExprInfo {
          source,
          span: token.span,
        }),
      })
    }
    TokenKind::Float => {
      let slice = &source.source()[token.span.start..token.span.end];
      let literal = slice.parse().map_err(|_| ParseError {
        source: source.clone(),
        kind: ParseErrorKind::InvalidLiteral(token),
      })?;

      Ok(Expr {
        kind: ExprKind::Float(literal),
        info: Some(ExprInfo {
          source,
          span: token.span,
        }),
      })
    }
    TokenKind::String => {
      //   // Discard the quotation marks from the slice.
      let slice = &source.source()[token.span.start + 1..token.span.end - 1];

      Ok(Expr {
        kind: ExprKind::String(
          slice
            .replace("\\n", "\n")
            .replace("\\t", "\t")
            .replace("\\r", "\r")
            .replace("\\0", "\0"),
        ),
        info: Some(ExprInfo {
          source,
          span: token.span,
        }),
      })
    }
    TokenKind::Symbol => {
      let slice = &source.source()[token.span.start..token.span.end];

      Ok(Expr {
        kind: match slice {
          "nil" => ExprKind::Nil,
          "true" => ExprKind::Boolean(true),
          "false" => ExprKind::Boolean(false),
          "fn" => ExprKind::Fn(FnIdent {
            scoped: true,
            scope: Default::default(),
          }),
          "fn!" => ExprKind::Fn(FnIdent {
            scoped: false,
            scope: Default::default(),
          }),
          slice => ExprKind::Symbol(Symbol::from_ref(slice)),
        },
        info: Some(ExprInfo {
          source,
          span: token.span,
        }),
      })
    }
  }
}

fn parse_list(lexer: &mut Lexer) -> Result<(Vec<Expr>, Span), ParseError> {
  let mut list = Vec::new();

  loop {
    let token = lexer.peek();

    match token.kind {
      TokenKind::RightParen => break Ok((list, lexer.next().span)),
      _ => list.push(parse_expr(lexer)?),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
  pub source: Source,
  pub kind: ParseErrorKind,
}

impl std::error::Error for ParseError {}

impl fmt::Display for ParseError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{} at {}:{}",
      self.kind,
      self.source.name(),
      self
        .kind
        .location(&self.source)
        .map(|x| x.to_string())
        .unwrap_or_else(|| "?:?".into())
    )
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorKind {
  UnexpectedToken(Token),
  InvalidLiteral(Token),
}

impl ParseErrorKind {
  pub fn location(self, source: &Source) -> Option<Location> {
    match self {
      Self::UnexpectedToken(x) => source.location(x.span.start),
      Self::InvalidLiteral(x) => source.location(x.span.start),
    }
  }
}

impl fmt::Display for ParseErrorKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::UnexpectedToken(x) => write!(f, "unexpected token {x}"),
      Self::InvalidLiteral(x) => write!(f, "invalid literal {x}"),
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use test_case::case;

  fn s(source: &str) -> Source {
    Source::new("", source)
  }

  #[case("" => Ok(Vec::<Expr>::new()) ; "empty")]
  #[case("1" => Ok(vec![Expr { kind: ExprKind::Integer(1), info: Some(ExprInfo { source: s("1"), span: Span { start: 0, end: 1 } }) }]))]
  fn parse(source: &str) -> Result<Vec<Expr>, ParseError> {
    let mut lexer = Lexer::new(s(source));
    super::parse(&mut lexer)
  }
}
