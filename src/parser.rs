use thiserror::Error;

use crate::{
  Ast, AstIndex, Expr, FnSymbol, Lexer, Scope, Span, TokenKind, TokenVec,
};

/// Converts a stream of [`Token`]s into a stream of [`Expr`]s.
///
/// [`Token`]: crate::Token
#[derive(Debug)]
pub struct Parser<'source, 'ast> {
  tokens: TokenVec<'source>,
  cursor: usize,
  ast: &'ast mut Ast,
}

impl<'source, 'ast> Parser<'source, 'ast> {
  /// Creates a new [`Parser`].
  ///
  /// Prefer [`Parser::reuse`] where possible.
  #[inline]
  pub const fn new(lexer: Lexer<'source>, ast: &'ast mut Ast) -> Self {
    Self {
      tokens: TokenVec::new(lexer),
      cursor: 0,
      ast,
    }
  }

  /// Creates a [`Parser`] by re-using the allocations of an existing one.
  #[inline]
  pub fn reuse(&mut self, lexer: Lexer<'source>) {
    self.tokens.reuse(lexer);
  }

  /// Parses all of the available [`Expr`]s into an [`Ast`].
  ///
  /// If a [`ParseError`] is encountered, the whole collect fails.
  #[inline]
  pub fn parse(mut self) -> Result<(), ParseError> {
    let mut exprs = Vec::new();

    while let Some(result) = self.next().transpose() {
      exprs.push(result?);
    }

    Ok(())
  }

  /// Returns the next [`AstIndex`].
  ///
  /// Once the first <code>[Ok]\([None]\)</code> has been returned, it will
  /// continue to return them thereafter, akin to a [`FusedIterator`].
  ///
  /// [`FusedIterator`]: core::iter::FusedIterator
  #[allow(clippy::should_implement_trait)]
  // ^ This is fine. If it acts like an iterator, it's an iterator.
  pub fn next(&mut self) -> Result<Option<AstIndex>, ParseError> {
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
          break Ok(Some(self.ast.push_expr(Expr::Boolean(x))));
        }
        TokenKind::Integer(x) => {
          break Ok(Some(self.ast.push_expr(Expr::Integer(x))));
        }
        TokenKind::Float(x) => {
          break Ok(Some(self.ast.push_expr(Expr::Float(x))));
        }
        TokenKind::String(x) => {
          break Ok(Some(self.ast.push_expr(Expr::String(x))));
        }

        TokenKind::Ident(x) => {
          break Ok(Some(self.ast.push_expr(Expr::Call(x))));
        }

