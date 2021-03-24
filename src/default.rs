use crate::MiniVec;

impl<T> Default for MiniVec<T> {
  fn default() -> Self {
    Self::new()
  }
}
