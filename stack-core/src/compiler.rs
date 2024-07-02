use std::str::FromStr;

use crate::{
  expr::{Expr, ExprKind},
  intrinsic::Intrinsic,
  val::Val,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
  Push(Val),
  Intrinsic(Intrinsic),
  Goto(usize, usize),

  Return,
  End,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Block {
  constants: Vec<Val>,
  ops: Vec<Op>,
}

impl Block {
  pub fn new() -> Self {
    Self {
      constants: Vec::new(),
      ops: Vec::new(),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum VMError {
  #[default]
  Unknown,

  Halt,
  IPBounds,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct VM {
  blocks: Vec<Block>,
  bp: usize,
  ip: usize,

  registers: Vec<Val>,
  stack: Vec<Val>,
}

impl VM {
  pub fn new() -> Self {
    Self {
      blocks: Vec::new(),
      bp: 0,
      ip: 0,

      registers: Vec::new(),
      stack: Vec::new(),
    }
  }

  pub fn stack(&self) -> &[Val] {
    &self.stack
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
    let mut block = Block::new();
    for expr in exprs.into_iter() {
      block.ops.push(self.compile_expr(expr));
    }

    block.ops.push(Op::End);
    self.blocks.push(block);
  }

  pub fn step(&mut self) -> Result<(), VMError> {
    let block = self.blocks.get(self.bp);
    if let Some(block) = block {
      // We have to clone here so we can pass the mutable ref to self in the
      // match at the end of this fn.
      let op = block.ops.get(self.ip).cloned();

      let ip = self.ip.checked_add(1).map(|res| res.min(block.ops.len()));
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

          Op::Goto(bp, sp) => todo!(),
          Op::Return => todo!(),
          Op::End => Err(VMError::Halt),
        }
      } else {
        todo!("None op")
      }
    } else {
      todo!("None block")
    }
  }
}
