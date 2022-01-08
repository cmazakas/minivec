use crate::MiniVec;

impl<T> core::ops::Deref for MiniVec<T> {
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    let header = self.header();
    let data = self.data();
    let len = header.len;
    unsafe { core::slice::from_raw_parts(data, len) }
  }
}

impl<T> core::ops::DerefMut for MiniVec<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    let header = self.header();
    let data = self.data();
    let len = header.len;
    unsafe { core::slice::from_raw_parts_mut(data, len) }
  }
}