        TokenKind::Apostrophe => {
          break match self.next() {
            Ok(Some(expr)) => Ok(Some(self.ast.push_expr(Expr::Lazy(expr)))),
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
                break Ok(Some(self.ast.push_expr(Expr::List(list))));
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
          break Ok(Some(self.ast.push_expr(Expr::Nil)));
        }
        TokenKind::Fn => {
          break Ok(Some(self.ast.push_expr(Expr::Fn(FnSymbol {
            scoped: true,
            scope: Scope::new(),
          }))));
        }
        TokenKind::FnExclamation => {
          break Ok(Some(self.ast.push_expr(Expr::Fn(FnSymbol {
            scoped: false,
            scope: Scope::new(),
          }))));
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
  #[test_case("12 34 +" => Ok(vec![Expr::Integer(12), Expr::Integer(34), Expr::Call(interned().PLUS)]) ; "int int add")]
  #[test_case("æ 34 -" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::Invalid }, span: Span { start: 0, end: 2 } }) ; "invalid int sub")]
  #[test_case("12 æ *" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::Invalid }, span: Span { start: 3, end: 5 } }) ; "int invalid mul")]
  #[test_case("'" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::Apostrophe }, span: Span { start: 0, end: 1 } }) ; "empty lazy")]
  #[test_case("'12" => Ok(vec![Expr::Lazy(Box::new(Expr::Integer(12)))]) ; "lazy int")]
  #[test_case("()" => Ok(vec![Expr::List(vec![])]) ; "empty list")]
  #[test_case("(\n)" => Ok(vec![Expr::List(vec![])]) ; "empty list whitespace")]
  #[test_case("'()" => Ok(vec![Expr::Lazy(Box::new(Expr::List(vec![])))]) ; "lazy empty list")]
  #[test_case("(')" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::ParenClose }, span: Span { start: 2, end: 3 } }) ; "list empty lazy")]
  #[test_case("('())" => Ok(vec![Expr::List(vec![Expr::Lazy(Box::new(Expr::List(vec![])))])]) ; "list lazy list")]
  #[test_case("'('())" => Ok(vec![Expr::Lazy(Box::new(Expr::List(vec![Expr::Lazy(Box::new(Expr::List(vec![])))])))]) ; "lazy list lazy list")]
  #[test_case("('('))" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::ParenClose }, span: Span { start: 4, end: 5 } }) ; "list lazy list empty lazy")]
  #[test_case("'('('))" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::ParenClose }, span: Span { start: 5, end: 6 } }) ; "lazy list lazy list empty lazy")]
  #[test_case("(12 +)" => Ok(vec![Expr::List(vec![Expr::Integer(12), Expr::Call(interned().PLUS)])]) ; "list int add")]
  #[test_case("'(12 +)" => Ok(vec![Expr::Lazy(Box::new(Expr::List(vec![Expr::Integer(12), Expr::Call(interned().PLUS)])))]) ; "lazy list int add")]
  #[test_case("(æ +)" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::Invalid }, span: Span { start: 1, end: 3 } }) ; "invalid int add")]
  #[test_case("'(æ +)" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::Invalid }, span: Span { start: 2, end: 4 } }) ; "lazy invalid int add")]
  #[test_case("[]" => Ok(vec![]) ; "square empty")]
  #[test_case("[ßℝ💣]" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::Invalid }, span: Span { start: 1, end: 10 } }) ; "square invalid")]
  #[test_case("['ßℝ💣]" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::Invalid }, span: Span { start: 2, end: 11 } }) ; "square lazy invalid")]
  #[test_case("[12 34 +]" => Ok(vec![Expr::Integer(12), Expr::Integer(34), Expr::Call(interned().PLUS)]) ; "square int int add")]
  #[test_case("[æ 34 -]" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::Invalid }, span: Span { start: 1, end: 3 } }) ; "square invalid int sub")]
  #[test_case("[12 æ *]" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::Invalid }, span: Span { start: 4, end: 6 } }) ; "square int invalid mul")]
  #[test_case("[']" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::Apostrophe }, span: Span { start: 1, end: 2 } }) ; "square empty lazy")]
  #[test_case("['12]" => Ok(vec![Expr::Lazy(Box::new(Expr::Integer(12)))]) ; "square lazy int")]
  #[test_case("[()]" => Ok(vec![Expr::List(vec![])]) ; "square empty list")]
  #[test_case("[(\n)]" => Ok(vec![Expr::List(vec![])]) ; "square empty list whitespace")]
  #[test_case("['()]" => Ok(vec![Expr::Lazy(Box::new(Expr::List(vec![])))]) ; "square lazy empty list")]
  #[test_case("[(')]" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::ParenClose }, span: Span { start: 3, end: 4 } }) ; "square list empty lazy")]
  #[test_case("[('())]" => Ok(vec![Expr::List(vec![Expr::Lazy(Box::new(Expr::List(vec![])))])]) ; "square list lazy list")]
  #[test_case("['('())]" => Ok(vec![Expr::Lazy(Box::new(Expr::List(vec![Expr::Lazy(Box::new(Expr::List(vec![])))])))]) ; "square lazy list lazy list")]
  #[test_case("[('('))]" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::ParenClose }, span: Span { start: 5, end: 6 } }) ; "square list lazy list empty lazy")]
  #[test_case("['('('))]" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::ParenClose }, span: Span { start: 6, end: 7 } }) ; "square lazy list lazy list empty lazy")]
  #[test_case("[(12 +)]" => Ok(vec![Expr::List(vec![Expr::Integer(12), Expr::Call(interned().PLUS)])]) ; "square list int add")]
  #[test_case("['(12 +)]" => Ok(vec![Expr::Lazy(Box::new(Expr::List(vec![Expr::Integer(12), Expr::Call(interned().PLUS)])))]) ; "square lazy list int add")]
  #[test_case("[(æ +)]" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::Invalid }, span: Span { start: 2, end: 4 } }) ; "square invalid int add")]
  #[test_case("['(æ +)]" => Err(ParseError { reason: ParseErrorReason::UnexpectedToken { kind: TokenKind::Invalid }, span: Span { start: 3, end: 5 } }) ; "square lazy invalid int add")]
  fn parser(source: &str) -> Result<Vec<Expr>, ParseError> {
    let lexer = Lexer::new(source);
    let parser = Parser::new(lexer);
    parser.parse()
  }
}
