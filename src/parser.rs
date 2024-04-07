use lasso::Spur;
use thiserror::Error;

use crate::{
  DebugData, Expr, ExprKind, FnSymbol, Lexer, Scope, Span, TokenKind, TokenVec,
};

/// Converts a stream of [`Token`]s into a stream of [`Expr`]s.
///
/// [`Token`]: crate::Token
#[derive(Debug, Clone, PartialEq)]
pub struct Parser<'source> {
  filename: Spur,
  tokens: TokenVec<'source>,
  cursor: usize,
}

impl<'source> Parser<'source> {
  /// Creates a new [`Parser`].
  ///
  /// Prefer [`Parser::reuse`] where possible.
  #[inline]
  pub const fn new(lexer: Lexer<'source>, filename: Spur) -> Self {
    Self {
      filename,
      tokens: TokenVec::new(lexer),
      cursor: 0,
    }
  }

  /// Creates a [`Parser`] by re-using the allocations of an existing one.
  #[inline]
  pub fn reuse(&mut self, lexer: Lexer<'source>) {
    self.tokens.reuse(lexer);
  }

  /// Parses all of the available [`Expr`]s into a [`Vec`].
  ///
  /// If a [`ParseError`] is encountered, the whole collect fails.
  #[inline]
  pub fn parse(mut self) -> Result<Vec<Expr>, ParseError> {
    let mut exprs = Vec::new();

    while let Some(result) = self.next().transpose() {
      exprs.push(result?);
    }

    Ok(exprs)
  }

  /// Returns the next [`Expr`].
  ///
  /// Once the first <code>[Ok]\([None]\)</code> has been returned, it will
  /// continue to return them thereafter, akin to a [`FusedIterator`].
  ///
  /// [`FusedIterator`]: core::iter::FusedIterator
  #[allow(clippy::should_implement_trait)]
  // ^ This is fine. If it acts like an iterator, it's an iterator.
  pub fn next(&mut self) -> Result<Option<Expr>, ParseError> {
    loop {
      let token = self.tokens.token(self.cursor);
      self.cursor += 1;

      match token.kind {
        TokenKind::Invalid | TokenKind::ParenClose => {
          break Err(ParseError {
            reason: ParseErrorReason::UnexpectedToken { kind: token.kind },
            span: token.span,
          });
        }
        TokenKind::Eoi => {
          break Ok(None);
        }

        TokenKind::Whitespace | TokenKind::Comment => {
          continue;
        }

        TokenKind::Boolean(x) => {
          break Ok(Some(ExprKind::Boolean(x).into_expr(DebugData {
            source_file: Some(self.filename),
            span: Some(token.span),
            ingredients: vec![],
          })));
        }
        TokenKind::Integer(x) => {
          break Ok(Some(ExprKind::Integer(x).into_expr(DebugData {
            source_file: Some(self.filename),
            span: Some(token.span),
            ingredients: vec![],
          })));
        }
        TokenKind::Float(x) => {
          break Ok(Some(ExprKind::Float(x).into_expr(DebugData {
            source_file: Some(self.filename),
            span: Some(token.span),
            ingredients: vec![],
          })));
        }
        TokenKind::String(x) => {
          break Ok(Some(ExprKind::String(x).into_expr(DebugData {
            source_file: Some(self.filename),
            span: Some(token.span),
            ingredients: vec![],
          })));
        }

        TokenKind::Ident(x) => {
          break Ok(Some(ExprKind::Call(x).into_expr(DebugData {
            source_file: Some(self.filename),
            span: Some(token.span),
            ingredients: vec![],
          })));
        }

        TokenKind::Apostrophe => {
          break match self.next() {
            Ok(Some(expr)) => {
              Ok(Some(ExprKind::Lazy(Box::new(expr)).into_expr(DebugData {
                source_file: Some(self.filename),
                span: Some(Span {
                  start: token.span.start,
                  // TODO: We probably shouldn't be unwrapping here, though processed expressions should
                  // ALWAYS have a span in debug data, so this is fine if it hard-errors
                  end: expr.debug_data.span.unwrap().end,
                }),
                ingredients: vec![],
              })))
            }
            Ok(None) => Err(ParseError {
              reason: ParseErrorReason::UnexpectedToken { kind: token.kind },
              span: token.span,
            }),
            err @ Err(_) => err,
          };
        }
        TokenKind::ParenOpen => {
          let mut list = Vec::new();

          break loop {
            let token = self.tokens.token(self.cursor);

            match token.kind {
              TokenKind::Whitespace | TokenKind::Comment => {
                self.cursor += 1;
                continue;
              }
              TokenKind::ParenClose => {
                self.cursor += 1;
                break Ok(Some(ExprKind::List(list).into_expr(DebugData {
                  source_file: Some(self.filename),
                  span: Some(Span {
                    start: match list.first() {
                      // TODO: same as the TODO above
                      Some(expr) => expr.debug_data.span.unwrap().start,
                      None => token.span.start,
                    } - 1,
                    end: token.span.end,
                  }),
                  ingredients: vec![],
                })));
              }
              _ => match self.next()? {
                Some(expr) => list.push(expr),
                None => {
                  break Err(ParseError {
                    reason: ParseErrorReason::UnexpectedToken {
                      kind: token.kind,
                    },
                    span: token.span,
                  });
                }
              },
            }
          };
        }
        // // TODO: This should check to make sure there are matching brakets.
        // TokenKind::SquareOpen | TokenKind::SquareClose => {
        //   continue;
        // }
        TokenKind::Nil => {
          break Ok(Some(ExprKind::Nil.into_expr(DebugData {
            source_file: Some(self.filename),
            span: Some(token.span),
            ingredients: vec![],
          })));
        }
        TokenKind::Fn => {
          break Ok(Some(
            ExprKind::Fn(FnSymbol {
              scoped: true,
              scope: Scope::new(),
            })
            .into_expr(DebugData {
              source_file: Some(self.filename),
              span: Some(token.span),
              ingredients: vec![],
            }),
          ));
        }
        TokenKind::FnExclamation => {
          break Ok(Some(
            ExprKind::Fn(FnSymbol {
              scoped: false,
              scope: Scope::new(),
            })
            .into_expr(DebugData {
              source_file: Some(self.filename),
              span: Some(token.span),
              ingredients: vec![],
            }),
          ));
        }
      }
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Error)]
#[error("{reason} at {span}")]
pub struct ParseError {
  pub reason: ParseErrorReason,
  pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Error)]
