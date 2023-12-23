use core::fmt;

use crate::Token;

#[derive(Debug, Clone, Default)]
pub enum Expr {
  Integer(i64),
  Float(f64),

  String(String),
  Boolean(bool),

  Lazy(Box<Expr>),
  Call(String),
  FnScope(Option<usize>),

  /// `(1 2 3)` is a list
  List(Vec<Expr>),

  /// Creates a new scope
  ScopePush,

  /// Pops the current scope
  ScopePop,

  #[default]
  Nil,
}

impl fmt::Display for Expr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Expr::Integer(i) => write!(f, "{}", i),
      Expr::Float(float) => write!(f, "{}", float),

      Expr::String(s) => write!(f, "\"{}\"", s),
      Expr::Boolean(b) => write!(f, "{}", b),

      Expr::Lazy(expr) => write!(f, "'{}", expr),
      Expr::Call(s) => write!(f, "{}", s),
      Expr::FnScope(i) => write!(
        f,
        "fn{}",
        if i.is_some() {
          format!("({})", i.unwrap())
        } else {
          "".to_owned()
        }
      ),

      Expr::List(l) => write!(
        f,
        "({})",
        l.iter()
          .map(|e| e.to_string())
          .collect::<Vec<String>>()
          .join(" ")
      ),

      Expr::ScopePush => write!(f, "scope_push"),
      Expr::ScopePop => write!(f, "scope_pop"),

      Expr::Nil => write!(f, "nil"),
    }
  }
}

impl From<Option<Expr>> for Expr {
  fn from(value: Option<Expr>) -> Self {
    match value {
      Option::None => Expr::Nil,
      Option::Some(value) => value,
    }
  }
}

impl PartialEq for Expr {
  fn eq(&self, other: &Expr) -> bool {
    match (self, other) {
      // Same types
      (Expr::Float(a), Expr::Float(b)) => a == b,
      (Expr::Integer(a), Expr::Integer(b)) => a == b,

      (Expr::String(a), Expr::String(b)) => a == b,
      (Expr::Boolean(a), Expr::Boolean(b)) => a == b,

      (Expr::Lazy(a), Expr::Lazy(b)) => a == b,
      (Expr::Call(a), Expr::Call(b)) => a == b,
      (Expr::FnScope(a), Expr::FnScope(b)) => a == b,

      (Expr::List(a), Expr::List(b)) => a == b,

      (Expr::ScopePush, Expr::ScopePush) => true,
      (Expr::ScopePop, Expr::ScopePop) => true,

      (Expr::Nil, Expr::Nil) => true,

      // Different types
      (Expr::Float(a), Expr::Integer(b)) => {
        let b = *b as f64;
        a == &b
      }
      (Expr::Integer(a), Expr::Float(b)) => {
        let a = *a as f64;
        &a == b
      }

      (Expr::Integer(a), Expr::Boolean(b)) => {
        let a = *a != 0;
        a == *b
      }
      (Expr::Boolean(a), Expr::Integer(b)) => {
        let b = *b != 0;
        *a == b
      }
      _ => false,
    }
  }
}

impl PartialOrd for Expr {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Eq for Expr {
  fn assert_receiver_is_total_eq(&self) {}
}

impl Ord for Expr {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    match (self, other) {
      // Same types
      (Expr::Integer(a), Expr::Integer(b)) => a.cmp(b),
      (Expr::Float(a), Expr::Float(b)) => a.partial_cmp(b).unwrap(),

      (Expr::String(a), Expr::String(b)) => a.cmp(b),
      (Expr::Boolean(a), Expr::Boolean(b)) => a.cmp(b),

      (Expr::Lazy(a), Expr::Lazy(b)) => a.cmp(b),
      (Expr::Call(a), Expr::Call(b)) => a.cmp(b),
      (Expr::FnScope(a), Expr::FnScope(b)) => a.cmp(b),

      (Expr::List(a), Expr::List(b)) => a.cmp(b),

      (Expr::Nil, Expr::Nil) => std::cmp::Ordering::Equal,

      // Different types
      (Expr::Float(a), Expr::Integer(b)) => {
        let b = *b as f64;
        a.partial_cmp(&b).unwrap()
      }
      (Expr::Integer(a), Expr::Float(b)) => {
        let a = *a as f64;
        a.partial_cmp(b).unwrap()
      }

