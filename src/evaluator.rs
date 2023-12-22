use crate::Expr;
use core::fmt;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Program {
  pub stack: Vec<Expr>,
  pub scope: Vec<HashMap<String, Expr>>,
}

impl fmt::Display for Program {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Stack: [")?;

    self.stack.iter().enumerate().try_for_each(|(i, expr)| {
      if i == self.stack.len() - 1 {
        write!(f, "{}", expr)
      } else {
        write!(f, "{}, ", expr)
      }
    })?;
    write!(f, "]")?;

    writeln!(f,)?;

    if !self.scope.is_empty() {
      writeln!(f, "Scope:")?;

      let layers = self.scope.len();
      for (layer_i, layer) in self.scope.iter().enumerate() {
        let items = layer.len();
        writeln!(f, "Layer {}:", layer_i)?;
        for (item_i, (key, value)) in layer.iter().enumerate() {
          if item_i == items - 1 && layer_i == layers - 1 {
            write!(f, " + {}: {}", key, value)?;
          } else {
            writeln!(f, " + {}: {}", key, value)?;
          }
        }
      }
    }

    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct EvalError {
  program: Program,
  message: String,
  expr: Expr,
}

impl fmt::Display for EvalError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "Error: {}", self.message)?;
    writeln!(f, "Expr: {}", self.expr)?;
    writeln!(f,)?;
    write!(f, "{}", self.program)
  }
}

impl Program {
  pub fn new() -> Self {
    Self {
      stack: vec![],
      scope: vec![HashMap::new()],
    }
  }

  fn pop(&mut self) -> Option<Expr> {
    self.stack.pop()
  }

  fn scope_item(&self, symbol: &str) -> Option<Expr> {
    let len = self.scope.len();
    let is_scoped = symbol.starts_with('@');
    let symbol = if is_scoped {
      symbol.replace('@', format!("__{}@", len - 1).as_str())
    } else {
      symbol.to_owned()
    };

    for layer in self.scope.iter().rev() {
      if let Some(item) = layer.get(&symbol) {
        return Some(item.clone());
      }
    }

    None
  }

  fn scope_item_layer(&self, symbol: &str) -> Option<usize> {
    for (layer_i, layer) in self.scope.iter().rev().enumerate() {
      if layer.contains_key(symbol) {
        return Some(layer_i);
      }
    }

    None
  }

  fn set_scope_item(&mut self, symbol: &str, value: Expr) {
    let len = self.scope.len();
    if let Some(layer) = self.scope.last_mut() {
      let is_scoped = symbol.starts_with('@');
      let symbol = if is_scoped {
        symbol.replace('@', format!("__{}@", len - 1).as_str())
      } else {
        symbol.to_owned()
      };

      layer.insert(symbol.to_string(), value);
    } else {
      panic!("No scope to set item in. Maybe there's an extra \"}}\"?");
    }
  }

  fn remove_scope_item(&mut self, symbol: &str) {
    let layer = self.scope_item_layer(symbol);

    if let Some(layer) = layer {
      self.scope[layer].remove(symbol);
    }
  }

  fn push_scope(&mut self) {
    self.scope.push(HashMap::new());
  }

  fn pop_scope(&mut self) {
    self.scope.pop();
  }

