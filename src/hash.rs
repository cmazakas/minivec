use crate::MiniVec;

impl<T> core::hash::Hash for MiniVec<T>
where
  T: core::hash::Hash,
{
  fn hash<H>(&self, state: &mut H)
  where
    H: core::hash::Hasher,
  {
    let this: &[T] = &**self;
    core::hash::Hash::hash(this, state);
  }
}
