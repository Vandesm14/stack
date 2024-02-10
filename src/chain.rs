use core::cell::RefCell;
use std::rc::Rc;

// TODO: Look into a more efficient way to implement this.

#[derive(Debug, Clone)]
pub enum Chain<T> {
  Root(Rc<RefCell<T>>),
  Link(Rc<RefCell<Self>>),
}

impl<T> PartialEq for Chain<T>
where
  T: PartialEq,
{
  fn eq(&self, other: &Self) -> bool {
    RefCell::borrow(&self.root()).eq(&RefCell::borrow(&other.root()))
  }
}

impl<T> Chain<T> {
  #[inline]
  pub fn new(value: T) -> Self {
    Self::Root(Rc::new(RefCell::new(value)))
  }

  pub fn root(&self) -> Rc<RefCell<T>> {
    match self {
      Self::Root(root) => root.clone(),
      Self::Link(link) => RefCell::borrow(link).root(),
    }
  }

  pub fn link(&self) -> Self {
    match self {
      Self::Root(root) => {
        Self::Link(Rc::new(RefCell::new(Self::Root(root.clone()))))
      }
      Self::Link(link) => {
        Self::Link(Rc::new(RefCell::new(Self::Link(link.clone()))))
      }
    }
  }

  pub fn unlink_with<F>(&self, f: F)
  where
    F: FnOnce(&Self) -> T,
  {
    match self {
      Self::Root(_) => {}
      Self::Link(link) => *RefCell::borrow_mut(link) = Self::new(f(self)),
    }
  }
}

impl<T> Chain<T>
where
  T: Clone,
{
  #[inline]
  pub fn unlink(&self) {
    self.unlink_with(|chain| RefCell::borrow(&chain.root()).clone())
  }
}

// TODO: Add tests
