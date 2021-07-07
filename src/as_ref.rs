use crate::MiniVec;

impl<T> AsRef<[T]> for MiniVec<T> {
  fn as_ref(&self) -> &[T] {
    self
  }
}

impl<T> AsRef<MiniVec<T>> for MiniVec<T> {
  fn as_ref(&self) -> &MiniVec<T> {
    self
  }
}
