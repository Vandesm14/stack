use std::str::FromStr;

use crate::{
  expr::{Expr, ExprKind},
  intrinsic::Intrinsic,
  parser,
  prelude::Lexer,
  source::Source,
  val::Val,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
  Push(Val),
  Intrinsic(Intrinsic),
  End,
}

type Ops = Vec<Op>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum VMError {
  #[default]
  Unknown,

  Halt,
  IPBounds,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct VM {
  ops: Ops,
  ip: usize,

  registers: Vec<Val>,
  stack: Vec<Val>,
  sp: usize,
}

impl VM {
  pub fn new() -> Self {
    Self {
      ops: Ops::new(),
      ip: 0,

      registers: Vec::new(),
      stack: Vec::new(),
      sp: 0,
    }
  }

  pub fn stack_pop(&mut self) -> Result<Val, VMError> {
    match self.stack.pop() {
      Some(val) => Ok(val),
      None => Err(VMError::Unknown),
    }
  }

  pub fn stack_push(&mut self, val: Val) {
    self.stack.push(val);
  }

  pub fn compile_expr(&self, expr: Expr) -> Op {
    match expr.kind {
      ExprKind::Nil => todo!(),
      ExprKind::Boolean(_) => todo!(),
      ExprKind::Integer(int) => Op::Push(Val::Integer(int)),
      ExprKind::Float(_) => todo!(),
      ExprKind::String(_) => todo!(),
      ExprKind::Symbol(symbol) => {
        if let Ok(intrinsic) = Intrinsic::from_str(symbol.as_str()) {
          Op::Intrinsic(intrinsic)
        } else {
          todo!()
        }
      }
      ExprKind::Lazy(_) => todo!(),
      ExprKind::List(_) => todo!(),
      ExprKind::Record(_) => todo!(),
      ExprKind::Function { scope, body } => todo!(),
      ExprKind::SExpr { call, body } => todo!(),
      ExprKind::Underscore => todo!(),
    }
  }

  pub fn compile(&mut self, exprs: Vec<Expr>) {
    for expr in exprs.into_iter() {
      self.ops.push(self.compile_expr(expr));
    }

    self.ops.push(Op::End);
  }

  pub fn step(&mut self) -> Result<(), VMError> {
    // We have to copy here so we can pass the mutable ref to self in the match
    // at the end of this fn.
    let op = self.ops.get(self.ip).copied();

    let ip = self.ip.checked_add(1).map(|res| res.min(self.ops.len()));
    if let Some(ip) = ip {
      self.ip = ip;
    } else {
      return Err(VMError::IPBounds);
    }

    if let Some(op) = op {
      match op {
        Op::Push(val) => {
          self.stack.push(val);
          Ok(())
        }
        Op::Intrinsic(intrinsic) => intrinsic.run(self),
        Op::End => Err(VMError::Halt),
      }
    } else {
      todo!("ip out of bounds")
    }
  }
}
