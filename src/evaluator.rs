use ariadne::{Color, Label, Report, ReportKind, Source};
use lasso::Spur;

use crate::{
  interner::interner, module, Expr, ExprKind, Func, Journal, JournalOp, Lexer,
  Module, Parser, Scanner, Scope, Span, Type,
};
use core::{fmt, iter};
use std::{collections::HashMap, fs, time::SystemTime};

#[derive(Debug, Clone)]
pub struct SourceFile {
  pub contents: Spur,
  pub mtime: SystemTime,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EvalErrorKind {
  Push,
  StackUnderflow,
  UnknownCall,
  ParseError,
  Message(String),
  ExpectedFound(Type, Type),
  Halt,
  Panic(String),
  UnableToRead(String, String),
  Cast(Type),
}

impl fmt::Display for EvalErrorKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Push => write!(f, "failed to push to the stack"),
      Self::StackUnderflow => write!(f, "stack underflow"),
      Self::UnknownCall => write!(f, "unknown call"),
      Self::ParseError => write!(f, "parse error"),
      Self::ExpectedFound(expected, found) => {
        write!(f, "expected {}, found {}", expected, found)
      }
      Self::Message(message) => write!(f, "{}", message),
      Self::Halt => write!(f, "halted"),
      Self::Panic(message) => write!(f, "panic: {}", message),
      Self::UnableToRead(filename, error) => {
        write!(f, "unable to read {}: {}", filename, error)
      }
      Self::Cast(t) => {
        write!(f, "unable to cast into {}", t)
      }
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvalError {
  pub kind: EvalErrorKind,
  pub expr: Option<Expr>,
}

impl fmt::Display for EvalError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "Error: {}", self.kind)?;
    writeln!(
      f,
      "Expr: {}",
      match self.expr.clone() {
        Some(expr) => expr.to_string(),
        None => "no expr to display".into(),
      }
    )
  }
}

impl EvalError {
  pub fn print_report(&self) {
    let out = Color::Red;

    let source_file: Option<&str> = self.expr.as_ref().and_then(|expr| {
      expr
        .debug_data
        .source_file
        .and_then(|spur| interner().try_resolve(&spur))
    });
    let file_name = source_file.unwrap_or("");

    let span: Option<Span> =
      self.expr.as_ref().and_then(|expr| expr.debug_data.span);
    let span = match span {
      Some(span) => span.start..span.end,
      None => 0..0,
    };

    let source = match source_file {
      Some(path) => fs::read_to_string(path).ok().unwrap_or("".into()),
      None => "".into(),
    };

    Report::build(ReportKind::Error, file_name, span.start)
      .with_message(self.to_string())
      .with_label(
        Label::new((file_name, span))
          .with_message("error occurs here")
          .with_color(out),
      )
      .finish()
      .print((file_name, Source::from(source)))
      .unwrap();
  }
}

#[derive(Debug, Clone)]
pub struct Program {
  pub stack: Vec<Expr>,
  pub scopes: Vec<Scope>,
  pub funcs: HashMap<Spur, Func>,
  pub sources: HashMap<String, SourceFile>,
  pub journal: Journal,
  pub debug: bool,
}

impl Default for Program {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for Program {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Stack: [")?;

    self.stack.iter().enumerate().try_for_each(|(i, expr)| {
      if i == self.stack.len() - 1 {
        write!(f, "{}", expr.to_pretty_string())
      } else {
        write!(f, "{}, ", expr.to_pretty_string())
      }
    })?;
    write!(f, "]")?;

    // if !self.scopes.is_empty() {
    //   writeln!(f, "Scope:")?;

    //   let layer = self.scopes.last().unwrap();
    //   let items = layer.items.len();
    //   for (item_i, (key, value)) in
    //     layer.items.iter().sorted_by_key(|(s, _)| *s).enumerate()
    //   {
    //     if item_i == items - 1 {
    //       write!(
    //         f,
    //         " + {}: {}",
    //         interner().resolve(key),
    //         match value.borrow().val() {
    //           Some(expr) => expr.to_string(),
    //           None => "None".to_owned(),
    //         }
    //       )?;
    //     } else {
    //       writeln!(
    //         f,
    //         " + {}: {}",
    //         interner().resolve(key),
    //         match value.borrow().val() {
    //           Some(expr) => expr.to_string(),
    //           None => "None".to_owned(),
    //         }
    //       )?;
    //     }
    //   }
    // }

    write!(f, "{}", self.journal)?;

    Ok(())
  }
}

impl Program {
  #[inline]
  pub fn new() -> Self {
    Self {
      stack: vec![],
      scopes: vec![Scope::new()],
      funcs: HashMap::new(),
      sources: HashMap::new(),
      journal: Journal::new(),
      debug: false,
    }
  }

