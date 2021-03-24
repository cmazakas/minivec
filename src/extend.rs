use crate::MiniVec;

impl<'a, T> core::iter::Extend<&'a T> for MiniVec<T>
where
  T: 'a + core::marker::Copy,
{
  fn extend<I>(&mut self, iter: I)
  where
    I: core::iter::IntoIterator<Item = &'a T>,
  {
    for &x in iter {
      self.push(x);
    }
  }
}

impl<T> core::iter::Extend<T> for MiniVec<T> {
  fn extend<I>(&mut self, iter: I)
  where
    I: core::iter::IntoIterator<Item = T>,
  {
    for x in iter {
      self.push(x);
    }
  }
}