  fn eval_call(&mut self, call: String) -> Result<Option<Expr>, EvalError> {
    match call.as_str() {
      "+" => {
        let b = self.pop();
        let a = self.pop();
        if let (Some(Expr::Integer(a)), Some(Expr::Integer(b))) =
          (a.clone(), b.clone())
        {
          Ok(Some(Expr::Integer(a + b)))
        } else if let (Some(Expr::String(a)), Some(Expr::String(b))) =
          (a.clone(), b.clone())
        {
          Ok(Some(Expr::String(format!("{}{}", a, b))))
        } else if let (Some(Expr::Float(a)), Some(Expr::Float(b))) =
          (a.clone(), b.clone())
        {
          Ok(Some(Expr::Float(a + b)))
        } else {
          Err(EvalError {
            expr: Expr::Call(call.clone()),
            program: self.clone(),
            message: format!("Invalid args: [{:?} {:?}]", a, b),
          })
        }
      }
      "-" => {
        let b = self.pop();
        let a = self.pop();
        if let (Some(Expr::Integer(a)), Some(Expr::Integer(b))) =
          (a.clone(), b.clone())
        {
          Ok(Some(Expr::Integer(a - b)))
        } else if let (Some(Expr::Float(a)), Some(Expr::Float(b))) =
          (a.clone(), b.clone())
        {
          Ok(Some(Expr::Float(a - b)))
        } else {
          Err(EvalError {
            expr: Expr::Call(call.clone()),
            program: self.clone(),
            message: format!("Invalid args: [{:?} {:?}]", a, b),
          })
        }
      }
      "*" => {
        let b = self.pop();
        let a = self.pop();
        if let (Some(Expr::Integer(a)), Some(Expr::Integer(b))) =
          (a.clone(), b.clone())
        {
          Ok(Some(Expr::Integer(a * b)))
        } else if let (Some(Expr::Float(a)), Some(Expr::Float(b))) =
          (a.clone(), b.clone())
        {
          Ok(Some(Expr::Float(a * b)))
        } else {
          Err(EvalError {
            expr: Expr::Call(call.clone()),
            program: self.clone(),
            message: format!("Invalid args:[{:?} {:?}]", a, b),
          })
        }
      }
      "/" => {
        let b = self.pop();
        let a = self.pop();
        if let (Some(Expr::Integer(a)), Some(Expr::Integer(b))) =
          (a.clone(), b.clone())
        {
          Ok(Some(Expr::Integer(a / b)))
        } else if let (Some(Expr::Float(a)), Some(Expr::Float(b))) =
          (a.clone(), b.clone())
        {
          Ok(Some(Expr::Float(a / b)))
        } else {
          Err(EvalError {
            expr: Expr::Call(call.clone()),
            program: self.clone(),
            message: format!("Invalid args: [{:?} {:?}]", a, b),
          })
        }
      }
      "%" => {
        let b = self.pop();
        let a = self.pop();
        if let (Some(Expr::Integer(a)), Some(Expr::Integer(b))) =
          (a.clone(), b.clone())
        {
          Ok(Some(Expr::Integer(a % b)))
        } else if let (Some(Expr::Float(a)), Some(Expr::Float(b))) =
          (a.clone(), b.clone())
        {
          Ok(Some(Expr::Float(a % b)))
        } else {
          Err(EvalError {
            expr: Expr::Call(call.clone()),
            program: self.clone(),
            message: format!("Invalid args: [{:?} {:?}]", a, b),
          })
        }
      }
      "=" => {
        let b = self.pop();
        let a = self.pop();
        Ok(Some(Expr::Boolean(a.eq(&b))))
      }
      "!=" => {
        let b = self.pop();
        let a = self.pop();
        Ok(Some(Expr::Boolean(!a.eq(&b))))
      }
      ">" => {
        let b = self.pop();
        let a = self.pop();
        Ok(Some(Expr::Boolean(a.gt(&b))))
      }
      "<" => {
        let b = self.pop();
        let a = self.pop();
        Ok(Some(Expr::Boolean(a.lt(&b))))
      }
      "or" => {
        let b = self.pop();
        let a = self.pop();
        if let (Some(a), Some(b)) = (a.clone(), b.clone()) {
          Ok(Some(Expr::Boolean(a.is_truthy() || b.is_truthy())))
        } else {
          Err(EvalError {
            expr: Expr::Call(call.clone()),
            program: self.clone(),
            message: format!("Invalid args: [{:?} {:?}]", a, b),
          })
        }
      }
      "and" => {
        let b = self.pop();
        let a = self.pop();
        if let (Some(a), Some(b)) = (a.clone(), b.clone()) {
          Ok(Some(Expr::Boolean(a.is_truthy() && b.is_truthy())))
        } else {
          Err(EvalError {
            expr: Expr::Call(call.clone()),
            program: self.clone(),
            message: format!("Invalid args: [{:?} {:?}]", a, b),
          })
        }
      }
      "explode" => {
        let string = self.pop();
        if let Some(Expr::String(string)) = string.clone() {
          let mut chars = vec![];
          for c in string.chars() {
            chars.push(Expr::String(c.to_string()));
          }
          Ok(Some(Expr::List(chars)))
        } else {
          Err(EvalError {
            expr: Expr::Call(call.clone()),
            program: self.clone(),
            message: format!("Invalid args: [{:?}]", string),
          })
        }
      }
      "len" => {
        let list = self.pop();
        if let Some(Expr::List(list)) = list {
          Ok(Some(Expr::Integer(list.len() as i64)))
        } else {
          Err(EvalError {
            expr: Expr::Call(call.clone()),
            program: self.clone(),
            message: format!("Invalid args: [{:?}]", list),
          })
        }
      }
      "parse" => {
        let string = self.pop();
        if let Some(Expr::String(string)) = string {
          let tokens = crate::lex(string.as_str());
          let exprs = crate::parse(tokens);
          Ok(Some(Expr::List(exprs)))
        } else {
          Err(EvalError {
            expr: Expr::Call(call.clone()),
            program: self.clone(),
            message: format!("Invalid args: [{:?}]", string),
          })
        }
      }
      "read-file" => {
        let path = self.pop();
        if let Some(Expr::String(path)) = path {
          let contents = std::fs::read_to_string(path.clone());
          match contents {
            Ok(contents) => Ok(Some(Expr::String(contents))),
            Err(err) => Err(EvalError {
              expr: Expr::Call(call.clone()),
              program: self.clone(),
              message: format!("Error reading [{}]: {}", path, err),
            }),
          }
        } else {
          Err(EvalError {
            expr: Expr::Call(call.clone()),
            program: self.clone(),
            message: format!("Invalid args: [{:?}]", path),
          })
        }
      }
      "import" => {
        self.eval(vec![
          Expr::Call("read-file".to_string()),
          Expr::Call("parse".to_string()),
          Expr::Call("call".to_string()),
        ])?;
        Ok(None)
      }
      "nth" => {
        let index = self.pop();
        let list = self.pop();
        if let (Some(Expr::Integer(index)), Some(Expr::List(list))) =
          (index.clone(), list.clone())
        {
          if index >= 0 && index < list.len() as i64 {
            Ok(Some(list[index as usize].clone()))
          } else if index < 0 {
            let positive_index = -index;
            let positive_index = positive_index as usize;
            let actual_index = list.len().checked_sub(positive_index);

            match actual_index {
              Some(index) => Ok(Some(list[index].clone())),
              None => Err(EvalError {
                expr: Expr::Call(call.clone()),
                program: self.clone(),
                message: format!(
                  "Index {} out of bounds for: {}",
                  index,
                  Expr::List(list)
                ),
              }),
            }
          } else {
            Err(EvalError {
              expr: Expr::Call(call.clone()),
              program: self.clone(),
              message: format!("Index out of bounds: {}", index),
            })
          }
        } else {
          Err(EvalError {
            expr: Expr::Call(call.clone()),
            program: self.clone(),
            message: format!("Invalid args: [{:?} {:?}]", list, index),
          })
        }
      }
      "join" => {
        let delimiter = self.pop();
        let list = self.pop();
        if let (Some(Expr::String(delimiter)), Some(Expr::List(list))) =
          (delimiter.clone(), list.clone())
        {
          let mut string = String::new();
          for (i, item) in list.iter().enumerate() {
            if i > 0 {
              string.push_str(&delimiter);
            }

            match item {
              Expr::String(str) => string.push_str(str),
              _ => string.push_str(item.to_string().as_str()),
            }
          }
          Ok(Some(Expr::String(string)))
        } else {
          Err(EvalError {
            expr: Expr::Call(call.clone()),
            program: self.clone(),
            message: format!("Invalid args: [{:?} {:?}]", list, delimiter),
          })
        }
      }
      // Pushes the last value in the stack into the list
      "insert" => {
        let item = self.pop();
        let list = self.pop();
        if let (Some(Expr::List(list)), Some(item)) =
          (list.clone(), item.clone())
        {
          let mut list = list;
          list.push(item);
          Ok(Some(Expr::List(list)))
        } else {
          Err(EvalError {
            expr: Expr::Call(call.clone()),
            program: self.clone(),
            message: format!("Invalid args: [{:?}]", list),
          })
        }
      }
      // Pops the last value of a list onto the stack
      "last" => {
        let list = self.pop();
        if let Some(Expr::List(list)) = list {
          let mut list = list;
          let item = list.pop();
          if let Some(item) = item {
            self.stack.push(Expr::List(list));
            Ok(Some(item))
          } else {
            Ok(None)
          }
        } else {
          Err(EvalError {
            expr: Expr::Call(call.clone()),
            program: self.clone(),
            message: format!("Invalid args: [{:?}]", list),
          })
        }
      }
      "concat" => {
        let b = self.pop();
        let a = self.pop();
        if let (Some(Expr::List(a)), Some(Expr::List(b))) =
          (a.clone(), b.clone())
        {
          let mut a = a;
          a.extend(b);
          Ok(Some(Expr::List(a)))
        } else {
          Err(EvalError {
            expr: Expr::Call(call.clone()),
            program: self.clone(),
            message: format!("Invalid args: [{:?} {:?}]", a, b),
          })
        }
      }
      "ifelse" => {
        let condition = self.pop();
        let block = self.pop();
        let else_block = self.pop();
        if let (
          Some(Expr::List(condition)),
          Some(Expr::List(block)),
          Some(Expr::List(else_block)),
        ) = (condition.clone(), block.clone(), else_block.clone())
        {
          let result = self.eval(condition.clone());
          match result {
            Ok(_) => {
              let bool = self.pop();

              if let Some(bool) = bool {
                if bool.is_truthy() {
                  self.eval(block)?;
                } else {
                  self.eval(else_block)?;
                }

                Ok(None)
              } else {
                Err(EvalError {
                  expr: Expr::Call(call.clone()),
                  program: self.clone(),
                  message: format!(
                    "Invalid args: [{:?} {:?} {:?}]",
                    else_block, block, condition
                  ),
                })
              }
            }
            Err(err) => Err(err),
          }
        } else {
          Err(EvalError {
            expr: Expr::Call(call.clone()),
            program: self.clone(),
            message: format!(
              "Invalid args: [{:?} {:?} {:?}]",
              else_block, block, condition
            ),
          })
        }
      }
      "if" => {
        let condition = self.pop();
        let block = self.pop();
        if let (Some(Expr::List(condition)), Some(Expr::List(block))) =
          (condition.clone(), block.clone())
        {
          match self.eval(vec![
            Expr::List(vec![]),
            Expr::Lazy(Expr::List(block).into()),
            Expr::Lazy(Expr::List(condition).into()),
            Expr::Call("ifelse".to_string()),
          ]) {
            Ok(_) => Ok(None),
            Err(err) => Err(EvalError {
              expr: Expr::Call(call.clone()),
              program: err.clone().program,
              message: format!("Error in if condition: {}", err.message),
            }),
          }
        } else {
          Err(EvalError {
            expr: Expr::Call(call.clone()),
            program: self.clone(),
            message: format!("Invalid args: [{:?} {:?}]", block, condition),
          })
        }
      }
      "while" => {
        let condition = self.pop();
        let block = self.pop();

        if let (Some(Expr::List(condition)), Some(Expr::List(block))) =
          (condition, block)
        {
          loop {
            let mut block = block.clone();
            block.push(Expr::Boolean(true));

            match self.eval(vec![
              Expr::List(vec![Expr::Boolean(false)]),
              Expr::Lazy(Expr::List(block.clone()).into()),
              Expr::Lazy(Expr::List(condition.clone()).into()),
              Expr::Call("ifelse".to_string()),
            ]) {
              Ok(_) => {
                let bool = self.pop();

                if let Some(bool) = bool {
                  if !bool.is_truthy() {
                    break;
                  }
                } else {
                  return Err(EvalError {
                    expr: Expr::Call(call.clone()),
                    program: self.clone(),
                    message: format!(
                      "Invalid args: [{:?} {:?}]",
                      block, condition
                    ),
                  });
                }
              }
              Err(err) => {
                return Err(EvalError {
                  expr: Expr::Call(call.clone()),
                  program: err.clone().program,
                  message: format!("Error in while condition: {}", err.message),
                })
              }
            }
          }
        }

        Ok(None)
      }
      "print" => {
        let a = self.pop();

        if let Some(a) = a {
          match a {
            Expr::String(string) => println!("{}", string),
            _ => println!("{}", a),
          }

          Ok(None)
        } else {
          Err(EvalError {
            expr: Expr::Call(call.clone()),
            program: self.clone(),
            message: "Invalid args: []".to_string(),
          })
        }
      }
      "set" => {
        let name = self.pop();
        let value = self.pop();

        if let (Some(Expr::Call(name)), Some(value)) =
          (name.clone(), value.clone())
        {
          self.set_scope_item(&name, value);
          Ok(None)
        } else {
          Err(EvalError {
            expr: Expr::Call(call),
            program: self.clone(),
            message: format!("Invalid args: [{:?} {:?}]", value, name),
          })
        }
      }
      "get" => {
        let name = self.pop();

        if let Some(Expr::Call(name)) = name {
          Ok(self.scope_item(&name))
        } else {
          Err(EvalError {
            expr: Expr::Call(call),
            program: self.clone(),
            message: format!("Invalid args: [{:?}]", name),
          })
        }
      }
      "unset" => {
        let name = self.pop();
        if let Some(Expr::Call(name)) = name {
          self.remove_scope_item(&name);

          Ok(None)
        } else {
          Err(EvalError {
            expr: Expr::Call(call.clone()),
            program: self.clone(),
            message: format!("Invalid args: [{:?}]", name),
          })
        }
      }
      "halt" => Err(EvalError {
        expr: Expr::Call(call.clone()),
        program: self.clone(),
        message: "Halted.".to_string(),
      }),
      "collect" => Ok(Some(Expr::List(core::mem::take(&mut self.stack)))),
      "tostring" => {
        let a = self.stack.pop().unwrap_or_default();

        let string = match a {
          Expr::String(string) | Expr::Call(string) => string,
          _ => a.to_string(),
        };

        Ok(Some(Expr::String(string)))
      }
      "tocall" => {
        let a = self.stack.pop().unwrap_or_default();

        let string = match a {
          Expr::String(string) => string,
          _ => a.to_string(),
        };

        Ok(Some(Expr::Call(string)))
      }
      "tointeger" => {
        let a = self.stack.pop().unwrap_or_default();

        match a {
          Expr::String(string) => match string.parse() {
            Ok(integer) => Ok(Some(Expr::Integer(integer))),
            Err(err) => Err(EvalError {
              expr: Expr::Call(call.clone()),
              program: self.clone(),
              message: format!(
                "Error parsing [{}] as integer: {}",
                string, err
              ),
            }),
          },
          Expr::Boolean(boolean) => Ok(Some(Expr::Integer(boolean as i64))),
          Expr::Integer(integer) => Ok(Some(Expr::Integer(integer))),
          Expr::Float(float) => Ok(Some(Expr::Integer(float as i64))),

          a => Err(EvalError {
            expr: Expr::Call(call.clone()),
            program: self.clone(),
            message: format!("[{}] cannot be cast to integer", a),
          }),
        }
      }
      "tolist" => {
        let a = self.stack.pop().unwrap_or_default();

        match a {
          Expr::List(list) => Ok(Some(Expr::List(list))),
          _ => Ok(Some(Expr::List(vec![a]))),
        }
      }
      "typeof" => {
        let a = self.stack.pop().unwrap_or_default();
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
        let a = self.pop();

        if let Some(a) = a {
          self.stack.push(a.clone());
          self.stack.push(a);

          Ok(None)
        } else {
          Err(EvalError {
            expr: Expr::Call(call.clone()),
            program: self.clone(),
            message: "Not enough items on stack".to_string(),
          })
        }
      }
      "swap" => {
        let a = self.pop();
        let b = self.pop();

        if let (Some(a), Some(b)) = (a, b) {
          self.stack.push(a);
          self.stack.push(b);

          Ok(None)
        } else {
          Err(EvalError {
            expr: Expr::Call(call.clone()),
            program: self.clone(),
            message: "Not enough items on stack".to_string(),
          })
        }
      }
      "rot" => {
        if self.stack.len() < 3 {
          return Err(EvalError {
            expr: Expr::Call(call.clone()),
            program: self.clone(),
            message: "Not enough items on stack".to_string(),
          });
        }

        let len = self.stack.len();
        self.stack.swap(len - 1, len - 3);
        self.eval(vec![Expr::Call("swap".to_string())])?;

        Ok(None)
      }
      "call" => {
        let a = self.stack.pop();

        if let Some(Expr::Call(a)) = a {
          self.eval_expr(Expr::Call(a))
        } else if let Some(Expr::List(a)) = a {
          self.eval(a)?;
          Ok(None)
        } else {
          Err(EvalError {
            expr: Expr::Call(call.clone()),
            program: self.clone(),
            message: format!("Invalid args: [{:?}]", a),
          })
        }
      }
      "unwrap" => {
        let list = self.stack.pop();
        if let Some(Expr::List(list)) = list {
          for expr in list {
            self.stack.push(expr);
          }
          Ok(None)
        } else {
          Err(EvalError {
            expr: Expr::Call(call.clone()),
            program: self.clone(),
            message: format!("Invalid args: [{:?}]", list),
          })
        }
      }
      _ => {
        if let Some(value) = self.scope_item(&call) {
          if let Expr::List(_) = value {
            match self.eval(vec![
              Expr::Lazy(Expr::Call(call.clone()).into()),
              Expr::Call("get".to_string()),
              // Expr::Call("halt".to_string()),
              Expr::Call("call".to_string()),
            ]) {
              Ok(_) => Ok(None),
              Err(err) => Err(err),
            }
          } else {
            Ok(Some(value))
          }
        } else {
          Err(EvalError {
            expr: Expr::Call(call.clone()),
            program: self.clone(),
            message: format!("Unknown call: {}", call),
          })
        }
      }
    }
  }

