use crate::MiniVec;

impl<T> AsRef<[T]> for MiniVec<T> {
    fn as_ref(&self) -> &[T] {
        &*self
    }
}
