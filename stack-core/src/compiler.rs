use core::fmt;
use std::{cell::RefCell, collections::HashMap, rc::Rc, str::FromStr};

use crate::{
  expr::{Expr, ExprKind},
  intrinsic::Intrinsic,
  symbol::Symbol,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
  Push(Expr),
  GetVal(Symbol),
  Intrinsic(Intrinsic),
  NoOp,

  Goto(usize, usize),
  Return,
}

pub type ScopeVal = Rc<RefCell<Expr>>;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Block {
  scope: HashMap<Symbol, ScopeVal>,
  ops: Vec<Op>,
}

impl Block {
  pub fn new() -> Self {
    Self {
      scope: HashMap::new(),
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
  End,
}

impl fmt::Display for VMError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      VMError::Unknown => write!(f, "Unknown error"),
      VMError::Halt => write!(f, "Halted"),
      VMError::IPBounds => write!(f, "Instruction pointer out of bounds"),
      VMError::End => write!(f, "End of execution"),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct VM {
  call_stack: Vec<(usize, usize)>,
  blocks: Vec<Block>,
  bp: usize,
  ip: usize,

  registers: Vec<Expr>,
  stack: Vec<Expr>,
}

impl VM {
  pub fn new() -> Self {
    Self {
      call_stack: Vec::new(),
      blocks: Vec::new(),
      bp: 0,
      ip: 0,

      registers: Vec::new(),
      stack: Vec::new(),
    }
  }

  pub fn def_const(&mut self, symbol: Symbol, value: Expr) {
    if let Some(block) = self.block_mut() {
      block.scope.insert(symbol, Rc::new(RefCell::new(value)));
    } else {
      todo!()
    }
  }

  pub fn set_const(&mut self, symbol: Symbol, value: Expr) {
    if let Some(block) = self.block_mut() {
      if let Some(item) = block.scope.get(&symbol) {
        *item.borrow_mut() = value;
      } else {
        todo!()
      }
    } else {
      todo!()
    }
  }

  pub fn get_const(&self, symbol: Symbol) -> Option<&ScopeVal> {
    if let Some(block) = self.block() {
      block.scope.get(&symbol)
    } else {
      todo!()
    }
  }

  pub fn stack(&self) -> &[Expr] {
    &self.stack
  }

  pub fn stack_mut(&mut self) -> &mut [Expr] {
    &mut self.stack
  }

  pub fn stack_pop(&mut self) -> Result<Expr, VMError> {
    match self.stack.pop() {
      Some(val) => Ok(val),
      None => Err(VMError::Unknown),
    }
  }

  pub fn stack_push(&mut self, expr: Expr) {
    self.stack.push(expr);
  }

  pub fn compile_expr(&mut self, expr: Expr) -> Op {
    match expr.kind {
      ExprKind::Nil => Op::Push(expr),
      ExprKind::Boolean(_) => Op::Push(expr),
      ExprKind::Integer(_) => Op::Push(expr),
      ExprKind::Float(_) => Op::Push(expr),
      ExprKind::String(_) => Op::Push(expr),
      ExprKind::Symbol(symbol) => {
        if let Ok(intrinsic) = Intrinsic::from_str(symbol.as_str()) {
          Op::Intrinsic(intrinsic)
        } else {
          Op::GetVal(symbol)
        }
      }
      ExprKind::Lazy(expr) => Op::Push(*expr),
      ExprKind::List(_) => Op::Push(expr),
      ExprKind::Record(_) => Op::Push(expr),
      ExprKind::Function { scope, body } => {
        let mut fn_block = Block::new();
        for expr in body.into_iter() {
          fn_block.ops.push(self.compile_expr(expr));
        }

        fn_block.ops.push(Op::Return);
        self.blocks.push(fn_block);

        Op::Goto(self.blocks.len() - 1, 0)
      }
      ExprKind::SExpr { call, body } => todo!(),
      ExprKind::Underscore => Op::NoOp,
    }
  }

  pub fn block(&self) -> Option<&Block> {
    self.blocks.get(self.bp)
  }

  pub fn block_mut(&mut self) -> Option<&mut Block> {
    self.blocks.get_mut(self.bp)
  }

  pub fn op(&self) -> Option<&Op> {
    self.block().and_then(|block| block.ops.get(self.ip))
  }

  pub fn compile(&mut self, exprs: Vec<Expr>) {
    let mut block = Block::new();
    for expr in exprs.into_iter() {
      block.ops.push(self.compile_expr(expr));
    }

    self.blocks.push(block);
    self.bp = self.blocks.len() - 1;
  }

  pub fn run_op(&mut self, op: Op) -> Result<(), VMError> {
    match op {
      Op::Push(val) => {
        self.stack.push(val);

        Ok(())
      }
      Op::GetVal(symbol) => {
        if let Some(expr) = self.get_const(symbol).cloned() {
          self.stack_push(expr.borrow().clone());

          Ok(())
        } else {
          todo!("no var")
        }
      }
      Op::Intrinsic(intrinsic) => intrinsic.run(self),
      Op::NoOp => Ok(()),

      Op::Goto(bp, ip) => {
        self.call_stack.push((self.bp, self.ip));

        self.bp = bp;
        self.ip = ip;

        Ok(())
      }
      Op::Return => {
        if let Some((bp, ip)) = self.call_stack.pop() {
          self.bp = bp;
          self.ip = ip;

          Ok(())
        } else {
          todo!("None call stack")
        }
      }
    }
  }

  pub fn step(&mut self) -> Result<(), VMError> {
    let op = self.op().cloned();
    if let Some(op) = op {
      let ip = self.ip.checked_add(1);
      if let Some(ip) = ip {
        self.ip = ip;
      } else {
        return Err(VMError::IPBounds);
      }

      self.run_op(op)
    } else {
      Err(VMError::End)
    }
  }

  pub fn run(&mut self) -> Result<&[Expr], VMError> {
    loop {
      if let Err(err) = self.step() {
        return match err {
          VMError::End => Ok(self.stack()),

          err => Err(err),
        };
      }
    }
  }
}
