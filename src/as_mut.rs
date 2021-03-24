use crate::MiniVec;

impl<T> core::convert::AsMut<[T]> for MiniVec<T> {
  fn as_mut(&mut self) -> &mut [T] {
    &mut *self
  }
}

impl<T> core::convert::AsMut<MiniVec<T>> for MiniVec<T> {
  fn as_mut(&mut self) -> &mut MiniVec<T> {
    self
  }
}
