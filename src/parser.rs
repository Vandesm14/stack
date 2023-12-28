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

////////////////////////////////////////////////////////////////////////////////

// use crate::{Context, Expr, Token};

// pub fn parse(context: &mut Context, tokens: Vec<Token>) -> Vec<Expr> {
//   let mut lists: Vec<Vec<Expr>> = vec![Vec::new()];
//   let mut paren_count: isize = 0;

//   // Wrap tokens in parens for easier parsing
//   let mut wrapped_tokens = vec![Token::ParenStart];
//   wrapped_tokens.extend(tokens);
//   wrapped_tokens.push(Token::ParenEnd);

//   for token in wrapped_tokens {
//     let expr = match token {
//       Token::Integer(i) => Some(Expr::Integer(i)),
//       Token::Float(f) => Some(Expr::Float(f)),
//       Token::String(s) => Some(Expr::String(s)),
//       Token::NoEval => Some(Expr::Lazy(Expr::Nil.into())),
//       Token::Call(s) => match context.resolve(&s) {
//         "true" => Some(Expr::Boolean(true)),
//         "false" => Some(Expr::Boolean(false)),
//         "fn" => Some(Expr::FnScope(None)),
//         _ => Some(Expr::Call(s)),
//       },
//       Token::Nil => Some(Expr::Nil),

//       Token::ParenStart => {
//         lists.push(Vec::new());
//         paren_count += 1;
//         None
//       }
//       // We can run this both when we see an ending paren and at the end of the code
//       Token::ParenEnd => {
//         let block = lists.pop().unwrap();
//         let mut new_block = Vec::new();
//         let mut temp_expr: Option<Expr> = None;

//         for expr in block.into_iter().rev() {
//           match expr {
//             Expr::Lazy(_) => {
//               temp_expr =
//                 Some(Expr::Lazy(temp_expr.take().unwrap_or(Expr::Nil).into()));
//             }
//             _ => {
//               if let Some(temp) = temp_expr.take() {
//                 new_block.push(temp);
//               }
//               temp_expr = Some(expr);
//             }
//           }
//         }

//         if let Some(temp) = temp_expr.take() {
//           new_block.push(temp);
//         }

//         new_block.reverse();

//         lists
//           .last_mut()
//           .unwrap_or(&mut vec![])
//           .push(Expr::List(new_block));
//         paren_count -= 1;
//         None
//       }
//       Token::CurlyStart => Some(Expr::ScopePush),
//       Token::CurlyEnd => Some(Expr::ScopePop),
//     };

//     if let Some(expr) = expr {
//       lists.last_mut().unwrap().push(expr);
//     }
//   }

//   if lists.len() != 1 {
//     eprintln!("Unbalanced blocks: {:?}", lists);
//     return vec![];
//   }

//   if paren_count != 0 {
//     eprintln!("Unbalanced parens: {:?}", lists);
//     return vec![];
//   }

//   // Unwrap the exprs from the list we wrapped them in at the beginning
//   if let Some(Expr::List(exprs)) = lists.last().unwrap().clone().first() {
//     exprs.clone()
//   } else {
//     vec![]
//   }
// }

// #[cfg(test)]
// mod tests {
//   use super::*;

//   use test_case::test_case;

//   #[test_case(
//     "(1 2 3)"
//     => vec![Expr::List(vec![
//       Expr::Integer(1),
//       Expr::Integer(2),
//       Expr::Integer(3),
//     ])]
//     ; "block"
//   )]
//   #[test_case(
//     "(1 2 3) 4 5 6"
//     => vec![
//       Expr::List(vec![
//         Expr::Integer(1),
//         Expr::Integer(2),
//         Expr::Integer(3),
//       ]),
//       Expr::Integer(4),
//       Expr::Integer(5),
//       Expr::Integer(6),
//     ]
//     ; "block before exprs"
//   )]
//   #[test_case(
//     "1 2 3 (4 5 6)"
//     => vec![
//       Expr::Integer(1),
//       Expr::Integer(2),
//       Expr::Integer(3),
//       Expr::List(vec![
//         Expr::Integer(4),
//         Expr::Integer(5),
//         Expr::Integer(6),
//       ]),
//     ]
//     ; "block after exprs"
//   )]
//   #[test_case(
//     "1 2 (3 4 5) 6"
//     => vec![
//       Expr::Integer(1),
//       Expr::Integer(2),
//       Expr::List(vec![
//         Expr::Integer(3),
//         Expr::Integer(4),
//         Expr::Integer(5),
//       ]),
//       Expr::Integer(6),
//     ]
//     ; "block between exprs"
//   )]
//   #[test_case(
//     "(1 (2 3) 4)"
//     => vec![Expr::List(vec![
//       Expr::Integer(1),
//       Expr::List(vec![
//         Expr::Integer(2),
//         Expr::Integer(3),
//       ]),
//       Expr::Integer(4),
//     ])]
//     ; "nested blocks"
//   )]
//   #[test_case("(" => Vec::<Expr>::new() ; "invalid block 0")]
//   #[test_case(")" => Vec::<Expr>::new() ; "invalid block 1")]
//   #[test_case("(]" => Vec::<Expr>::new() ; "invalid block 2")]
//   #[test_case("(}" => Vec::<Expr>::new() ; "invalid block 3")]
//   #[test_case(
//     "false true"
//     => vec![Expr::Boolean(false), Expr::Boolean(true)]
//     ; "boolean"
//   )]
//   // TODO: Implement a nice way to test with Spurs.
//   // #[test_case(
//   //   "{1 'var set}"
//   //   => vec![
//   //     Expr::ScopePush,
//   //     Expr::Integer(1),
//   //     Expr::Lazy(Expr::Call("var".into()).into()),
//   //     Expr::Call("set".into()),
//   //     Expr::ScopePop,
//   //   ]
//   //   ; "scope"
//   // )]
//   #[test_case(
//     "'(1 2 3)"
//     => vec![Expr::Lazy(Expr::List(vec![
//       Expr::Integer(1),
//       Expr::Integer(2),
//       Expr::Integer(3),
//     ]).into())]
//     ; "lazy block"
//   )]
//   #[test_case(
//     "'(1 '(2) 3)"
//     => vec![Expr::Lazy(Expr::List(vec![
//       Expr::Integer(1),
//       Expr::Lazy(Expr::List(vec![Expr::Integer(2)]).into()),
//       Expr::Integer(3),
//     ]).into())]
//     ; "lazy nested blocks"
//   )]
//   #[test_case(
//     "''1"
//     => vec![Expr::Lazy(Expr::Lazy(Expr::Integer(1).into()).into())]
//     ; "lazy lazy expr"
//   )]
//   fn parse(code: impl AsRef<str>) -> Vec<Expr> {
//     let mut context = Context::new();
//     let tokens = crate::lex(&mut context, code.as_ref());
//     super::parse(&mut context, tokens)
//   }
// }