  fn eval_expr(&mut self, expr: Expr) -> Result<Option<Expr>, EvalError> {
    match expr {
      Expr::Call(call) => self.eval_call(call),
      Expr::Lazy(block) => Ok(Some(*block)),
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
      Expr::ScopePush => {
        self.push_scope();
        Ok(None)
      }
      Expr::ScopePop => {
        self.pop_scope();
        Ok(None)
      }
      _ => Ok(Some(expr)),
    }
  }

  pub fn eval_string(&mut self, line: &str) -> Result<(), EvalError> {
    let tokens = crate::lex(line);
    let exprs = crate::parse(tokens.clone());

    self.eval(exprs)
  }

  pub fn eval(&mut self, exprs: Vec<Expr>) -> Result<(), EvalError> {
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
    fn add_two_numbers() {
      let mut program = Program::new();
      program.eval_string("1 2 +").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(3)]);
    }

    #[test]
    fn subtract_two_numbers() {
      let mut program = Program::new();
      program.eval_string("1 2 -").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(-1)]);
    }

    #[test]
    fn multiply_two_numbers() {
      let mut program = Program::new();
      program.eval_string("1 2 *").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(2)]);
    }

    #[test]
    fn divide_two_numbers() {
      let mut program = Program::new();
      program.eval_string("1 2 /").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(0)]);

      let mut program = Program::new();
      program.eval_string("1.0 2.0 /").unwrap();
      assert_eq!(program.stack, vec![Expr::Float(0.5)]);
    }

    #[test]
    fn modulo_two_numbers() {
      let mut program = Program::new();
      program.eval_string("10 5 %").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(0)]);

      let mut program = Program::new();
      program.eval_string("11 5 %").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(1)]);
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
      program.eval_string("'(1 2 +) unwrap call").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(3)]);
    }

    #[test]
    fn dont_eval_skips() {
      let mut program = Program::new();
      program.eval_string("6 'var set 'var").unwrap();
      assert_eq!(program.stack, vec![Expr::Call("var".to_string())]);
    }

    #[test]
    fn eval_lists() {
      let mut program = Program::new();
      program.eval_string("(1 2 3)").unwrap();
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
      program.eval_string("6 'var set (var)").unwrap();
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

    mod bitwise {
      use super::*;

      #[test]
      fn and_int() {
        let mut program = Program::new();
        program.eval_string("1 1 and").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);

        let mut program = Program::new();
        program.eval_string("1 0 and").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new();
        program.eval_string("0 1 and").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new();
        program.eval_string("0 0 and").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);
      }

      #[test]
      fn and_bool() {
        let mut program = Program::new();
        program.eval_string("true true and").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);

        let mut program = Program::new();
        program.eval_string("true false and").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new();
        program.eval_string("false true and").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);

        let mut program = Program::new();
        program.eval_string("false false and").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);
      }

      #[test]
      fn or_int() {
        let mut program = Program::new();
        program.eval_string("1 1 or").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);

        let mut program = Program::new();
        program.eval_string("1 0 or").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);

        let mut program = Program::new();
        program.eval_string("0 1 or").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);

        let mut program = Program::new();
        program.eval_string("0 0 or").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(false)]);
      }

      #[test]
      fn or_bool() {
        let mut program = Program::new();
        program.eval_string("true true or").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);

        let mut program = Program::new();
        program.eval_string("true false or").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);

        let mut program = Program::new();
        program.eval_string("false true or").unwrap();
        assert_eq!(program.stack, vec![Expr::Boolean(true)]);

        let mut program = Program::new();
        program.eval_string("false false or").unwrap();
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
        vec![HashMap::from_iter(vec![(
          "a".to_string(),
          Expr::Integer(1)
        )])]
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
      assert_eq!(program.scope, vec![HashMap::new()]);
    }

    mod scope {
      use super::*;

      #[test]
      fn scope_pop() {
        let mut program = Program::new();
        program.eval_string("{1 'a set}").unwrap();
        assert_eq!(program.scope, vec![HashMap::new()]);
      }

      #[test]
      fn scope_open() {
        let mut program = Program::new();
        program.eval_string("{1 'a set").unwrap();
        assert_eq!(
          program.scope,
          vec![
            // Main Scope
            HashMap::new(),
            // Block Scope
            HashMap::from_iter(vec![("a".to_string(), Expr::Integer(1))])
          ]
        );
      }

      #[test]
      fn no_overwriting_outside() {
        let mut program = Program::new();
        program.eval_string("1 'a set {2 'a set").unwrap();
        assert_eq!(
          program.scope,
          vec![
            HashMap::from_iter(vec![("a".to_string(), Expr::Integer(1))]),
            HashMap::from_iter(vec![("a".to_string(), Expr::Integer(2))])
          ]
        );
      }

      #[test]
      fn overwriting_inside() {
        let mut program = Program::new();
        program.eval_string("{1 'a set 2 'a set").unwrap();
        assert_eq!(
          program.scope,
          vec![
            HashMap::new(),
            HashMap::from_iter(vec![("a".to_string(), Expr::Integer(2))])
          ]
        );
      }

      #[test]
      fn scoped_variables() {
        let mut program = Program::new();
        program.eval_string("1 '@a set @a").unwrap();
        assert_eq!(program.stack, vec![Expr::Integer(1)]);

        let mut program = Program::new();
        program.eval_string("{1 '@a set @a}").unwrap();
        assert_eq!(program.stack, vec![Expr::Integer(1)]);
      }
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
    fn duplicating() {
      let mut program = Program::new();
      program.eval_string("1 dup").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(1), Expr::Integer(1)]);
    }

    #[test]
    fn swapping() {
      let mut program = Program::new();
      program.eval_string("1 2 swap").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(2), Expr::Integer(1)]);
    }

    #[test]
    fn rotating() {
      let mut program = Program::new();
      program.eval_string("1 2 3 rot").unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::Integer(3), Expr::Integer(1), Expr::Integer(2)]
      );
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
        .eval_string("1 2 3 collect 'a set 'a get unwrap")
        .unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::Integer(1), Expr::Integer(2), Expr::Integer(3)]
      );
      assert_eq!(
        program.scope,
        vec![HashMap::from_iter(vec![(
          "a".to_string(),
          Expr::List(vec![
            Expr::Integer(1),
            Expr::Integer(2),
            Expr::Integer(3)
          ])
        )])]
      );
    }
  }

  mod list_ops {
    use super::*;

    #[test]
    fn inserting_into_list() {
      let mut program = Program::new();
      program.eval_string("(1 2) 3 insert").unwrap();
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
      program.eval_string("(1 2) last").unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::List(vec![Expr::Integer(1)]), Expr::Integer(2)]
      );
    }

    #[test]
    fn concatenating_lists() {
      let mut program = Program::new();
      program.eval_string("(1 2) (3 \"4\") concat").unwrap();
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
    fn concatenating_blocks() {
      let mut program = Program::new();
      program.eval_string("(1 2) ('+) concat").unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::List(vec![
          Expr::Integer(1),
          Expr::Integer(2),
          Expr::Call("+".to_owned())
        ])]
      );
    }

    #[test]
    fn getting_length_of_list() {
      let mut program = Program::new();
      program.eval_string("(1 2 3) len").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(3)]);
    }

    #[test]
    fn getting_nth_item_of_list() {
      let mut program = Program::new();
      program.eval_string("(1 2 3) 1 nth").unwrap();
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
        .eval_string("(\"a\" 3 \"hello\" 1.2) \"\" join")
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
        .eval_string("1 2 + '(\"correct\") '(3 =) if")
        .unwrap();
      assert_eq!(program.stack, vec![Expr::String("correct".to_owned())]);
    }

    #[test]
    fn if_empty_condition() {
      let mut program = Program::new();
      program
        .eval_string("1 2 + 3 = '(\"correct\") '() if")
        .unwrap();
      assert_eq!(program.stack, vec![Expr::String("correct".to_owned())]);
    }

    #[test]
    fn if_else_true() {
      let mut program = Program::new();
      program
        .eval_string("1 2 + 3 = '(\"incorrect\") '(\"correct\") '() ifelse")
        .unwrap();
      assert_eq!(program.stack, vec![Expr::String("correct".to_owned())]);
    }

    #[test]
    fn if_else_false() {
      let mut program = Program::new();
      program
        .eval_string("1 2 + 2 = '(\"incorrect\") '(\"correct\") '() ifelse")
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

           '(
             ;; Decrement i by 1
             i 1 -
             ;; Set i
             'i set

             i
           ) '(
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
    fn to_call() {
      let mut program = Program::new();
      program.eval_string("\"a\" tocall").unwrap();
      assert_eq!(program.stack, vec![Expr::Call("a".to_string())]);
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

    #[test]
    fn list_to_list() {
      let mut program = Program::new();
      program.eval_string("(1 2 3) tolist").unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::List(vec![
          Expr::Integer(1),
          Expr::Integer(2),
          Expr::Integer(3)
        ])]
      );
    }
  }
}