  pub fn with_core(self) -> Result<Self, EvalError> {
    let mut program = self;
    module::core::Core::default().link(&mut program)?;
    program.journal.clear();
    Ok(program)
  }

  pub fn with_module<M>(self, module: M) -> Result<Self, EvalError>
  where
    M: Module,
  {
    let mut program = self;
    module.link(&mut program)?;
    program.journal.clear();
    Ok(program)
  }

  pub fn with_debug(mut self) -> Self {
    self.debug = true;
    self
  }

  pub fn loaded_files(&self) -> impl Iterator<Item = &str> {
    self.sources.keys().map(|s| s.as_str())
  }

  pub fn pop(&mut self, trace_expr: &Expr) -> Result<Expr, EvalError> {
    match self.stack.pop() {
      Some(expr) => {
        if self.debug {
          self.journal.op(JournalOp::Pop(expr.clone()));
        }
        Ok(expr)
      }
      None => Err(EvalError {
        expr: Some(trace_expr.clone()),
        kind: EvalErrorKind::StackUnderflow,
      }),
    }
  }

  pub fn push(&mut self, expr: Expr) -> Result<(), EvalError> {
    let expr = if expr.val.is_function() {
      let mut scanner =
        Scanner::new(self.scopes.last().unwrap().duplicate(), &self.funcs);

      match scanner.scan(expr.clone()) {
        Ok(expr) => expr,
        Err(message) => {
          return Err(EvalError {
            expr: Some(expr),
            kind: EvalErrorKind::Message(message),
          })
        }
      }
    } else {
      expr
    };

    if self.debug {
      self.journal.op(JournalOp::Push(expr.clone()));
    }

    self.stack.push(expr);
    // TODO: I don't think we need to commit after each push.
    // self.journal.commit();

    Ok(())
  }

  pub fn scope_item(&self, symbol: &str) -> Option<Expr> {
    self
      .scopes
      .last()
      .and_then(|layer| layer.get_val(interner().get_or_intern(symbol)))
  }

  pub fn def_scope_item(&mut self, symbol: &str, value: Expr) {
    if let Some(layer) = self.scopes.last_mut() {
      layer
        .define(interner().get_or_intern(symbol), value)
        .unwrap();
    }
  }

  pub fn set_scope_item(
    &mut self,
    symbol: &str,
    value: Expr,
  ) -> Result<(), EvalError> {
    if let Some(layer) = self.scopes.last_mut() {
      match layer.set(interner().get_or_intern(symbol), value.clone()) {
        Ok(_) => Ok(()),
        Err(string) => Err(EvalError {
          expr: Some(value),
          kind: EvalErrorKind::Message(string),
        }),
      }
    } else {
      Err(EvalError {
        expr: Some(value),
        kind: EvalErrorKind::Message("no scopes to set to".into()),
      })
    }
  }

  pub fn remove_scope_item(&mut self, symbol: &str) {
    if let Some(layer) = self.scopes.last_mut() {
      layer.remove(interner().get_or_intern(symbol));
    }
  }

  pub fn push_scope(&mut self, scope: Scope) {
    self.scopes.push(scope);
  }

  pub fn pop_scope(&mut self) {
    self.scopes.pop();
  }

