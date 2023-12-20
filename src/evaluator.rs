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
        if let (Expr::Integer(a), Expr::Integer(b)) = (a.clone(), b.clone()) {
          Ok(Some(Expr::Integer(a + b)))
        } else {
          Err(format!("Invalid args for: {} found {:?} {:?}", call, a, b))
        }
      }
      "-" => {
        let b = self.pop_eval()?;
        let a = self.pop_eval()?;
        if let (Expr::Integer(a), Expr::Integer(b)) = (a.clone(), b.clone()) {
          Ok(Some(Expr::Integer(a - b)))
        } else {
          Err(format!("Invalid args for: {} found {:?} {:?}", call, a, b))
        }
      }
      "*" => {
        let b = self.pop_eval()?;
        let a = self.pop_eval()?;
        if let (Expr::Integer(a), Expr::Integer(b)) = (a.clone(), b.clone()) {
          Ok(Some(Expr::Integer(a * b)))
        } else {
          Err(format!("Invalid args for: {} found {:?} {:?}", call, a, b))
        }
      }
      "/" => {
        let b = self.pop_eval()?;
        let a = self.pop_eval()?;
        if let (Expr::Integer(a), Expr::Integer(b)) = (a.clone(), b.clone()) {
          Ok(Some(Expr::Integer(a / b)))
        } else {
          Err(format!("Invalid args for: {} found {:?} {:?}", call, a, b))
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
      ">" => {
        let b = self.pop_eval()?;
        let a = self.pop_eval()?;
        Ok(Some(Expr::Boolean(a.gt(&b))))
      }
      "<" => {
        let b = self.pop_eval()?;
        let a = self.pop_eval()?;
        Ok(Some(Expr::Boolean(a.lt(&b))))
      }
      "explode" => {
        let string = self.pop_eval()?;
        if let Expr::String(string) = string.clone() {
          let mut chars = vec![];
          for c in string.chars() {
            chars.push(Expr::String(c.to_string()));
          }
          Ok(Some(Expr::List(chars)))
        } else {
          Err(format!("Invalid args for: {} found {:?}", call, string))
        }
      }
      "len" => {
        let list = self.pop_eval()?;
        if let Expr::List(list) = list {
          Ok(Some(Expr::Integer(list.len() as i64)))
        } else {
          Err(format!("Invalid args for: {} found {:?}", call, list))
        }
      }
      "nth" => {
        let index = self.pop_eval()?;
        let list = self.pop_eval()?;
        if let (Expr::Integer(index), Expr::List(list)) =
          (index.clone(), list.clone())
        {
          let index = index as usize;
          if index < list.len() {
            Ok(Some(list[index].clone()))
          } else {
            Err(format!("Index out of bounds: {}", index))
          }
        } else {
          Err(format!(
            "Invalid args for: {} found {:?} {:?}",
            call, list, index
          ))
        }
      }
      "join" => {
        let delimiter = self.pop_eval()?;
        let list = self.pop_eval()?;
        if let (Expr::String(delimiter), Expr::List(list)) =
          (delimiter.clone(), list.clone())
        {
          let mut string = String::new();
          for (i, item) in list.iter().enumerate() {
            if i > 0 {
              string.push_str(&delimiter);
            }
            string.push_str(item.to_string().as_str());
          }
          Ok(Some(Expr::String(string)))
        } else {
          Err(format!(
            "Invalid args for: {} found {:?} {:?}",
            call, list, delimiter
          ))
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
          Err(format!("Invalid args for: {} found {:?}", call, list))
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
          Err(format!("Invalid args for: {} found {:?}", call, list))
        }
      }
      "concat" => {
        let b = self.pop_eval()?;
        let a = self.pop_eval()?;
        if let (Expr::List(a), Expr::List(b)) = (a.clone(), b.clone()) {
          let mut a = a;
          a.extend(b);
          Ok(Some(Expr::List(a)))
        } else {
          Err(format!("Invalid args for: {} found {:?} {:?}", call, a, b))
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
        ) = (condition.clone(), block.clone(), else_block.clone())
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
          Err(format!(
            "Invalid args for: {} found {:?} {:?} {:?}",
            call, else_block, block, condition
          ))
        }
      }
      "if" => {
        let condition = self.pop_eval()?;
        let block = self.pop_eval()?;
        if let (Expr::Block(condition), Expr::Block(block)) =
          (condition.clone(), block.clone())
        {
          match self.eval(vec![
            Expr::Block(vec![]),
            Expr::Block(block),
            Expr::Block(condition),
            Expr::Call("ifelse".to_string()),
          ]) {
            Ok(_) => Ok(None),
            Err(err) => Err(format!("Error in if condition: {}", err)),
          }
        } else {
          Err(format!(
            "Invalid args for: {} found {:?} {:?}",
            call, block, condition
          ))
        }
      }
      "while" => {
        let condition = self.pop_eval()?;
        let block = self.pop_eval()?;

        if let (Expr::Block(condition), Expr::Block(block)) = (condition, block)
        {
          loop {
            let mut block = block.clone();
            block.push(Expr::Boolean(true));

            match self.eval(vec![
              Expr::Block(vec![Expr::Boolean(false)]),
              Expr::Block(block),
              Expr::Block(condition.clone()),
              Expr::Call("ifelse".to_string()),
            ]) {
              Ok(_) => {
                let bool = self.pop_eval()?;

                if !bool.is_truthy() {
                  break;
                }
              }
              Err(err) => {
                return Err(format!("Error in while condition: {}", err))
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
        let symbol = self.pop_eval()?;
        let value = self.pop_eval()?;
        if let (Expr::Symbol(symbol), value) = (symbol.clone(), value.clone()) {
          self.scope.insert(symbol, value);
          Ok(None)
        } else {
          Err(format!(
            "Invalid args for: {} found {:?} {:?}",
            call, value, symbol
          ))
        }
      }
      "unset" => {
        let symbol = self.pop_eval()?;
        if let Expr::Symbol(symbol) = symbol {
          self.scope.remove(&symbol);
          Ok(None)
        } else {
          Err(format!("Invalid args for: {} found {:?}", call, symbol))
        }
      }
      "collect" => Ok(Some(Expr::List(core::mem::take(&mut self.stack)))),
      "tostring" => {
        let a = self.pop_eval()?;
        Ok(Some(Expr::String(a.to_string())))
      }
      "tosymbol" => {
        let a = self.pop_eval()?;
        Ok(Some(Expr::Symbol(a.to_string())))
      }
      "tointeger" => {
        let a = self.pop_eval()?;
        match a.to_string().parse() {
          Ok(a) => Ok(Some(Expr::Integer(a))),
          Err(err) => Err(format!("Error parsing integer: {}", err)),
        }
      }
      "typeof" => {
        let a = self.pop_eval()?;
        Ok(Some(Expr::String(a.type_of())))
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
          Err(format!("Invalid args for: {} found {:?}", call, a))
        }
      }
      "unwrap" => {
        let list = self.pop_eval()?;
        if let Expr::List(list) | Expr::Block(list) = list {
          for expr in list {
            self.stack.push(expr);
          }
          Ok(None)
        } else {
          Err(format!("Invalid args for: {} found {:?}", call, list))
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

  pub fn eval_string(&mut self, line: &str) -> Result<(), String> {
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
      program.eval_string("1 2").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(1), Expr::Integer(2)]);
    }

    #[test]
    fn symbols_are_pushed() {
      let mut program = Program::new();
      program.eval_string("'a").unwrap();
      assert_eq!(program.stack, vec![Expr::Symbol("a".to_string())]);
    }

    #[test]
    fn add_two_numbers() {
      let mut program = Program::new();
      program.eval_string("1 2 +").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(3)]);
    }

    #[test]
    fn complex_operations() {
      let mut program = Program::new();
      program.eval_string("1 2 + 3 *").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(9)]);
    }

    #[test]
    fn eval_from_stack() {
      let mut program = Program::new();
      program.eval_string("1 2 '+ call").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(3)]);
    }

    #[test]
    fn dont_eval_blocks() {
      let mut program = Program::new();
      program.eval_string("6 'var set (var)").unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::Block(vec![Expr::Call("var".to_string())])]
      );
    }

    #[test]
    fn dont_eval_blocks_symbols() {
      let mut program = Program::new();
      program.eval_string("6 'var set ('var)").unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::Block(vec![Expr::Symbol("var".to_string())])]
      );
    }

    #[test]
    fn eval_lists() {
      let mut program = Program::new();
      program.eval_string("[1 2 3]").unwrap();
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
      program.eval_string("6 'var set [var]").unwrap();
      assert_eq!(program.stack, vec![Expr::List(vec![Expr::Integer(6)])]);
    }
  }

  mod comparison {
    use super::*;

    mod greater_than {
      use super::*;

      #[test]
      fn greater_than_int() {
        let mut program = Program::new();
        program.eval_string("1 1 >").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new();
        program.eval_string("1 2 >").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new();
        program.eval_string("2 1 >").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);
      }

      #[test]
      fn greater_than_float() {
        let mut program = Program::new();
        program.eval_string("1.0 1.0 >").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new();
        program.eval_string("1.0 1.1 >").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new();
        program.eval_string("1.1 1.0 >").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);
      }

      #[test]
      fn greater_than_int_and_float() {
        // Int first
        let mut program = Program::new();
        program.eval_string("1 1.0 >").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new();
        program.eval_string("1 1.1 >").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new();
        program.eval_string("2 1.0 >").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);

        // Float first
        let mut program = Program::new();
        program.eval_string("1.0 1 >").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new();
        program.eval_string("1.0 1 >").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new();
        program.eval_string("1.1 1 >").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);
      }
    }

    mod less_than {
      use super::*;

      #[test]
      fn less_than_int() {
        let mut program = Program::new();
        program.eval_string("1 1 <").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new();
        program.eval_string("1 2 <").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);

        let mut program = Program::new();
        program.eval_string("2 1 <").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);
      }

      #[test]
      fn less_than_float() {
        let mut program = Program::new();
        program.eval_string("1.0 1.0 <").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new();
        program.eval_string("1.0 1.1 <").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);

        let mut program = Program::new();
        program.eval_string("1.1 1.0 <").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);
      }

      #[test]
      fn less_than_int_and_float() {
        // Int first
        let mut program = Program::new();
        program.eval_string("1 1.0 <").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new();
        program.eval_string("1 1.1 <").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);

        let mut program = Program::new();
        program.eval_string("2 1.0 <").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        // Float first
        let mut program = Program::new();
        program.eval_string("1.0 1 <").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new();
        program.eval_string("0.9 1 <").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);

        let mut program = Program::new();
        program.eval_string("1.1 1 <").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);
      }
    }
  }

  mod variables {
    use super::*;

    #[test]
    fn storing_variables() {
      let mut program = Program::new();
      program.eval_string("1 'a set").unwrap();
      assert_eq!(
        program.scope,
        HashMap::from_iter(vec![("a".to_string(), Expr::Integer(1))])
      );
    }

    #[test]
    fn retrieving_variables() {
      let mut program = Program::new();
      program.eval_string("1 'a set a").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(1)]);
    }

    #[test]
    fn evaluating_variables() {
      let mut program = Program::new();
      program.eval_string("1 'a set a 2 +").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(3)]);
    }

    #[test]
    fn removing_variables() {
      let mut program = Program::new();
      program.eval_string("1 'a set 'a unset").unwrap();
      assert_eq!(program.scope, HashMap::new());
    }

    #[test]
    fn collect() {
      let mut program = Program::new();
      program.eval_string("1 2 3 collect").unwrap();
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
        .eval_string("1 2 3 collect 'a set a unwrap")
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
      program.eval_string("1 2 clear").unwrap();
      assert_eq!(program.stack, vec![]);
    }

    #[test]
    fn popping_from_stack() {
      let mut program = Program::new();
      program.eval_string("1 2 pop").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(1)]);
    }

    #[test]
    fn duplicating_stack_item() {
      let mut program = Program::new();
      program.eval_string("1 dup").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(1), Expr::Integer(1)]);
    }

    #[test]
    fn swapping_stack_items() {
      let mut program = Program::new();
      program.eval_string("1 2 swap").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(2), Expr::Integer(1)]);
    }
  }

  mod list_ops {
    use super::*;

    #[test]
    fn inserting_into_list() {
      let mut program = Program::new();
      program.eval_string("[1 2] 3 insert").unwrap();
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
      program.eval_string("[1 2] last").unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::List(vec![Expr::Integer(1)]), Expr::Integer(2)]
      );
    }

    #[test]
    fn concatenating_lists() {
      let mut program = Program::new();
      program.eval_string("[1 2] [3 \"4\"] concat").unwrap();
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
      program.eval_string("[1 2 3] len").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(3)]);
    }

    #[test]
    fn getting_nth_item_of_list() {
      let mut program = Program::new();
      program.eval_string("[1 2 3] 1 nth").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(2)]);
    }
  }

  mod string_ops {
    use super::*;

    #[test]
    fn exploding_string() {
      let mut program = Program::new();
      program.eval_string("\"abc\" explode").unwrap();
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
        .eval_string("[\"a\" 3 \"hello\" 1.2] \"\" join")
        .unwrap();

      assert_eq!(program.stack, vec![Expr::String("a3hello1.2".to_string())]);
    }
  }

  mod control_flow {
    use super::*;

    #[test]
    fn if_true() {
      let mut program = Program::new();
      program.eval_string("1 2 + (\"correct\") (3 =) if").unwrap();
      assert_eq!(program.stack, vec![Expr::String("correct".to_owned())]);
    }

    #[test]
    fn if_empty_condition() {
      let mut program = Program::new();
      program
        .eval_string("1 2 + 3 = (\"correct\") () if")
        .unwrap();
      assert_eq!(program.stack, vec![Expr::String("correct".to_owned())]);
    }

    #[test]
    fn if_else_true() {
      let mut program = Program::new();
      program
        .eval_string("1 2 + 3 = (\"incorrect\") (\"correct\") () ifelse")
        .unwrap();
      assert_eq!(program.stack, vec![Expr::String("correct".to_owned())]);
    }

    #[test]
    fn if_else_false() {
      let mut program = Program::new();
      program
        .eval_string("1 2 + 2 = (\"incorrect\") (\"correct\") () ifelse")
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
           ) while",
        )
        .unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::Integer(2), Expr::Integer(1), Expr::Integer(0)]
      );
    }
  }

  mod type_ops {
    use super::*;

    #[test]
    fn to_string() {
      let mut program = Program::new();
      program.eval_string("1 tostring").unwrap();
      assert_eq!(program.stack, vec![Expr::String("1".to_string())]);
    }

    #[test]
    fn to_symbol() {
      let mut program = Program::new();
      program.eval_string("\"a\" tosymbol").unwrap();
      assert_eq!(program.stack, vec![Expr::Symbol("a".to_string())]);
    }

    #[test]
    fn to_integer() {
      let mut program = Program::new();
      program.eval_string("\"1\" tointeger").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(1)]);
    }

    #[test]
    fn type_of() {
      let mut program = Program::new();
      program.eval_string("1 typeof").unwrap();
      assert_eq!(program.stack, vec![Expr::String("integer".to_string())]);
    }
  }
}
