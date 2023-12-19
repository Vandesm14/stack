use crate::Expr;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Program {
  pub stack: Vec<Expr>,
  pub scope: HashMap<String, Expr>,
}

impl Program {
  pub fn new() -> Self {
    Self {
      stack: vec![],
      scope: HashMap::new(),
    }
  }

  fn pop_eval(&mut self) -> Result<Expr, String> {
    let expr = self.stack.pop();
    if let Some(expr) = expr {
      if let Some(result) = self.eval_expr(expr)? {
        Ok(result)
      } else {
        Ok(Expr::Nil)
      }
    } else {
      Ok(Expr::Nil)
    }
  }

  fn eval_call(&mut self, call: String) -> Result<Option<Expr>, String> {
    match call.as_str() {
      "+" => {
        let a = self.pop_eval()?;
        let b = self.pop_eval()?;
        if let (Expr::Integer(a), Expr::Integer(b)) = (a, b) {
          Ok(Some(Expr::Integer(a + b)))
        } else {
          Err(format!("Invalid args for: {}", call))
        }
      }
      "-" => {
        let a = self.pop_eval()?;
        let b = self.pop_eval()?;
        if let (Expr::Integer(a), Expr::Integer(b)) = (a, b) {
          Ok(Some(Expr::Integer(a - b)))
        } else {
          Err(format!("Invalid args for: {}", call))
        }
      }
      "*" => {
        let a = self.pop_eval()?;
        let b = self.pop_eval()?;
        if let (Expr::Integer(a), Expr::Integer(b)) = (a, b) {
          Ok(Some(Expr::Integer(a * b)))
        } else {
          Err(format!("Invalid args for: {}", call))
        }
      }
      "/" => {
        let a = self.pop_eval()?;
        let b = self.pop_eval()?;
        if let (Expr::Integer(a), Expr::Integer(b)) = (a, b) {
          Ok(Some(Expr::Integer(a / b)))
        } else {
          Err(format!("Invalid args for: {}", call))
        }
      }
      "set" => {
        let a = self.pop_eval()?;
        let b = self.pop_eval()?;
        if let (Expr::Symbol(a), b) = (a, b) {
          self.scope.insert(a, b);
          Ok(None)
        } else {
          Err(format!("Invalid args for: {}", call))
        }
      }
      "unset" => {
        let a = self.pop_eval()?;
        if let Expr::Symbol(a) = a {
          self.scope.remove(&a);
          Ok(None)
        } else {
          Err(format!("Invalid args for: {}", call))
        }
      }
      "clear" => {
        self.stack.clear();
        Ok(None)
      }
      "pop" => {
        self.stack.pop();
        Ok(None)
      }
      "dup" => {
        if self.stack.is_empty() {
          return Err("Not enough items on stack".to_string());
        }

        let a = self.pop_eval()?;
        self.stack.push(a.clone());
        self.stack.push(a);
        Ok(None)
      }
      "swap" => {
        if self.stack.len() < 2 {
          return Err("Not enough items on stack".to_string());
        }

        let a = self.pop_eval()?;
        let b = self.pop_eval()?;
        self.stack.push(a);
        self.stack.push(b);
        Ok(None)
      }
      "iswap" => {
        if self.stack.len() < 2 {
          return Err("Not enough items on stack".to_string());
        }

        let index = self.pop_eval()?;
        if let Expr::Integer(index) = index {
          let len = self.stack.len();
          self.stack.swap(len - 1, index as usize);
          Ok(None)
        } else {
          Err(format!("Invalid args for: {}", call))
        }
      }
      "call" => {
        let a = self.stack.pop();

        if let Some(Expr::Symbol(a)) | Some(Expr::Call(a)) = a {
          self.eval_expr(Expr::Call(a))
        } else {
          Err(format!("Invalid args for: {}", call))
        }
      }
      "unwrap" => {
        let a = self.pop_eval()?;
        if let Expr::List(a) | Expr::Block(a) = a {
          for expr in a {
            self.stack.push(expr);
          }
          Ok(None)
        } else {
          Err(format!("Invalid args for: {}", call))
        }
      }
      _ => {
        if let Some(value) = self.scope.get(&call) {
          self.eval_expr(value.clone())
        } else {
          Err(format!("Unknown call: {}", call))
        }
      }
    }
  }

  fn eval_expr(&mut self, expr: Expr) -> Result<Option<Expr>, String> {
    match expr {
      Expr::Call(call) => self.eval_call(call),
      Expr::List(list) => {
        // let exprs: Vec<Expr> = list
        //   .into_iter()
        //   .filter_map(|expr| self.eval_expr(expr))
        //   .collect();
        let maybe_exprs = list
          .into_iter()
          .filter_map(|expr| self.eval_expr(expr).transpose())
          .try_fold(Vec::new(), |mut acc, expr| {
            acc.push(expr?);
            Ok(acc)
          });

        match maybe_exprs {
          Err(err) => Err(err),
          Ok(exprs) => Ok(Some(Expr::List(exprs))),
        }
      }
      _ => Ok(Some(expr)),
    }
  }

