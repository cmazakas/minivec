use crate::MiniVec;

impl<T: Clone> Clone for MiniVec<T> {
  fn clone(&self) -> Self {
    if self.is_default() {
      return MiniVec::new();
    }

    let mut copy = MiniVec::<T>::new();

    copy.reserve(self.len());
    for i in 0..self.len() {
      copy.push(self[i].clone());
    }

    copy
  }
}