pub enum ParseErrorReason {
  #[error("unexpected token: {kind}")]
  UnexpectedToken { kind: TokenKind },
}

#[cfg(test)]
mod tests {
  use crate::interner::interned;

  use super::*;

  use test_case::test_case;

  #[test_case("" => Ok(Vec::<Expr>::new()) ; "empty")]
  #[test_case(" \t\r\n" => Ok(Vec::<Expr>::new()) ; "whitespace")]
  #[test_case("ßℝ💣" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::Invalid }, span: Span { start: 0, end: 9 } }) ; "invalid")]
  #[test_case("'ßℝ💣" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::Invalid }, span: Span { start: 1, end: 10 } }) ; "lazy invalid")]
  #[test_case("12 34 +" => Ok(vec![ExprKind::Integer(12), ExprKind::Integer(34), ExprKind::Call(interned().PLUS)]) ; "int int add")]
  #[test_case("æ 34 -" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::Invalid }, span: Span { start: 0, end: 2 } }) ; "invalid int sub")]
  #[test_case("12 æ *" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::Invalid }, span: Span { start: 3, end: 5 } }) ; "int invalid mul")]
  #[test_case("'" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::Apostrophe }, span: Span { start: 0, end: 1 } }) ; "empty lazy")]
  #[test_case("'12" => Ok(vec![ExprKind::Lazy(Box::new(ExprKind::Integer(12)))]) ; "lazy int")]
  #[test_case("()" => Ok(vec![ExprKind::List(vec![])]) ; "empty list")]
  #[test_case("(\n)" => Ok(vec![ExprKind::List(vec![])]) ; "empty list whitespace")]
  #[test_case("'()" => Ok(vec![ExprKind::Lazy(Box::new(ExprKind::List(vec![])))]) ; "lazy empty list")]
  #[test_case("(')" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::ParenClose }, span: Span { start: 2, end: 3 } }) ; "list empty lazy")]
  #[test_case("('())" => Ok(vec![ExprKind::List(vec![ExprKind::Lazy(Box::new(ExprKind::List(vec![])))])]) ; "list lazy list")]
  #[test_case("'('())" => Ok(vec![ExprKind::Lazy(Box::new(ExprKind::List(vec![ExprKind::Lazy(Box::new(ExprKind::List(vec![])))])))]) ; "lazy list lazy list")]
  #[test_case("('('))" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::ParenClose }, span: Span { start: 4, end: 5 } }) ; "list lazy list empty lazy")]
  #[test_case("'('('))" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::ParenClose }, span: Span { start: 5, end: 6 } }) ; "lazy list lazy list empty lazy")]
  #[test_case("(12 +)" => Ok(vec![ExprKind::List(vec![ExprKind::Integer(12), ExprKind::Call(interned().PLUS)])]) ; "list int add")]
  #[test_case("'(12 +)" => Ok(vec![ExprKind::Lazy(Box::new(ExprKind::List(vec![ExprKind::Integer(12), ExprKind::Call(interned().PLUS)])))]) ; "lazy list int add")]
  #[test_case("(æ +)" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::Invalid }, span: Span { start: 1, end: 3 } }) ; "invalid int add")]
  #[test_case("'(æ +)" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::Invalid }, span: Span { start: 2, end: 4 } }) ; "lazy invalid int add")]
  #[test_case("[]" => Ok(vec![]) ; "square empty")]
  #[test_case("[ßℝ💣]" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::Invalid }, span: Span { start: 1, end: 10 } }) ; "square invalid")]
  #[test_case("['ßℝ💣]" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::Invalid }, span: Span { start: 2, end: 11 } }) ; "square lazy invalid")]
  #[test_case("[12 34 +]" => Ok(vec![ExprKind::Integer(12), ExprKind::Integer(34), ExprKind::Call(interned().PLUS)]) ; "square int int add")]
  #[test_case("[æ 34 -]" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::Invalid }, span: Span { start: 1, end: 3 } }) ; "square invalid int sub")]
  #[test_case("[12 æ *]" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::Invalid }, span: Span { start: 4, end: 6 } }) ; "square int invalid mul")]
  #[test_case("[']" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::Apostrophe }, span: Span { start: 1, end: 2 } }) ; "square empty lazy")]
  #[test_case("['12]" => Ok(vec![ExprKind::Lazy(Box::new(ExprKind::Integer(12)))]) ; "square lazy int")]
  #[test_case("[()]" => Ok(vec![ExprKind::List(vec![])]) ; "square empty list")]
  #[test_case("[(\n)]" => Ok(vec![ExprKind::List(vec![])]) ; "square empty list whitespace")]
  #[test_case("['()]" => Ok(vec![ExprKind::Lazy(Box::new(ExprKind::List(vec![])))]) ; "square lazy empty list")]
  #[test_case("[(')]" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::ParenClose }, span: Span { start: 3, end: 4 } }) ; "square list empty lazy")]
  #[test_case("[('())]" => Ok(vec![ExprKind::List(vec![ExprKind::Lazy(Box::new(ExprKind::List(vec![])))])]) ; "square list lazy list")]
  #[test_case("['('())]" => Ok(vec![ExprKind::Lazy(Box::new(ExprKind::List(vec![ExprKind::Lazy(Box::new(ExprKind::List(vec![])))])))]) ; "square lazy list lazy list")]
  #[test_case("[('('))]" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::ParenClose }, span: Span { start: 5, end: 6 } }) ; "square list lazy list empty lazy")]
  #[test_case("['('('))]" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::ParenClose }, span: Span { start: 6, end: 7 } }) ; "square lazy list lazy list empty lazy")]
  #[test_case("[(12 +)]" => Ok(vec![ExprKind::List(vec![ExprKind::Integer(12), ExprKind::Call(interned().PLUS)])]) ; "square list int add")]
  #[test_case("['(12 +)]" => Ok(vec![ExprKind::Lazy(Box::new(ExprKind::List(vec![ExprKind::Integer(12), ExprKind::Call(interned().PLUS)])))]) ; "square lazy list int add")]
  #[test_case("[(æ +)]" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::Invalid }, span: Span { start: 2, end: 4 } }) ; "square invalid int add")]
  #[test_case("['(æ +)]" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::Invalid }, span: Span { start: 3, end: 5 } }) ; "square lazy invalid int add")]
  fn parser(source: &str) -> Result<Vec<Expr>, ParseError> {
    let lexer = Lexer::new(source);
    let parser = Parser::new(lexer);
    parser.parse()
  }
}