      _ => std::cmp::Ordering::Equal,
    }
  }
}

impl Expr {
  pub fn is_truthy(&self) -> bool {
    match self {
      Expr::Nil => false,
      Expr::Boolean(b) => *b,
      Expr::Integer(i) => *i != 0,
      Expr::Float(f) => *f != 0.0,
      _ => true,
    }
  }

  pub fn is_nil(&self) -> bool {
    matches!(self, Expr::Nil)
  }

  pub fn type_of(&self) -> String {
    match self {
      Expr::Integer(_) => "integer".to_owned(),
      Expr::Float(_) => "float".to_owned(),

      Expr::String(_) => "string".to_owned(),
      Expr::Boolean(_) => "boolean".to_owned(),

      Expr::Lazy(expr) => expr.type_of(),
      Expr::Call(_) => "call".to_owned(),
      Expr::FnScope(_) => "fn".to_owned(),

      Expr::List(_) => "list".to_owned(),

      Expr::ScopePush => self.to_string(),
      Expr::ScopePop => self.to_string(),

      Expr::Nil => self.to_string(),
    }
  }

  pub fn is_function(&self) -> bool {
    if let Expr::List(list) = self {
      if let Expr::FnScope(_) = list.first().unwrap_or(&Expr::Nil) {
        return true;
      }
    }

    false
  }

  pub fn function_scope(&self) -> Option<usize> {
    if let Expr::List(list) = self {
      if let Expr::FnScope(scope) = list.first().unwrap_or(&Expr::Nil) {
        return *scope;
      }
    }

    None
  }

  pub fn contains_block(&self) -> bool {
    if let Expr::List(list) = self {
      if let Expr::ScopePush = list.get(1).unwrap_or(&Expr::Nil) {
        return true;
      }
    }

    false
  }
}

