#![allow(dead_code)]

use core::slice::{Iter, IterMut, SliceIndex};
use std::vec::IntoIter;

/// A [`Vec`] with at least one element.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub(crate) struct VecOne<T> {
  vec: Vec<T>,
}

impl<T> VecOne<T> {
  #[inline]
  pub fn new(one: T) -> Self {
    Self { vec: vec![one] }
  }

  #[inline]
  #[allow(clippy::len_without_is_empty)]
  pub fn len(&self) -> usize {
    self.vec.len()
  }

  #[inline]
  pub fn capacity(&self) -> usize {
    self.vec.capacity()
  }

  #[inline]
  pub fn push(&mut self, value: T) {
    self.vec.push(value)
  }

  #[inline]
  pub fn try_pop(&mut self) -> Option<T> {
    debug_assert_ne!(self.vec.len(), 0);

    if self.len() > 1 {
      self.vec.pop()
    } else {
      None
    }
  }

  #[inline]
  pub fn pop(mut self) -> (Option<Self>, T) {
    debug_assert_ne!(self.vec.len(), 0);

    // SAFETY: This is upheld by the invariants of this type.
    let value = unsafe { self.vec.pop().unwrap_unchecked() };

    if self.vec.is_empty() {
      (None, value)
    } else {
      (Some(self), value)
    }
  }

  #[inline]
  #[must_use]
  pub fn get<I>(&self, index: I) -> Option<&I::Output>
  where
    I: SliceIndex<[T]>,
  {
    debug_assert_ne!(self.vec.len(), 0);
    self.vec.get(index)
  }

  #[inline]
  #[must_use]
  pub fn get_mut<I>(&mut self, index: I) -> Option<&mut I::Output>
  where
    I: SliceIndex<[T]>,
  {
    debug_assert_ne!(self.vec.len(), 0);
    self.vec.get_mut(index)
  }

  #[inline]
  #[must_use]
  pub fn first(&self) -> &T {
    debug_assert_ne!(self.vec.len(), 0);
    // SAFETY: This is upheld by the invariants of this type.
    unsafe { self.vec.first().unwrap_unchecked() }
  }

  #[inline]
  #[must_use]
  pub fn first_mut(&mut self) -> &mut T {
    debug_assert_ne!(self.vec.len(), 0);
    // SAFETY: This is upheld by the invariants of this type.
    unsafe { self.vec.first_mut().unwrap_unchecked() }
  }

  #[inline]
  #[must_use]
  pub fn last(&self) -> &T {
    debug_assert_ne!(self.vec.len(), 0);
    // SAFETY: This is upheld by the invariants of this type.
    unsafe { self.vec.last().unwrap_unchecked() }
  }

  #[inline]
  #[must_use]
  pub fn last_mut(&mut self) -> &mut T {
    debug_assert_ne!(self.vec.len(), 0);
    // SAFETY: This is upheld by the invariants of this type.
    unsafe { self.vec.last_mut().unwrap_unchecked() }
  }

  #[inline]
  pub fn iter(&self) -> Iter<T> {
    self.vec.iter()
  }

  #[inline]
  pub fn iter_mut(&mut self) -> IterMut<T> {
    self.vec.iter_mut()
  }
}

impl<T> IntoIterator for VecOne<T> {
  type Item = T;
  type IntoIter = IntoIter<T>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.vec.into_iter()
  }
}

impl<T> Extend<T> for VecOne<T> {
  #[inline]
  fn extend<I>(&mut self, iter: I)
  where
    I: IntoIterator<Item = T>,
  {
    self.vec.extend(iter)
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn vec_one() {
    let mut vec = VecOne::new(123);

    assert_eq!(vec.len(), 1);

    assert_eq!(*vec.first(), 123);
    assert_eq!(*vec.first_mut(), 123);
    assert_eq!(*vec.last(), 123);
    assert_eq!(*vec.last_mut(), 123);

    assert_eq!(vec.try_pop(), None);

    vec.push(456);
    vec.push(789);

    assert_eq!(vec.len(), 3);

    assert_eq!(*vec.first(), 123);
    assert_eq!(*vec.first_mut(), 123);
    assert_eq!(*vec.last(), 789);
    assert_eq!(*vec.last_mut(), 789);

    assert_eq!(vec.try_pop(), Some(789));
    assert_eq!(vec.try_pop(), Some(456));
    assert_eq!(vec.try_pop(), None);

    assert_eq!(vec.len(), 1);

    vec.extend([456, 789]);

    assert!(vec.iter().eq([123, 456, 789].iter()));
    assert!(vec.iter_mut().eq([123, 456, 789].iter_mut()));
    assert!(vec.into_iter().eq([123, 456, 789].into_iter()));
  }

  #[test]
  fn vec_one_pop() {
    let mut vec = VecOne::new(123);
    vec.extend([456, 789]);

    let (vec, val) = vec.pop();
    assert!(vec.is_some());
    assert_eq!(val, 789);

    let (vec, val) = vec.unwrap().pop();
    assert!(vec.is_some());
    assert_eq!(val, 456);

    let (vec, val) = vec.unwrap().pop();
    assert_eq!(vec, None);
    assert_eq!(val, 123);
  }
}
