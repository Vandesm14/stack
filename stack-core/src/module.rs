use std::{collections::HashMap, sync::Arc};

use crate::{
  context::Context,
  engine::{Engine, RunError},
  expr::Expr,
  symbol::Symbol,
};

// pub type Func = fn(&Engine, Context, Expr) -> Result<Context, RunError>;
pub type Func =
  Arc<dyn Fn(&Engine, Context, Expr) -> Result<Context, RunError>>;

#[derive(Clone)]
pub struct Module {
  name: Symbol,
  funcs: HashMap<Symbol, Func>,
}

impl Module {
  #[inline]
  pub fn new(name: Symbol) -> Self {
    Self {
      name,
      funcs: HashMap::new(),
    }
  }

  #[inline]
  pub fn with_func(mut self, name: Symbol, func: Func) -> Self {
    self.add_func(name, func);
    self
  }

  #[inline]
  pub fn add_func(&mut self, name: Symbol, func: Func) -> &mut Self {
    self.funcs.insert(name, func);
    self
  }

  #[inline]
  pub const fn name(&self) -> Symbol {
    self.name
  }

  #[inline]
  pub fn func(&self, name: Symbol) -> Option<Func> {
    self.funcs.get(&name).cloned()
  }
}