pub fn parse(tokens: Vec<Token>) -> Vec<Expr> {
  let mut lists: Vec<Vec<Expr>> = vec![Vec::new()];
  let mut paren_count: isize = 0;

  // Wrap tokens in parens for easier parsing
  let mut wrapped_tokens = vec![Token::ParenStart];
  wrapped_tokens.extend(tokens);
  wrapped_tokens.push(Token::ParenEnd);

  for token in wrapped_tokens {
    let expr = match token {
      Token::Integer(i) => Some(Expr::Integer(i)),
      Token::Float(f) => Some(Expr::Float(f)),
      Token::String(s) => Some(Expr::String(s)),
      Token::NoEval => Some(Expr::Lazy(Expr::Nil.into())),
      Token::Call(s) => match s.as_str() {
        "true" => Some(Expr::Boolean(true)),
        "false" => Some(Expr::Boolean(false)),
        "fn" => Some(Expr::FnScope(None)),
        _ => Some(Expr::Call(s)),
      },
      Token::Nil => Some(Expr::Nil),

      Token::ParenStart => {
        lists.push(Vec::new());
        paren_count += 1;
        None
      }
      // We can run this both when we see an ending paren and at the end of the code
      Token::ParenEnd => {
        let block = lists.pop().unwrap();
        let mut new_block = Vec::new();
        let mut temp_expr: Option<Expr> = None;

        for expr in block.into_iter().rev() {
          match expr {
            Expr::Lazy(_) => {
              temp_expr =
                Some(Expr::Lazy(temp_expr.take().unwrap_or(Expr::Nil).into()));
            }
            _ => {
              if let Some(temp) = temp_expr.take() {
                new_block.push(temp);
              }
              temp_expr = Some(expr);
            }
          }
        }

        if let Some(temp) = temp_expr.take() {
          new_block.push(temp);
        }

        new_block.reverse();

        lists
          .last_mut()
          .unwrap_or(&mut vec![])
          .push(Expr::List(new_block));
        paren_count -= 1;
        None
      }
      Token::CurlyStart => Some(Expr::ScopePush),
      Token::CurlyEnd => Some(Expr::ScopePop),
    };

    if let Some(expr) = expr {
      lists.last_mut().unwrap().push(expr);
    }
  }

  if lists.len() != 1 {
    eprintln!("Unbalanced blocks: {:?}", lists);
    return vec![];
  }

  if paren_count != 0 {
    eprintln!("Unbalanced parens: {:?}", lists);
    return vec![];
  }

  // Unwrap the exprs from the list we wrapped them in at the beginning
  if let Some(Expr::List(exprs)) = lists.last().unwrap().clone().first() {
    exprs.clone()
  } else {
    vec![]
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod parsing {
    use super::*;

    #[test]
    fn implicit_block() {
      let tokens = crate::lex("(1 2 3)");
      let expected = vec![Expr::List(vec![
        Expr::Integer(1),
        Expr::Integer(2),
        Expr::Integer(3),
      ])];

      assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn block_at_beginning() {
      let tokens = crate::lex("(1 2 3) 4 5 6");
      let expected = vec![
        Expr::List(vec![Expr::Integer(1), Expr::Integer(2), Expr::Integer(3)]),
        Expr::Integer(4),
        Expr::Integer(5),
        Expr::Integer(6),
      ];

      assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn nested_blocks() {
      let tokens = crate::lex("(1 (2 3) 4)");
      let expected = vec![Expr::List(vec![
        Expr::Integer(1),
        Expr::List(vec![Expr::Integer(2), Expr::Integer(3)]),
        Expr::Integer(4),
      ])];

      assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn fail_for_only_start_paren() {
      let tokens = crate::lex("(");
      let exprs = parse(tokens);
      assert_eq!(exprs, vec![]);
    }

    #[test]
    fn fail_for_only_end_paren() {
      let tokens = crate::lex(")");
      let exprs = parse(tokens);
      assert_eq!(exprs, vec![]);
    }

    #[test]
    fn fail_for_mismatched_parens() {
      let tokens = crate::lex("(1 2 3]");
      let exprs = parse(tokens);
      assert_eq!(exprs, vec![]);
    }

    #[test]
    fn booleans() {
      let tokens = crate::lex("true false");
      let expected = vec![Expr::Boolean(true), Expr::Boolean(false)];

      assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn scope() {
      let tokens = crate::lex("{1 'var set}");
      let expected = vec![
        Expr::ScopePush,
        Expr::Integer(1),
        Expr::Lazy(Box::new(Expr::Call("var".to_owned()))),
        Expr::Call("set".to_owned()),
        Expr::ScopePop,
      ];

      assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn lazy_calls() {
      let tokens = crate::lex("'set");
      let expected = vec![Expr::Lazy(Box::new(Expr::Call("set".to_owned())))];

      assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn lazy_lists() {
      let tokens = crate::lex("'(1 2 3)");
      let expected = vec![Expr::Lazy(Box::new(Expr::List(vec![
        Expr::Integer(1),
        Expr::Integer(2),
        Expr::Integer(3),
      ])))];

      assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn lazy_nested_lists() {
      let tokens = crate::lex("'(1 (2 3) 4)");
      let expected = vec![Expr::Lazy(Box::new(Expr::List(vec![
        Expr::Integer(1),
        Expr::List(vec![Expr::Integer(2), Expr::Integer(3)]),
        Expr::Integer(4),
      ])))];

      assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn double_lazy() {
      let tokens = crate::lex("''set");
      let expected = vec![Expr::Lazy(Box::new(Expr::Lazy(Box::new(
        Expr::Call("set".to_owned()),
      ))))];

      assert_eq!(parse(tokens), expected);
    }
  }

  mod equality {
    use super::*;

    mod same_types {
      use super::*;

      #[test]
      fn integer_to_integer() {
        let a = Expr::Integer(1);
        let b = Expr::Integer(1);
        assert_eq!(a, b);

        let a = Expr::Integer(1);
        let b = Expr::Integer(2);
        assert_ne!(a, b);
      }

      #[test]
      fn float_to_float() {
        let a = Expr::Float(1.0);
        let b = Expr::Float(1.0);
        assert_eq!(a, b);

        let a = Expr::Float(1.0);
        let b = Expr::Float(1.1);
        assert_ne!(a, b);
      }

      #[test]
      fn string_to_string() {
        let a = Expr::String("hello".to_owned());
        let b = Expr::String("hello".to_owned());
        assert_eq!(a, b);

        let a = Expr::String("hello".to_owned());
        let b = Expr::String("world".to_owned());
        assert_ne!(a, b);
      }

      #[test]
      fn boolean_to_boolean() {
        let a = Expr::Boolean(true);
        let b = Expr::Boolean(true);
        assert_eq!(a, b);

        let a = Expr::Boolean(true);
        let b = Expr::Boolean(false);
        assert_ne!(a, b);
      }

      #[test]
      fn call_to_call() {
        let a = Expr::Call("hello".to_owned());
        let b = Expr::Call("hello".to_owned());
        assert_eq!(a, b);

        let a = Expr::Call("hello".to_owned());
        let b = Expr::Call("world".to_owned());
        assert_ne!(a, b);
      }

      #[test]
      fn list_to_list() {
        let a = Expr::List(vec![Expr::Integer(1), Expr::Integer(2)]);
        let b = Expr::List(vec![Expr::Integer(1), Expr::Integer(2)]);
        assert_eq!(a, b);

        let a = Expr::List(vec![Expr::Integer(1), Expr::Integer(2)]);
        let b = Expr::List(vec![Expr::Integer(1), Expr::Integer(3)]);
        assert_ne!(a, b);
      }

      #[test]
      fn nil_to_nil() {
        let a = Expr::Nil;
        let b = Expr::Nil;
        assert_eq!(a, b);
      }
    }

    mod different_types {
      use super::*;

      #[test]
      fn integer_to_float() {
        let a = Expr::Integer(1);
        let b = Expr::Float(1.0);
        assert_eq!(a, b);

        let a = Expr::Integer(1);
        let b = Expr::Float(1.1);
        assert_ne!(a, b);
      }

      #[test]
      fn float_to_integer() {
        let a = Expr::Float(1.0);
        let b = Expr::Integer(1);
        assert_eq!(a, b);

        let a = Expr::Float(1.1);
        let b = Expr::Integer(1);
        assert_ne!(a, b);
      }

      #[test]
      fn integer_to_boolean() {
        let a = Expr::Integer(1);
        let b = Expr::Boolean(true);
        assert_eq!(a, b);

        let a = Expr::Integer(0);
        let b = Expr::Boolean(false);
        assert_eq!(a, b);
      }

      #[test]
      fn boolean_to_integer() {
        let a = Expr::Boolean(true);
        let b = Expr::Integer(1);
        assert_eq!(a, b);

        let a = Expr::Boolean(false);
        let b = Expr::Integer(0);
        assert_eq!(a, b);
      }
    }
  }

  mod ordering {
    use super::*;

    mod same_types {
      use super::*;

      #[test]
      fn integer_to_integer() {
        let a = Expr::Integer(1);
        let b = Expr::Integer(1);
        assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Equal));

        let a = Expr::Integer(1);
        let b = Expr::Integer(2);
        assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Less));

        let a = Expr::Integer(2);
        let b = Expr::Integer(1);
        assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Greater));
      }

      #[test]
      fn float_to_float() {
        let a = Expr::Float(1.0);
        let b = Expr::Float(1.0);
        assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Equal));

        let a = Expr::Float(1.0);
        let b = Expr::Float(1.1);
        assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Less));

        let a = Expr::Float(1.1);
        let b = Expr::Float(1.0);
        assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Greater));
      }
    }

    mod different_types {
      use super::*;

      #[test]
      fn integer_to_float() {
        let a = Expr::Integer(1);
        let b = Expr::Float(1.0);
        assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Equal));

        let a = Expr::Integer(1);
        let b = Expr::Float(1.1);
        assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Less));

        let a = Expr::Integer(2);
        let b = Expr::Float(1.0);
        assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Greater));
      }

      #[test]
      fn float_to_integer() {
        let a = Expr::Float(1.0);
        let b = Expr::Integer(1);
        assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Equal));

        let a = Expr::Float(1.1);
        let b = Expr::Integer(1);
        assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Greater));

        let a = Expr::Float(1.0);
        let b = Expr::Integer(2);
        assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Less));
      }
    }
  }
}