  /// Reads, Lexes, Parses, and Evaluates a file
  pub fn eval_file(&mut self, path: &str) -> Result<(), EvalError> {
    let contents = fs::read_to_string(path);

    match contents {
      Ok(contents) => {
        let lexer = Lexer::new(&contents);
        let parser = Parser::new(lexer, interner().get_or_intern(path));
        // TODO: It might be time to add a proper EvalError enum.
        let exprs = parser.parse().map_err(|_| EvalError {
          expr: None,
          kind: EvalErrorKind::ParseError,
        })?;

        self.eval(exprs)
      }
      Err(err) => Err(EvalError {
        expr: None,
        kind: EvalErrorKind::UnableToRead(path.into(), err.to_string()),
      }),
    }
  }

  /// Lexes, Parses, and Evaluates a string, setting the source_file of the exprs to `name`
  pub fn eval_string_with_name(
    &mut self,
    line: &str,
    name: &str,
  ) -> Result<(), EvalError> {
    let lexer = Lexer::new(line);
    let parser = Parser::new(lexer, interner().get_or_intern(name));
    // TODO: It might be time to add a proper EvalError enum.
    let exprs = parser.parse().map_err(|_| EvalError {
      expr: None,
      kind: EvalErrorKind::ParseError,
    })?;

    self.eval(exprs)
  }

  /// Lexes, Parses, and Evaluates a string
  pub fn eval_string(&mut self, line: &str) -> Result<(), EvalError> {
    self.eval_string_with_name(line, "internal")
  }

  /// Evaluates a vec of expressions
  pub fn eval(&mut self, exprs: Vec<Expr>) -> Result<(), EvalError> {
    let result = exprs.into_iter().try_for_each(|expr| self.eval_expr(expr));
    match result {
      Ok(_) => Ok(()),
      Err(err) => {
        self.journal.commit();
        Err(err)
      }
    }
  }

  /// Evaluates an expression and makes decisions on how to evaluate it
  /// - Lazy expressions don't get evaluated
  /// - Lists get evaluated in order
  /// - Calls get run through [`Self::eval_symbol`]
  pub fn eval_expr(&mut self, expr: Expr) -> Result<(), EvalError> {
    match expr.clone().val {
      ExprKind::Call(call) => self.eval_symbol(&expr, call),
      ExprKind::Lazy(block) => self.push(*block),
      ExprKind::List(list) => {
        let stack_len = self.stack.len();

        self.eval(list)?;

        let list_len = self.stack.len() - stack_len;

        let mut list = iter::repeat_with(|| self.pop(&expr).unwrap())
          .take(list_len)
          .collect::<Vec<_>>();
        list.reverse();

        self.push(Expr {
          val: ExprKind::List(list),
          debug_data: expr.debug_data,
        })
      }
      ExprKind::Fn(_) => Ok(()),
      _ => self.push(expr),
    }
  }

  /// Makes decisions for how to evaluate a symbol (calls) such as
  /// - Running an intrinsic
  /// - Getting the value from the scope
  /// - Calling functions through [`Self::auto_call`]
  fn eval_symbol(
    &mut self,
    trace_expr: &Expr,
    symbol: Spur,
  ) -> Result<(), EvalError> {
    let symbol_str = interner().resolve(&symbol);

    if self.debug {
      self.journal.commit();
    }

    if let Some(func) = self.funcs.get(&symbol) {
      if self.debug {
        self.journal.op(JournalOp::FnCall(trace_expr.clone()));
      }
      let result = func(self, trace_expr);
      if self.debug && result.is_ok() {
        self.journal.commit();
      }

      return result;
    }

    if let Some(value) = self.scope_item(symbol_str) {
      if value.val.is_function() {
        self.auto_call(trace_expr, value)
      } else {
        if self.debug {
          self.journal.op(JournalOp::Call(trace_expr.clone()));
        }
        let result = self.push(value);
        if self.debug {
          self.journal.commit();
        }

        result
      }
    } else {
      Err(EvalError {
        kind: EvalErrorKind::UnknownCall,
        expr: Some(trace_expr.clone()),
      })
    }
  }