  pub fn eval_string(&mut self, line: String) -> Result<(), String> {
    let tokens = crate::lex(line);
    let exprs = crate::parse(tokens);

    self.eval(exprs)
  }

  pub fn eval(&mut self, exprs: Vec<Expr>) -> Result<(), String> {
    let mut clone = self.clone();

    for expr in exprs {
      let result = clone.eval_expr(expr)?;

      if let Some(expr) = result {
        clone.stack.push(expr);
      }
    }

    // TODO: Store each scope & stack op as a transaction and just rollback atomically if something happens instead of cloning
    self.stack = clone.stack;
    self.scope = clone.scope;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod eval {
    use super::*;

    #[test]
    fn implicitly_adds_to_stack() {
      let mut program = Program::new();
      program.eval_string("1 2".to_string()).unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(1), Expr::Integer(2)]);
    }

    #[test]
    fn symbols_are_pushed() {
      let mut program = Program::new();
      program.eval_string("'a".to_string()).unwrap();
      assert_eq!(program.stack, vec![Expr::Symbol("a".to_string())]);
    }

    #[test]
    fn add_two_numbers() {
      let mut program = Program::new();
      program.eval_string("1 2 +".to_string()).unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(3)]);
    }

    #[test]
    fn complex_operations() {
      let mut program = Program::new();
      program.eval_string("1 2 + 3 *".to_string()).unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(9)]);
    }

    #[test]
    fn eval_from_stack() {
      let mut program = Program::new();
      program.eval_string("1 2 '+ call".to_string()).unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(3)]);
    }

    #[test]
    fn dont_eval_blocks() {
      let mut program = Program::new();
      program.eval_string("6 'var set (var)".to_string()).unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::Block(vec![Expr::Call("var".to_string())])]
      );
    }

    #[test]
    fn dont_eval_blocks_symbols() {
      let mut program = Program::new();
      program
        .eval_string("6 'var set ('var)".to_string())
        .unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::Block(vec![Expr::Symbol("var".to_string())])]
      );
    }

    #[test]
    fn eval_lists() {
      let mut program = Program::new();
      program.eval_string("[1 2 3]".to_string()).unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::List(vec![
          Expr::Integer(1),
          Expr::Integer(2),
          Expr::Integer(3)
        ])]
      );
    }

    #[test]
    fn eval_lists_eagerly() {
      let mut program = Program::new();
      program.eval_string("6 'var set [var]".to_string()).unwrap();
      assert_eq!(program.stack, vec![Expr::List(vec![Expr::Integer(6)])]);
    }
  }

  mod variables {
    use super::*;

    #[test]
    fn storing_variables() {
      let mut program = Program::new();
      program.eval_string("1 'a set".to_string()).unwrap();
      assert_eq!(
        program.scope,
        HashMap::from_iter(vec![("a".to_string(), Expr::Integer(1))])
      );
    }

    #[test]
    fn retrieving_variables() {
      let mut program = Program::new();
      program.eval_string("1 'a set a".to_string()).unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(1)]);
    }

    #[test]
    fn evaluating_variables() {
      let mut program = Program::new();
      program.eval_string("1 'a set a 2 +".to_string()).unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(3)]);
    }

    #[test]
    fn removing_variables() {
      let mut program = Program::new();
      program
        .eval_string("1 'a set 'a unset".to_string())
        .unwrap();
      assert_eq!(program.scope, HashMap::new());
    }
  }

  mod stack_ops {
    use super::*;

    #[test]
    fn clearing_stack() {
      let mut program = Program::new();
      program.eval_string("1 2 clear".to_string()).unwrap();
      assert_eq!(program.stack, vec![]);
    }

    #[test]
    fn popping_from_stack() {
      let mut program = Program::new();
      program.eval_string("1 2 pop".to_string()).unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(1)]);
    }

    #[test]
    fn duplicating_stack_item() {
      let mut program = Program::new();
      program.eval_string("1 dup".to_string()).unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(1), Expr::Integer(1)]);
    }

    #[test]
    fn swapping_stack_items() {
      let mut program = Program::new();
      program.eval_string("1 2 swap".to_string()).unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(2), Expr::Integer(1)]);
    }

    #[test]
    fn swapping_with_index() {
      let mut program = Program::new();
      program.eval_string("1 2 3 4 0 iswap".to_string()).unwrap();
      assert_eq!(
        program.stack,
        vec![
          Expr::Integer(4),
          Expr::Integer(2),
          Expr::Integer(3),
          Expr::Integer(1)
        ]
      );
    }

    #[test]
    fn swapping_with_index2() {
      let mut program = Program::new();
      program.eval_string("1 2 3 4 1 iswap".to_string()).unwrap();
      assert_eq!(
        program.stack,
        vec![
          Expr::Integer(1),
          Expr::Integer(4),
          Expr::Integer(3),
          Expr::Integer(2)
        ]
      );
    }
  }
}
