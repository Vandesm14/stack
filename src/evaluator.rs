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
        let b = self.pop_eval()?;
        let a = self.pop_eval()?;
        if let (Expr::Integer(a), Expr::Integer(b)) = (a, b) {
          Ok(Some(Expr::Integer(a + b)))
        } else {
          Err(format!("Invalid args for: {}", call))
        }
      }
      "-" => {
        let b = self.pop_eval()?;
        let a = self.pop_eval()?;
        if let (Expr::Integer(a), Expr::Integer(b)) = (a, b) {
          Ok(Some(Expr::Integer(a - b)))
        } else {
          Err(format!("Invalid args for: {}", call))
        }
      }
      "*" => {
        let b = self.pop_eval()?;
        let a = self.pop_eval()?;
        if let (Expr::Integer(a), Expr::Integer(b)) = (a, b) {
          Ok(Some(Expr::Integer(a * b)))
        } else {
          Err(format!("Invalid args for: {}", call))
        }
      }
      "/" => {
        let b = self.pop_eval()?;
        let a = self.pop_eval()?;
        if let (Expr::Integer(a), Expr::Integer(b)) = (a, b) {
          Ok(Some(Expr::Integer(a / b)))
        } else {
          Err(format!("Invalid args for: {}", call))
        }
      }
      "=" => {
        let b = self.pop_eval()?;
        let a = self.pop_eval()?;
        Ok(Some(Expr::Boolean(a.eq(&b))))
      }
      "!=" => {
        let b = self.pop_eval()?;
        let a = self.pop_eval()?;
        Ok(Some(Expr::Boolean(!a.eq(&b))))
      }
      "explode" => {
        let a = self.pop_eval()?;
        if let Expr::String(a) = a {
          let mut chars = vec![];
          for c in a.chars() {
            chars.push(Expr::String(c.to_string()));
          }
          Ok(Some(Expr::List(chars)))
        } else {
          Err(format!("Invalid args for: {}", call))
        }
      }
      "len" => {
        let a = self.pop_eval()?;
        if let Expr::List(a) = a {
          Ok(Some(Expr::Integer(a.len() as i64)))
        } else {
          Err(format!("Invalid args for: {}", call))
        }
      }
      "nth" => {
        let index = self.pop_eval()?;
        let list = self.pop_eval()?;
        if let (Expr::Integer(index), Expr::List(list)) = (index, list) {
          let index = index as usize;
          if index < list.len() {
            Ok(Some(list[index].clone()))
          } else {
            Err(format!("Index out of bounds: {}", index))
          }
        } else {
          Err(format!("Invalid args for: {}", call))
        }
      }
      "join" => {
        let delimiter = self.pop_eval()?;
        let list = self.pop_eval()?;
        if let (Expr::String(delimiter), Expr::List(list)) = (delimiter, list) {
          let mut string = String::new();
          for (i, item) in list.iter().enumerate() {
            if i > 0 {
              string.push_str(&delimiter);
            }
            string.push_str(item.to_string().as_str());
          }
          Ok(Some(Expr::String(string)))
        } else {
          Err(format!("Invalid args for: {}", call))
        }
      }
      // Pushes the last value in the stack into the list
      "insert" => {
        let item = self.pop_eval()?;
        let list = self.pop_eval()?;
        if let Expr::List(list) = list {
          let mut list = list;
          list.push(item);
          Ok(Some(Expr::List(list)))
        } else {
          Err(format!("Invalid args for: {}", call))
        }
      }
      // Pops the last value of a list onto the stack
      "last" => {
        let list = self.pop_eval()?;
        if let Expr::List(list) = list {
          let mut list = list;
          let item = list.pop();
          if let Some(item) = item {
            self.stack.push(Expr::List(list));
            Ok(Some(item))
          } else {
            Ok(None)
          }
        } else {
          Err(format!("Invalid args for: {}", call))
        }
      }
      "concat" => {
        let a = self.pop_eval()?;
        let b = self.pop_eval()?;
        if let (Expr::List(a), Expr::List(b)) = (a, b) {
          let mut b = b;
          b.extend(a);
          Ok(Some(Expr::List(b)))
        } else {
          Err(format!("Invalid args for: {}", call))
        }
      }
      "if" => {
        let condition = self.pop_eval()?;
        let block = self.pop_eval()?;
        if let (Expr::Block(condition), Expr::Block(block)) = (condition, block)
        {
          let result = self.eval(condition);
          if result.is_ok() {
            let bool = self.pop_eval()?;

            if bool.is_truthy() {
              self.eval(block)?;
            }

            Ok(None)
          } else {
            Err(format!("Error in if condition: {}", result.unwrap_err()))
          }
        } else {
          Err(format!("Invalid args for: {}", call))
        }
      }
      "ifelse" => {
        let condition = self.pop_eval()?;
        let block = self.pop_eval()?;
        let else_block = self.pop_eval()?;
        if let (
          Expr::Block(condition),
          Expr::Block(block),
          Expr::Block(else_block),
        ) = (condition, block, else_block)
        {
          let result = self.eval(condition);
          if result.is_ok() {
            let bool = self.pop_eval()?;

            if bool.is_truthy() {
              self.eval(block)?;
            } else {
              self.eval(else_block)?;
            }

            Ok(None)
          } else {
            Err(format!("Error in if condition: {}", result.unwrap_err()))
          }
        } else {
          Err(format!("Invalid args for: {}", call))
        }
      }
      "while" => {
        let condition = self.pop_eval()?;
        let block = self.pop_eval()?;

        if let (Expr::Block(condition), Expr::Block(block)) = (condition, block)
        {
          loop {
            let result = self.eval(condition.clone());
            if result.is_ok() {
              let bool = self.pop_eval()?;

              if bool.is_truthy() {
                self.eval(block.clone())?;
              } else {
                break;
              }
            }
          }
        }

        Ok(None)
      }
      "print" => {
        let a = self.pop_eval()?;
        println!("{}", a.to_string());
        Ok(None)
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
      "collect" => Ok(Some(Expr::List(core::mem::take(&mut self.stack)))),
      "tostring" => {
        let a = self.pop_eval()?;
        Ok(Some(Expr::String(a.to_string())))
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
      "call" => {
        let a = self.stack.pop();

        if let Some(Expr::Symbol(a)) | Some(Expr::Call(a)) = a {
          self.eval_expr(Expr::Call(a))
        } else if let Some(Expr::Block(a)) = a {
          self.eval(a)?;
          Ok(None)
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

    #[test]
    fn collect() {
      let mut program = Program::new();
      program.eval_string("1 2 3 collect".to_string()).unwrap();
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
    fn collect_and_unwrap() {
      let mut program = Program::new();
      program
        .eval_string("1 2 3 collect 'a set a unwrap".to_string())
        .unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::Integer(1), Expr::Integer(2), Expr::Integer(3)]
      );
      assert_eq!(
        program.scope,
        HashMap::from_iter(vec![(
          "a".to_string(),
          Expr::List(vec![
            Expr::Integer(1),
            Expr::Integer(2),
            Expr::Integer(3)
          ])
        )])
      );
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
  }

  mod list_ops {
    use super::*;

    #[test]
    fn inserting_into_list() {
      let mut program = Program::new();
      program.eval_string("[1 2] 3 insert".to_string()).unwrap();
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
    fn popping_from_list() {
      let mut program = Program::new();
      program.eval_string("[1 2] last".to_string()).unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::List(vec![Expr::Integer(1)]), Expr::Integer(2)]
      );
    }

    #[test]
    fn concatenating_lists() {
      let mut program = Program::new();
      program
        .eval_string("[1 2] [3 \"4\"] concat".to_string())
        .unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::List(vec![
          Expr::Integer(1),
          Expr::Integer(2),
          Expr::Integer(3),
          Expr::String("4".to_owned())
        ])]
      );
    }

    #[test]
    fn getting_length_of_list() {
      let mut program = Program::new();
      program.eval_string("[1 2 3] len".to_string()).unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(3)]);
    }

    #[test]
    fn getting_nth_item_of_list() {
      let mut program = Program::new();
      program.eval_string("[1 2 3] 1 nth".to_string()).unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(2)]);
    }
  }

  mod string_ops {
    use super::*;

    #[test]
    fn exploding_string() {
      let mut program = Program::new();
      program.eval_string("\"abc\" explode".to_string()).unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::List(vec![
          Expr::String("a".to_string()),
          Expr::String("b".to_string()),
          Expr::String("c".to_string())
        ])]
      );
    }

    #[test]
    fn joining_to_string() {
      let mut program = Program::new();
      program
        .eval_string("[\"a\" 3 \"hello\" 1.2] \"\" join".to_string())
        .unwrap();

      assert_eq!(program.stack, vec![Expr::String("a3hello1.2".to_string())]);
    }
  }

  mod control_flow {
    use super::*;

    #[test]
    fn if_true() {
      let mut program = Program::new();
      program
        .eval_string("1 2 + (\"correct\") (3 =) if".to_string())
        .unwrap();
      assert_eq!(program.stack, vec![Expr::String("correct".to_owned())]);
    }

    #[test]
    fn if_empty_condition() {
      let mut program = Program::new();
      program
        .eval_string("1 2 + 3 = (\"correct\") () if".to_string())
        .unwrap();
      assert_eq!(program.stack, vec![Expr::String("correct".to_owned())]);
    }

    #[test]
    fn if_else_true() {
      let mut program = Program::new();
      program
        .eval_string(
          "1 2 + 3 = (\"incorrect\") (\"correct\") () ifelse".to_string(),
        )
        .unwrap();
      assert_eq!(program.stack, vec![Expr::String("correct".to_owned())]);
    }

    #[test]
    fn if_else_false() {
      let mut program = Program::new();
      program
        .eval_string(
          "1 2 + 2 = (\"incorrect\") (\"correct\") () ifelse".to_string(),
        )
        .unwrap();
      assert_eq!(program.stack, vec![Expr::String("incorrect".to_owned())]);
    }
  }

  mod loops {
    use super::*;

    #[test]
    fn while_loop() {
      let mut program = Program::new();
      program
        .eval_string(
          ";; Set i to 3
           3 'i set

           (
             ;; Decrement i by 1
             i 1 -
             ;; Set i
             'i set

             i
           ) (
             ;; If i is 0, break
             i 0 !=
           ) while"
            .to_owned(),
        )
        .unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::Integer(2), Expr::Integer(1), Expr::Integer(0)]
      );
    }
  }
}