  /// Handles auto-calling symbols (calls) when they're pushed to the stack
  /// This is also triggered by the `call` keyword
  pub fn auto_call(
    &mut self,
    trace_expr: &Expr,
    expr: Expr,
  ) -> Result<(), EvalError> {
    match expr.val {
      ExprKind::Call(_) => self.eval_expr(expr),
      item @ ExprKind::List(_) => match item.is_function() {
        true => {
          if self.debug {
            self.journal.op(JournalOp::FnCall(trace_expr.clone()));
          }

          let fn_symbol = item.fn_symbol().unwrap();
          let fn_body = item.fn_body().unwrap();

          if fn_symbol.scoped {
            self.push_scope(fn_symbol.scope.clone());
          }

          if self.debug {
            self.journal.commit();
            self.journal.op(JournalOp::FnStart(fn_symbol.scoped));
          }

          match self.eval(fn_body.to_vec()) {
            Ok(_) => {
              if self.debug {
                self.journal.commit();
                self.journal.op(JournalOp::FnEnd);
              }

              if fn_symbol.scoped {
                self.pop_scope();
              }

              Ok(())
            }
            Err(err) => Err(err),
          }
        }
        false => {
          let ExprKind::List(list) = item else {
            unreachable!()
          };
          self.eval(list)
        }
      },
      _ => Err(EvalError {
        expr: Some(trace_expr.clone()),
        kind: EvalErrorKind::ExpectedFound(
          Type::Set(vec![
            Type::Call,
            Type::List(vec![Type::FnScope, Type::Any]),
          ]),
          expr.val.type_of(),
        ),
      }),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{simple_exprs, TestExpr};

  #[test]
  fn implicitly_adds_to_stack() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 2").unwrap();
    assert_eq!(
      simple_exprs(program.stack),
      vec![TestExpr::Integer(1), TestExpr::Integer(2)]
    );
  }

  #[test]
  fn add_two_numbers() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 2 +").unwrap();
    assert_eq!(simple_exprs(program.stack), vec![TestExpr::Integer(3)]);
  }

  #[test]
  fn subtract_two_numbers() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 2 -").unwrap();
    assert_eq!(simple_exprs(program.stack), vec![TestExpr::Integer(-1)]);
  }

  #[test]
  fn multiply_two_numbers() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 2 *").unwrap();
    assert_eq!(simple_exprs(program.stack), vec![TestExpr::Integer(2)]);
  }

  #[test]
  fn divide_two_numbers() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 2 /").unwrap();
    assert_eq!(simple_exprs(program.stack), vec![TestExpr::Integer(0)]);

    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1.0 2.0 /").unwrap();
    assert_eq!(simple_exprs(program.stack), vec![TestExpr::Float(0.5)]);
  }

  #[test]
  fn modulo_two_numbers() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("10 5 %").unwrap();
    assert_eq!(simple_exprs(program.stack), vec![TestExpr::Integer(0)]);

    let mut program = Program::new().with_core().unwrap();
    program.eval_string("11 5 %").unwrap();
    assert_eq!(simple_exprs(program.stack), vec![TestExpr::Integer(1)]);
  }

  #[test]
  fn complex_operations() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("1 2 + 3 *").unwrap();
    assert_eq!(simple_exprs(program.stack), vec![TestExpr::Integer(9)]);
  }

  #[test]
  fn eval_from_stack() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("'(1 2 +) unwrap call").unwrap();
    assert_eq!(simple_exprs(program.stack), vec![TestExpr::Integer(3)]);
  }

  #[test]
  fn dont_eval_skips() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("6 'var def 'var").unwrap();
    assert_eq!(
      simple_exprs(program.stack),
      vec![TestExpr::Call(interner().get_or_intern_static("var"))]
    );
  }

  #[test]
  fn eval_lists() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("(1 2 3)").unwrap();
    assert_eq!(
      simple_exprs(program.stack),
      vec![TestExpr::List(vec![
        TestExpr::Integer(1),
        TestExpr::Integer(2),
        TestExpr::Integer(3)
      ])]
    );
  }

  #[test]
  fn eval_lists_eagerly() {
    let mut program = Program::new().with_core().unwrap();
    program.eval_string("6 'var def (var)").unwrap();
    assert_eq!(
      simple_exprs(program.stack),
      vec![TestExpr::List(vec![TestExpr::Integer(6)])]
    );
  }
}
