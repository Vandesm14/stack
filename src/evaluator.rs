use itertools::Itertools as _;
// use itertools::Itertools;
use lasso::Spur;
use syscalls::Sysno;

use crate::{interner::interner, Expr, Intrinsic, Lexer, Parser, Type};
use core::{fmt, iter};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default)]
pub struct Program {
  pub stack: Vec<Expr>,
  pub scopes: Vec<HashMap<String, Expr>>,
  pub loaded_files: HashSet<String>,
}

impl fmt::Display for Program {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Stack: [")?;

    self.stack.iter().enumerate().try_for_each(|(i, expr)| {
      if i == self.stack.len() - 1 {
        write!(f, "{expr}")
      } else {
        write!(f, "{expr}, ")
      }
    })?;
    write!(f, "]")?;

    writeln!(f,)?;

    if !self.scopes.is_empty() {
      writeln!(f, "Scope:")?;

      let layers = self.scopes.len();
      for (layer_i, layer) in self.scopes.iter().enumerate() {
        let items = layer.len();
        writeln!(f, "Layer {}:", layer_i)?;
        for (item_i, (key, value)) in
          layer.iter().sorted_by_key(|(s, _)| *s).enumerate()
        {
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
  #[inline]
  pub fn new() -> Self {
    Self {
      stack: vec![],
      scopes: vec![HashMap::new()],
      loaded_files: HashSet::new(),
    }
  }

  pub fn with_core(mut self) -> Result<Self, EvalError> {
    let core_lib = include_str!("./core.stack");
    self.eval_string(core_lib)?;

    Ok(self)
  }

  fn pop(&mut self, trace_expr: &Expr) -> Result<Expr, EvalError> {
    self.stack.pop().ok_or_else(|| EvalError {
      expr: trace_expr.clone(),
      program: self.clone(),
      message: "Stack underflow".into(),
    })
  }

  fn push(&mut self, expr: Expr) {
    self.stack.push(expr);
  }

  fn scope_item(&self, symbol: &str) -> Option<Expr> {
    let current = self.scopes.last();
    if let Some(item) = current.and_then(|layer| layer.get(symbol)) {
      return Some(item.clone());
    }

    let global = self.scopes.get(0);
    if let Some(item) = global.and_then(|layer| layer.get(symbol)) {
      return Some(item.clone());
    }

    None
  }

  fn set_scope_item(
    &mut self,
    trace_expr: &Expr,
    symbol: &str,
    value: Expr,
  ) -> Result<(), EvalError> {
    if let Some(layer) = self.scopes.last_mut() {
      layer.insert(symbol.to_string(), value);
      Ok(())
    } else {
      Err(EvalError {
        expr: trace_expr.clone(),
        program: self.clone(),
        message: format!(
          "no scope to set {symbol}, there may be too many \"}}\""
        ),
      })
    }
  }

  fn remove_scope_item(&mut self, symbol: &str) {
    if let Some(layer) = self.scopes.last_mut() {
      layer.remove(symbol);
    }
  }

  fn push_scope(&mut self) {
    self.scopes.push(HashMap::new());
  }

  fn pop_scope(&mut self) {
    self.scopes.pop();
  }

  fn eval_intrinsic(
    &mut self,
    trace_expr: &Expr,
    intrinsic: Intrinsic,
  ) -> Result<(), EvalError> {
    match intrinsic {
      // Arithmetic
      Intrinsic::Add => {
        let rhs = self.pop(trace_expr)?;
        let lhs = self.pop(trace_expr)?;

        match lhs.coerce_same_float(&rhs) {
          Some((Expr::Integer(lhs), Expr::Integer(rhs))) => {
            self.push(Expr::Integer(lhs + rhs));
            Ok(())
          }
          Some((Expr::Float(lhs), Expr::Float(rhs))) => {
            self.push(Expr::Float(lhs + rhs));
            Ok(())
          }
          Some((Expr::Pointer(lhs), Expr::Pointer(rhs))) => {
            self.push(Expr::Pointer(lhs + rhs));
            Ok(())
          }
          _ => Err(EvalError {
            expr: trace_expr.clone(),
            program: self.clone(),
            message: format!(
              "expected {}, found {}",
              Type::List(vec![
                Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
                Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
              ]),
              Type::List(vec![lhs.type_of(), rhs.type_of()]),
            ),
          }),
        }
      }
      Intrinsic::Subtract => {
        let rhs = self.pop(trace_expr)?;
        let lhs = self.pop(trace_expr)?;

        match lhs.coerce_same_float(&rhs) {
          Some((Expr::Integer(lhs), Expr::Integer(rhs))) => {
            self.push(Expr::Integer(lhs - rhs));
            Ok(())
          }
          Some((Expr::Float(lhs), Expr::Float(rhs))) => {
            self.push(Expr::Float(lhs - rhs));
            Ok(())
          }
          Some((Expr::Pointer(lhs), Expr::Pointer(rhs))) => {
            self.push(Expr::Pointer(lhs - rhs));
            Ok(())
          }
          _ => Err(EvalError {
            expr: trace_expr.clone(),
            program: self.clone(),
            message: format!(
              "expected {}, found {}",
              Type::List(vec![
                Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
                Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
              ]),
              Type::List(vec![lhs.type_of(), rhs.type_of()]),
            ),
          }),
        }
      }
      Intrinsic::Multiply => {
        let rhs = self.pop(trace_expr)?;
        let lhs = self.pop(trace_expr)?;

        match lhs.coerce_same_float(&rhs) {
          Some((Expr::Integer(lhs), Expr::Integer(rhs))) => {
            self.push(Expr::Integer(lhs * rhs));
            Ok(())
          }
          Some((Expr::Float(lhs), Expr::Float(rhs))) => {
            self.push(Expr::Float(lhs * rhs));
            Ok(())
          }
          Some((Expr::Pointer(lhs), Expr::Pointer(rhs))) => {
            self.push(Expr::Pointer(lhs * rhs));
            Ok(())
          }
          _ => Err(EvalError {
            expr: trace_expr.clone(),
            program: self.clone(),
            message: format!(
              "expected {}, found {}",
              Type::List(vec![
                Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
                Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
              ]),
              Type::List(vec![lhs.type_of(), rhs.type_of()]),
            ),
          }),
        }
      }
      Intrinsic::Divide => {
        let rhs = self.pop(trace_expr)?;
        let lhs = self.pop(trace_expr)?;

        match lhs.coerce_same_float(&rhs) {
          Some((Expr::Integer(lhs), Expr::Integer(rhs))) => {
            self.push(Expr::Integer(lhs / rhs));
            Ok(())
          }
          Some((Expr::Float(lhs), Expr::Float(rhs))) => {
            self.push(Expr::Float(lhs / rhs));
            Ok(())
          }
          Some((Expr::Pointer(lhs), Expr::Pointer(rhs))) => {
            self.push(Expr::Pointer(lhs / rhs));
            Ok(())
          }
          _ => Err(EvalError {
            expr: trace_expr.clone(),
            program: self.clone(),
            message: format!(
              "expected {}, found {}",
              Type::List(vec![
                Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
                Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
              ]),
              Type::List(vec![lhs.type_of(), rhs.type_of()]),
            ),
          }),
        }
      }
      Intrinsic::Remainder => {
        let rhs = self.pop(trace_expr)?;
        let lhs = self.pop(trace_expr)?;

        match lhs.coerce_same_float(&rhs) {
          Some((Expr::Integer(lhs), Expr::Integer(rhs))) => {
            self.push(Expr::Integer(lhs % rhs));
            Ok(())
          }
          Some((Expr::Float(lhs), Expr::Float(rhs))) => {
            self.push(Expr::Float(lhs % rhs));
            Ok(())
          }
          Some((Expr::Pointer(lhs), Expr::Pointer(rhs))) => {
            self.push(Expr::Pointer(lhs % rhs));
            Ok(())
          }
          _ => Err(EvalError {
            expr: trace_expr.clone(),
            program: self.clone(),
            message: format!(
              "expected {}, found {}",
              Type::List(vec![
                Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
                Type::Set(vec![Type::Integer, Type::Float, Type::Pointer]),
              ]),
              Type::List(vec![lhs.type_of(), rhs.type_of()]),
            ),
          }),
        }
      }

      // Comparison
      Intrinsic::Equal => {
        let rhs = self.pop(trace_expr)?;
        let lhs = self.pop(trace_expr)?;

        self.push(Expr::Boolean(lhs == rhs));

        Ok(())
      }
      Intrinsic::NotEqual => {
        let rhs = self.pop(trace_expr)?;
        let lhs = self.pop(trace_expr)?;

        self.push(Expr::Boolean(lhs != rhs));

        Ok(())
      }
      Intrinsic::GreaterThan => {
        let rhs = self.pop(trace_expr)?;
        let lhs = self.pop(trace_expr)?;

        self.push(Expr::Boolean(lhs > rhs));

        Ok(())
      }
      Intrinsic::LessThan => {
        let rhs = self.pop(trace_expr)?;
        let lhs = self.pop(trace_expr)?;

        self.push(Expr::Boolean(lhs < rhs));

        Ok(())
      }
      Intrinsic::Or => {
        let rhs = self.pop(trace_expr)?;
        let lhs = self.pop(trace_expr)?;

        self.push(Expr::Boolean(lhs.is_truthy() || rhs.is_truthy()));

        Ok(())
      }
      Intrinsic::And => {
        let rhs = self.pop(trace_expr)?;
        let lhs = self.pop(trace_expr)?;

        self.push(Expr::Boolean(lhs.is_truthy() && rhs.is_truthy()));

        Ok(())
      }

      // Code/IO
      Intrinsic::Parse => {
        let item = self.pop(trace_expr)?;

        match item {
          Expr::String(string) => {
            let source = interner().resolve(&string).to_string();

            let lexer = Lexer::new(&source);
            let parser = Parser::new(lexer);
            let expr = parser.parse().ok().map(Expr::List).unwrap_or(Expr::Nil);

            self.push(expr);

            Ok(())
          }
          _ => Err(EvalError {
            expr: trace_expr.clone(),
            program: self.clone(),
            message: format!(
              "expected {}, found {}",
              Type::String,
              item.type_of(),
            ),
          }),
        }
      }
      // TODO: Re-implement using syscalls.
      Intrinsic::ReadFile => {
        let item = self.pop(trace_expr)?;

        match item {
          Expr::String(path) => {
            let path_str = interner().resolve(&path);

            match std::fs::read_to_string(path_str) {
              Ok(contents) => {
                self.loaded_files.insert(path_str.to_string());

                let content = interner().get_or_intern(contents);
                self.push(Expr::String(content));

                Ok(())
              }
              Err(e) => Err(EvalError {
                expr: trace_expr.clone(),
                program: self.clone(),
                message: format!("unable to read {path_str}: {e}"),
              }),
            }
          }
          _ => Err(EvalError {
            expr: trace_expr.clone(),
            program: self.clone(),
            message: format!(
              "expected {}, found {}",
              Type::String,
              item.type_of(),
            ),
          }),
        }
      }
      Intrinsic::Syscall { arity } => {
        let sysno = self.pop(trace_expr).and_then(|sysno| match sysno {
          Expr::Integer(sysno) => (sysno >= 0)
            .then(|| sysno as usize)
            .and_then(|sysno| Sysno::new(sysno))
            .ok_or_else(|| EvalError {
              expr: trace_expr.clone(),
              program: self.clone(),
              message: format!("invalid syscall: {sysno}"),
            }),
          found => Err(EvalError {
            expr: trace_expr.clone(),
            program: self.clone(),
            message: format!(
              "expected {}, found {}",
              Type::Integer,
              found.type_of()
            ),
          }),
        })?;

        let args = (0..arity).try_fold(
          Vec::with_capacity(arity as usize),
          |mut args, _| match self.pop(trace_expr)? {
            Expr::Pointer(x) => {
              args.push(x);
              Ok(args)
            }
            Expr::Integer(x) => {
              if x >= 0 {
                args.push(x as usize);
                Ok(args)
              } else {
                Err(EvalError {
                  expr: trace_expr.clone(),
                  program: self.clone(),
                  message: format!("integer must be positive"),
                })
              }
            }
            arg => Err(EvalError {
              expr: trace_expr.clone(),
              program: self.clone(),
              message: format!(
                "expected {}, found {}",
                Type::Integer,
                arg.type_of()
              ),
            }),
          },
        )?;

        let result = match arity {
          0 => unsafe { syscalls::raw::syscall0(sysno) },
          1 => unsafe { syscalls::raw::syscall1(sysno, args[0]) },
          2 => unsafe { syscalls::raw::syscall2(sysno, args[0], args[1]) },
          3 => unsafe {
            syscalls::raw::syscall3(sysno, args[0], args[1], args[2])
          },
          4 => unsafe {
            syscalls::raw::syscall4(sysno, args[0], args[1], args[2], args[3])
          },
          5 => unsafe {
            syscalls::raw::syscall5(
              sysno, args[0], args[1], args[2], args[3], args[4],
            )
          },
          6 => unsafe {
            syscalls::raw::syscall6(
              sysno, args[0], args[1], args[2], args[3], args[4], args[5],
            )
          },
          _ => unimplemented!("invalid syscall arity: {arity}"),
        };

        self.push(Expr::Integer(result as i64));

        Ok(())
      }
      Intrinsic::Panic => {
        let string = self.pop(trace_expr)?;

        Err(EvalError {
          expr: trace_expr.clone(),
          program: self.clone(),
          message: format!(
            "panic: {}",
            string.display(&self.interner.as_ref())
          ),
        })
      }
      Intrinsic::Debug => {
        let item = self.pop(trace_expr)?;

        println!("{}", item.display(&self.interner.as_ref()));

        Ok(())
      }

      // List
      // TODO: Deprecate in favor of `"hello" tolist`
      Intrinsic::Explode => {
        let item = self.pop(trace_expr)?;

        match item {
          Expr::String(string) => {
            let string_str = interner().resolve(&string).to_owned();

            let list = Expr::List(
              string_str
                .chars()
                .map(|c| Expr::String(interner().get_or_intern(c.to_string())))
                .collect::<Vec<_>>(),
            );
            self.push(list);

            Ok(())
          }
          _ => Err(EvalError {
            expr: trace_expr.clone(),
            program: self.clone(),
            message: format!(
              "expected {}, found {}",
              Type::String,
              item.type_of(),
            ),
          }),
        }
      }
      Intrinsic::Length => {
        let item = self.pop(trace_expr)?;

        match item {
          Expr::List(list) => {
            // TODO: Check that the length fits in an i64.
            self.push(Expr::Integer(list.len() as i64));
            Ok(())
          }
          _ => Err(EvalError {
            expr: trace_expr.clone(),
            program: self.clone(),
            message: format!(
              "expected {}, found {}",
              Type::List(vec![]),
              item.type_of(),
            ),
          }),
        }
      }
      Intrinsic::Nth => {
        let index = self.pop(trace_expr)?;
        let indexable = self.pop(trace_expr)?;

        match (index, indexable) {
          (Expr::Integer(index), Expr::List(list)) => {
            let item = if index >= 0 && index < list.len() as i64 {
              list.get(index as usize).cloned()
            } else if index < 0 && -index <= list.len() as i64 {
              list.get(list.len() - -index as usize).cloned()
            } else {
              None
            };

            match item {
              Some(item) => {
                self.push(item);
                Ok(())
              }
              None => Err(EvalError {
                expr: trace_expr.clone(),
                program: self.clone(),
                message: format!("index {index} is out of bounds"),
              }),
            }
          }
          (index, indexable) => Err(EvalError {
            expr: trace_expr.clone(),
            program: self.clone(),
            message: format!(
              "expected {}, found {}",
              Type::List(vec![Type::List(vec![]), Type::Integer]),
              Type::List(vec![indexable.type_of(), index.type_of()]),
            ),
          }),
        }
      }
      Intrinsic::Join => {
        let delimiter = self.pop(trace_expr)?;
        let list = self.pop(trace_expr)?;

        match (delimiter, list) {
          (Expr::String(delimiter), Expr::List(list)) => {
            let delimiter_str = interner().resolve(&delimiter);

            let string = list
              .into_iter()
              .map(|expr| match expr {
                Expr::String(string) => interner().resolve(&string).to_string(),
                _ => expr.to_string(),
              })
              .join(delimiter_str);
            let string = Expr::String(interner().get_or_intern(string));
            self.push(string);

            Ok(())
          }
          (delimiter, list) => Err(EvalError {
            expr: trace_expr.clone(),
            program: self.clone(),
            message: format!(
              "expected {}, found {}",
              Type::List(vec![Type::List(vec![]), Type::String]),
              Type::List(vec![list.type_of(), delimiter.type_of()]),
            ),
          }),
        }
      }
      // Pushes the last value in the stack into the list
      Intrinsic::Insert => {
        let item = self.pop(trace_expr)?;
        let list = self.pop(trace_expr)?;

        match (item, list) {
          (item, Expr::List(mut list)) => {
            list.push(item);
            self.push(Expr::List(list));

            Ok(())
          }
          (item, list) => Err(EvalError {
            expr: trace_expr.clone(),
            program: self.clone(),
            message: format!(
              "expected {}, found {}",
              Type::List(vec![Type::List(vec![]), Type::Any,]),
              Type::List(vec![list.type_of(), item.type_of()]),
            ),
          }),
        }
      }
      // Pops the last value of a list onto the stack
      Intrinsic::ListPop => {
        let item = self.pop(trace_expr)?;

        match item {
          Expr::List(mut list) => {
            let item = list.pop().unwrap_or(Expr::Nil);

            self.push(Expr::List(list));
            self.push(item);

            Ok(())
          }
          item => Err(EvalError {
            expr: trace_expr.clone(),
            program: self.clone(),
            message: format!(
              "expected {}, found {}",
              Type::List(vec![]),
              item.type_of(),
            ),
          }),
        }
      }
      Intrinsic::ListShift => {
        let item = self.pop(trace_expr)?;

        match item {
          Expr::List(mut list) => {
            let item = (!list.is_empty())
              .then(|| list.remove(0))
              .unwrap_or(Expr::Nil);

            self.push(Expr::List(list));
            self.push(item);

            Ok(())
          }
          item => Err(EvalError {
            expr: trace_expr.clone(),
            program: self.clone(),
            message: format!(
              "expected {}, found {}",
              Type::List(vec![]),
              item.type_of(),
            ),
          }),
        }
      }
      Intrinsic::Concat => {
        let rhs = self.pop(trace_expr)?;
        let lhs = self.pop(trace_expr)?;

        match (lhs, rhs) {
          (Expr::List(mut lhs), Expr::List(rhs)) => {
            lhs.extend(rhs);
            self.push(Expr::List(lhs));

            Ok(())
          }
          (lhs, rhs) => Err(EvalError {
            expr: trace_expr.clone(),
            program: self.clone(),
            message: format!(
              "expected {}, found {}",
              Type::List(vec![Type::List(vec![]), Type::List(vec![])]),
              Type::List(vec![lhs.type_of(), rhs.type_of()]),
            ),
          }),
        }
      }
      Intrinsic::Unwrap => {
        let item = self.pop(trace_expr)?;

        match item {
          Expr::List(list) => {
            self.stack.extend(list);
            Ok(())
          }
          item => Err(EvalError {
            expr: trace_expr.clone(),
            program: self.clone(),
            message: format!(
              "expected {}, found {}",
              Type::List(vec![]),
              item.type_of(),
            ),
          }),
        }
      }
      Intrinsic::Reverse => {
        let item = self.pop(trace_expr)?;

        match item {
          Expr::List(mut list) => {
            list.reverse();
            self.push(Expr::List(list));

            Ok(())
          }
          item => Err(EvalError {
            expr: trace_expr.clone(),
            program: self.clone(),
            message: format!(
              "expected {}, found {}",
              Type::List(vec![]),
              item.type_of(),
            ),
          }),
        }
      }

      // Control Flow
      Intrinsic::IfElse => {
        let cond = self.pop(trace_expr)?;
        let then = self.pop(trace_expr)?;
        let r#else = self.pop(trace_expr)?;

        match (cond, then, r#else) {
          (Expr::List(cond), Expr::List(then), Expr::List(r#else)) => {
            self.eval(cond)?;
            let cond = self.pop(trace_expr)?;

            if cond.is_truthy() {
              self.eval(then)
            } else {
              self.eval(r#else)
            }
          }
          (cond, then, r#else) => Err(EvalError {
            expr: trace_expr.clone(),
            program: self.clone(),
            message: format!(
              "expected {}, found {}",
              Type::List(vec![
                // TODO: A type to represent functions.
                Type::List(vec![Type::Boolean]),
                Type::List(vec![]),
                Type::List(vec![]),
              ]),
              Type::List(vec![
                cond.type_of(),
                then.type_of(),
                r#else.type_of(),
              ]),
            ),
          }),
        }
      }
      Intrinsic::If => {
        let cond = self.pop(trace_expr)?;
        let then = self.pop(trace_expr)?;

        match (cond, then) {
          (Expr::List(cond), Expr::List(then)) => {
            self.eval(cond)?;
            let cond = self.pop(trace_expr)?;

            if cond.is_truthy() {
              self.eval(then)
            } else {
              Ok(())
            }
          }
          (cond, then) => Err(EvalError {
            expr: trace_expr.clone(),
            program: self.clone(),
            message: format!(
              "expected {}, found {}",
              Type::List(vec![
                // TODO: A type to represent functions.
                Type::List(vec![Type::Boolean]),
                Type::List(vec![]),
              ]),
              Type::List(vec![cond.type_of(), then.type_of(),]),
            ),
          }),
        }

        // self.push(Expr::List(vec![]));
        // self.eval_intrinsic(trace_expr, Intrinsic::IfElse)
      }
      Intrinsic::While => {
        let cond = self.pop(trace_expr)?;
        let block = self.pop(trace_expr)?;

        match (cond, block) {
          (Expr::List(cond), Expr::List(block)) => loop {
            self.eval(cond.clone())?;
            let cond = self.pop(trace_expr)?;

            if cond.is_truthy() {
              self.eval(block.clone())?;
            } else {
              break Ok(());
            }
          },
          (cond, block) => Err(EvalError {
            expr: trace_expr.clone(),
            program: self.clone(),
            message: format!(
              "expected {}, found {}",
              Type::List(vec![
                // TODO: A type to represent functions.
                Type::List(vec![Type::Boolean]),
                Type::List(vec![]),
              ]),
              Type::List(vec![cond.type_of(), block.type_of(),]),
            ),
          }),
        }
      }
      Intrinsic::Halt => Err(EvalError {
        expr: trace_expr.clone(),
        program: self.clone(),
        message: "halt".to_string(),
      }),

      // Scope
      Intrinsic::Set => {
        let key = self.pop(trace_expr)?;
        let val = self.pop(trace_expr)?;

        match key {
          Expr::Call(key) => {
            let key_str = interner().resolve(&key);

            match Intrinsic::try_from(key_str) {
              Ok(intrinsic) => Err(EvalError {
                expr: trace_expr.clone(),
                program: self.clone(),
                message: format!(
                  "cannot shadow an intrinsic {}",
                  intrinsic.as_str()
                ),
              }),
              Err(_) => {
                self.set_scope_item(trace_expr, &key_str.to_owned(), val)
              }
            }
          }
          key => Err(EvalError {
            expr: trace_expr.clone(),
            program: self.clone(),
            message: format!(
              "expected {}, found {}",
              Type::List(vec![
                // TODO: A type to represent functions.
                Type::Any,
                Type::Call,
              ]),
              Type::List(vec![val.type_of(), key.type_of(),]),
            ),
          }),
        }
      }
      Intrinsic::Get => {
        let item = self.pop(trace_expr)?;

        match item {
          Expr::Call(key) => {
            let key_str = interner().resolve(&key);
            // Always push something, otherwise it can get tricky to manage the
            // stack in-langauge.
            self.push(self.scope_item(key_str).unwrap_or(Expr::Nil));

            Ok(())
          }
          item => Err(EvalError {
            expr: trace_expr.clone(),
            program: self.clone(),
            message: format!(
              "expected {}, found {}",
              Type::Call,
              item.type_of(),
            ),
          }),
        }
      }
      Intrinsic::Unset => {
        let item = self.pop(trace_expr)?;

        match item {
          Expr::Call(key) => {
            let key_str = interner().resolve(&key).to_owned();
            self.remove_scope_item(&key_str);

            Ok(())
          }
          item => Err(EvalError {
            expr: trace_expr.clone(),
            program: self.clone(),
            message: format!(
              "expected {}, found {}",
              Type::Call,
              item.type_of(),
            ),
          }),
        }
      }

      // Stack
      Intrinsic::Collect => {
        let list = core::mem::take(&mut self.stack);
        self.push(Expr::List(list));

        Ok(())
      }
      Intrinsic::Clear => {
        self.stack.clear();
        Ok(())
      }
      Intrinsic::Pop => {
        self.stack.pop();
        Ok(())
      }
      Intrinsic::Dup => {
        let item = self.pop(trace_expr)?;

        self.push(item.clone());
        self.push(item);

        Ok(())
      }
      Intrinsic::Swap => {
        let rhs = self.pop(trace_expr)?;
        let lhs = self.pop(trace_expr)?;

        self.push(rhs);
        self.push(lhs);

        Ok(())
      }
      Intrinsic::Rot => {
        let rhs = self.pop(trace_expr)?;
        let mid = self.pop(trace_expr)?;
        let lhs = self.pop(trace_expr)?;

        self.push(rhs);
        self.push(lhs);
        self.push(mid);

        Ok(())
      }

      // Functions/Data
      Intrinsic::Call => {
        let item = self.pop(trace_expr)?;

        match item {
          call @ Expr::Call(_) => self.eval_expr(call),
          item @ Expr::List(_) => match item.create_fn_scope() {
            Some(_) => {
              let Expr::List(list) = item else {
                unreachable!()
              };

              self.push_scope();

              match self.eval(list) {
                Ok(_) => {
                  self.pop_scope();
                  Ok(())
                }
                Err(err) => Err(err),
              }
            }
            None => {
              let Expr::List(list) = item else {
                unreachable!()
              };
              self.eval(list)
            }
          },
          _ => Err(EvalError {
            expr: trace_expr.clone(),
            program: self.clone(),
            message: format!(
              "expected {}, found {}",
              Type::Set(vec![
                Type::Call,
                Type::List(vec![Type::FnScope, Type::Any])
              ]),
              item.type_of(),
            ),
          }),
        }
      }
      Intrinsic::CallNative => {
        let item = self.pop(trace_expr)?;

        match item {
          Expr::String(name) => {
            let name_str = interner().resolve(&name);

            match Intrinsic::try_from(name_str) {
              Ok(intrinsic) => self.eval_intrinsic(trace_expr, intrinsic),
              Err(_) => Err(EvalError {
                expr: trace_expr.clone(),
                program: self.clone(),
                message: format!("invalid intrinsic {name_str}"),
              }),
            }
          }
          _ => Err(EvalError {
            expr: trace_expr.clone(),
            program: self.clone(),
            message: format!(
              "expected {}, found {}",
              Type::String,
              item.type_of(),
            ),
          }),
        }
      }
      Intrinsic::Lazy => {
        let item = self.pop(trace_expr)?;
        self.push(Expr::Lazy(Box::new(item)));

        Ok(())
      }
      Intrinsic::Noop => Ok(()),

      // Type
      Intrinsic::ToBoolean => {
        let item = self.pop(trace_expr)?;

        match item {
          Expr::String(string) => {
            let string_str = interner().resolve(&string);

            self.push(
              string_str
                .parse()
                .ok()
                .map(Expr::Boolean)
                .unwrap_or(Expr::Nil),
            );
          }
          found => self.push(found.to_boolean().unwrap_or(Expr::Nil)),
        }

        Ok(())
      }
      Intrinsic::ToInteger => {
        let item = self.pop(trace_expr)?;

        match item {
          Expr::String(string) => {
            let string_str = interner().resolve(&string);

            self.push(
              string_str
                .parse()
                .ok()
                .map(Expr::Integer)
                .unwrap_or(Expr::Nil),
            );
          }
          found => self.push(found.to_integer().unwrap_or(Expr::Nil)),
        }

        Ok(())
      }
      Intrinsic::ToFloat => {
        let item = self.pop(trace_expr)?;

        match item {
          Expr::String(string) => {
            let string_str = interner().resolve(&string);

            self.push(
              string_str
                .parse()
                .ok()
                .map(Expr::Float)
                .unwrap_or(Expr::Nil),
            );
          }
          found => self.push(found.to_float().unwrap_or(Expr::Nil)),
        }

        Ok(())
      }
      Intrinsic::ToPointer => {
        let item = self.pop(trace_expr)?;

        match item {
          Expr::String(string) => {
            let string_str = interner().resolve(&string);
            self.push(Expr::Pointer(string_str.as_ptr() as usize));
          }
          found => self.push(found.to_pointer().unwrap_or(Expr::Nil)),
        }

        Ok(())
      }
      Intrinsic::ToList => {
        let item = self.pop(trace_expr)?;

        match item {
          list @ Expr::List(_) => {
            self.push(list);
            Ok(())
          }
          found => {
            self.push(Expr::List(vec![found]));
            Ok(())
          }
        }
      }
      Intrinsic::ToString => {
        let item = self.pop(trace_expr)?;

        match item {
          string @ Expr::String(_) => {
            self.push(string);
            Ok(())
          }
          found => {
            let string =
              Expr::String(interner().get_or_intern(found.to_string()));
            self.push(string);

            Ok(())
          }
        }
      }
      Intrinsic::ToCall => {
        let item = self.pop(trace_expr)?;

        match item {
          call @ Expr::Call(_) => {
            self.push(call);
            Ok(())
          }
          Expr::String(string) => {
            self.push(Expr::Call(string));
            Ok(())
          }
          found => {
            let call = Expr::Call(interner().get_or_intern(found.to_string()));
            self.push(call);

            Ok(())
          }
        }
      }
      Intrinsic::TypeOf => {
        let item = self.pop(trace_expr)?;
        let string =
          Expr::String(interner().get_or_intern(item.type_of().to_string()));
        self.push(string);

        Ok(())
      }
    }
  }

  fn eval_call(
    &mut self,
    trace_expr: &Expr,
    call: Spur,
  ) -> Result<(), EvalError> {
    let call_str = interner().resolve(&call);

    if let Ok(intrinsic) = Intrinsic::try_from(call_str) {
      return self.eval_intrinsic(trace_expr, intrinsic);
    }

    if let Some(value) = self.scope_item(call_str) {
      if value.is_function() {
        self.eval_expr(Expr::Lazy(Box::new(Expr::Call(call))))?;
        self.eval_intrinsic(trace_expr, Intrinsic::Get)?;
        self.eval_intrinsic(trace_expr, Intrinsic::Call)?;

        Ok(())
      } else {
        self.push(self.scope_item(call_str).unwrap_or(Expr::Nil));
        Ok(())
      }
    } else {
      Err(EvalError {
        expr: trace_expr.clone(),
        program: self.clone(),
        message: format!("unknown call {call_str}"),
      })
    }
  }

  fn eval_expr(&mut self, expr: Expr) -> Result<(), EvalError> {
    match expr.clone() {
      Expr::Call(call) => self.eval_call(&expr, call),
      Expr::Lazy(block) => {
        self.push(*block);
        Ok(())
      }
      Expr::List(list) => {
        let stack_len = self.stack.len();

        self.eval(list)?;

        let list_len = self.stack.len() - stack_len;

        let mut list = iter::repeat_with(|| self.pop(&expr).unwrap())
          .take(list_len)
          .collect::<Vec<_>>();
        list.reverse();

        self.push(Expr::List(list));

        Ok(())
      }
      Expr::Fn(_) => Ok(()),
      expr => {
        self.push(expr);
        Ok(())
      }
    }
  }

  pub fn eval_string(&mut self, line: &str) -> Result<(), EvalError> {
    let lexer = Lexer::new(line);
    let parser = Parser::new(lexer);
    // TODO: It might be time to add a proper EvalError enum.
    let exprs = parser.parse().map_err(|e| EvalError {
      program: self.clone(),
      message: e.to_string(),
      expr: Expr::Nil,
    })?;

    self.eval(exprs)
  }

  pub fn eval(&mut self, exprs: Vec<Expr>) -> Result<(), EvalError> {
    let mut clone = self.clone();
    let result = exprs.into_iter().try_for_each(|expr| clone.eval_expr(expr));

    self.loaded_files = clone.loaded_files;

    match result {
      Ok(x) => {
        // TODO: Store each operation in an append-only operations list, and
        //       rollback if there is an error.
        self.stack = clone.stack;
        self.scopes = clone.scopes;
        self.interner = clone.interner;

        Ok(x)
      }
      Err(e) => Err(e),
    }
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
      assert_eq!(
        program.stack,
        vec![Expr::Call(interner().get_or_intern_static("var"))]
      );
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
        program.scopes,
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
      assert_eq!(program.scopes, vec![HashMap::new()]);
    }

    #[test]
    fn auto_calling_functions() {
      let mut program = Program::new();
      program
        .eval_string("'(fn 1 2 +) 'is-three set is-three")
        .unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(3)]);
    }

    #[test]
    fn only_auto_call_functions() {
      let mut program = Program::new();
      program
        .eval_string("'(1 2 +) 'is-three set is-three")
        .unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::List(vec![
          Expr::Integer(1),
          Expr::Integer(2),
          Expr::Call(interner().get_or_intern_static("+"))
        ])]
      );
    }

    #[test]
    fn getting_function_body() {
      let mut program = Program::new();
      program
        .eval_string("'(fn 1 2 +) 'is-three set 'is-three get")
        .unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::List(vec![
          Expr::Fn(true),
          Expr::Integer(1),
          Expr::Integer(2),
          Expr::Call(interner().get_or_intern_static("+"))
        ])]
      );
    }

    #[test]
    fn assembling_functions_in_code() {
      let mut program = Program::new();
      program
        .eval_string("'() 'fn insert 1 insert 2 insert '+ insert dup call")
        .unwrap();
      assert_eq!(
        program.stack,
        vec![
          Expr::List(vec![
            Expr::Fn(true),
            Expr::Integer(1),
            Expr::Integer(2),
            Expr::Call(interner().get_or_intern_static("+"))
          ]),
          Expr::Integer(3)
        ]
      );
    }

    mod scope {
      use super::*;

      #[test]
      fn functions_are_isolated() {
        let mut program = Program::new();
        program
          .eval_string(
            "0 'a set
            '(fn 5 'a set)

            '(fn 1 'a set call) call",
          )
          .unwrap();
        assert_eq!(
          program.scopes,
          vec![HashMap::from_iter(vec![(
            "a".to_string(),
            Expr::Integer(0)
          )]),]
        )
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
        program.scopes,
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
      program.eval_string("(1 2 3) list-pop").unwrap();
      assert_eq!(
        program.stack,
        vec![
          Expr::List(vec![Expr::Integer(1), Expr::Integer(2)]),
          Expr::Integer(3)
        ]
      );
    }

    #[test]
    fn shifting_from_list() {
      let mut program = Program::new();
      program.eval_string("(1 2 3) list-shift").unwrap();
      assert_eq!(
        program.stack,
        vec![
          Expr::List(vec![Expr::Integer(2), Expr::Integer(3)]),
          Expr::Integer(1)
        ]
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
          Expr::String(interner().get_or_intern_static("4"))
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
          Expr::Call(interner().get_or_intern_static("+"))
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

    #[test]
    fn getting_nth_item_of_list_negative_index() {
      let mut program = Program::new();
      program.eval_string("(1 2 3) -1 nth").unwrap();
      assert_eq!(program.stack, vec![Expr::Integer(3)]);
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
          Expr::String(interner().get_or_intern_static("a")),
          Expr::String(interner().get_or_intern_static("b")),
          Expr::String(interner().get_or_intern_static("c"))
        ])]
      );
    }

    #[test]
    fn joining_to_string() {
      let mut program = Program::new();
      program
        .eval_string("(\"a\" 3 \"hello\" 1.2) \"\" join")
        .unwrap();

      assert_eq!(
        program.stack,
        vec![Expr::String(interner().get_or_intern_static("a3hello1.2"))]
      );
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
      assert_eq!(
        program.stack,
        vec![Expr::String(interner().get_or_intern_static("correct"))]
      );
    }

    #[test]
    fn if_empty_condition() {
      let mut program = Program::new();
      program
        .eval_string("1 2 + 3 = '(\"correct\") '() if")
        .unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::String(interner().get_or_intern_static("correct"))]
      );
    }

    #[test]
    fn if_else_true() {
      let mut program = Program::new();
      program
        .eval_string("1 2 + 3 = '(\"incorrect\") '(\"correct\") '() ifelse")
        .unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::String(interner().get_or_intern_static("correct"))]
      );
    }

    #[test]
    fn if_else_false() {
      let mut program = Program::new();
      program
        .eval_string("1 2 + 2 = '(\"incorrect\") '(\"correct\") '() ifelse")
        .unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::String(interner().get_or_intern_static("incorrect"))]
      );
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
      assert_eq!(
        program.stack,
        vec![Expr::String(interner().get_or_intern_static("1"))]
      );
    }

    #[test]
    fn to_call() {
      let mut program = Program::new();
      program.eval_string("\"a\" tocall").unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::Call(interner().get_or_intern_static("a"))]
      );
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
      assert_eq!(
        program.stack,
        vec![Expr::String(interner().get_or_intern_static("integer"))]
      );
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

    #[test]
    fn list_into_lazy() {
      let mut program = Program::new();
      program.eval_string("(1 2 3) lazy").unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::Lazy(
          Expr::List(vec![
            Expr::Integer(1),
            Expr::Integer(2),
            Expr::Integer(3)
          ])
          .into()
        )]
      );
    }

    #[test]
    fn call_into_lazy() {
      let mut program = Program::new();
      program.eval_string("'set lazy").unwrap();
      assert_eq!(
        program.stack,
        vec![Expr::Lazy(
          Expr::Call(interner().get_or_intern_static("set")).into()
        )]
      );
    }
  }
}
