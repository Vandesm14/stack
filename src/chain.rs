use core::cell::RefCell;
use std::{borrow::BorrowMut, rc::Rc};

// TODO: Look into a more efficient way to implement this.

// #[derive(Debug, Clone)]
// pub enum Chain<T> {
//   Root(Rc<RefCell<T>>),
//   Link(Rc<RefCell<Self>>),
// }

// impl<T> PartialEq for Chain<T>
// where
//   T: PartialEq,
// {
//   fn eq(&self, other: &Self) -> bool {
//     RefCell::borrow(&self.root()).eq(&RefCell::borrow(&other.root()))
//   }
// }

// impl<T> Chain<T> {
//   #[inline]
//   pub fn new(value: T) -> Self {
//     Self::Root(Rc::new(RefCell::new(value)))
//   }

//   pub fn root(&self) -> Rc<RefCell<T>> {
//     match self {
//       Self::Root(root) => root.clone(),
//       Self::Link(link) => RefCell::borrow(link).root(),
//     }
//   }

//   pub fn link(&self) -> Self {
//     match self {
//       Self::Root(root) => {
//         Self::Link(Rc::new(RefCell::new(Self::Root(root.clone()))))
//       }
//       Self::Link(link) => {
//         Self::Link(Rc::new(RefCell::new(Self::Link(link.clone()))))
//       }
//     }
//   }

//   pub fn unlink_with<F>(&self, f: F)
//   where
//     F: FnOnce(&Self) -> T,
//   {
//     match self {
//       Self::Root(_) => {}
//       Self::Link(link) => *RefCell::borrow_mut(link) = Self::new(f(self)),
//     }
//   }
// }

// impl<T> Chain<T>
// where
//   T: Clone,
// {
//   #[inline]
//   pub fn unlink(&self) {
//     self.unlink_with(|chain| RefCell::borrow(&chain.root()).clone())
//   }

//   #[inline]
//   pub fn val(&self) -> T {
//     RefCell::borrow(&self.root()).clone()
//   }
// }

#[derive(Debug, PartialEq, Clone)]
pub struct Chain<T> {
  value: Rc<RefCell<T>>,
  child: Option<Rc<RefCell<Chain<T>>>>,
}

impl<T> Chain<T> {
  pub fn new(value: T) -> Self {
    Self {
      value: Rc::new(RefCell::new(value)),
      child: None,
    }
  }

  pub fn val(&self) -> T
  where
    T: Clone,
  {
    self.value.borrow().clone()
  }

  pub fn link(&mut self) -> Self
  where
    T: Clone,
  {
    let child = Self {
      value: self.value.clone(),
      child: None,
    };
    *self.child.borrow_mut() = Some(Rc::new(RefCell::new(child.clone())));

    child
  }

  pub fn root(&self) -> Rc<RefCell<T>> {
    self.value.clone()
  }

  pub fn unlink_with_rc(&mut self, value: Rc<RefCell<T>>)
  where
    T: Clone,
  {
    self.value = value.clone();
    if let Some(child) = &self.child {
      RefCell::borrow_mut(child).unlink_with_rc(value);
    }
  }

  pub fn unlink_with(&mut self, val: T)
  where
    T: Clone,
  {
    self.unlink_with_rc(Rc::new(RefCell::new(val)));
  }

  pub fn set(&mut self, val: T)
  where
    T: Clone,
  {
    *RefCell::borrow_mut(&self.value) = val;
  }
}

// TODO: Add tests
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn create_chain() {
    let chain = Chain::new(1);
    assert_eq!(chain.val(), 1);
  }

  #[test]
  fn link_chain() {
    let mut chain = Chain::new(1);
    let link = chain.link();

    assert_eq!(chain.val(), 1);
    assert_eq!(link.val(), 1);
  }

  #[test]
  fn change_root_value() {
    let mut chain = Chain::new(1);
    let link = chain.link();
    chain.set(2);

    assert_eq!(chain.val(), 2);
    assert_eq!(link.val(), 2);
  }

  #[test]
  fn change_value_with_link() {
    let mut chain = Chain::new(1);
    let mut link = chain.link();
    link.set(2);

    assert_eq!(chain.val(), 2);
    assert_eq!(link.val(), 2);
  }

  #[test]
  fn unlink_chain() {
    let mut a = Chain::new(1);
    let mut b = a.link();
    let c = b.link();

    assert_eq!(a.val(), 1);
    assert_eq!(b.val(), 1);
    assert_eq!(c.val(), 1);

    b.unlink_with(2);

    assert_eq!(a.val(), 1);
    assert_eq!(b.val(), 2);
    assert_eq!(c.val(), 2);
  }

  #[test]
  fn cloned_chains_are_links() {
    let mut a = Chain::new(1);
    let mut b = a.link();
    let clone = b.clone();

    assert_eq!(a.val(), 1);
    assert_eq!(b.val(), 1);
    assert_eq!(clone.val(), 1);

    b.set(2);

    assert_eq!(a.val(), 2);
    assert_eq!(b.val(), 2);
    assert_eq!(clone.val(), 2);

    b.unlink_with(3);

    assert_eq!(a.val(), 2);
    assert_eq!(b.val(), 3);
    assert_eq!(clone.val(), 3);
  }
}
