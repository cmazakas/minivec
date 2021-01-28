use crate::MiniVec;

impl<T> core::borrow::Borrow<[T]> for MiniVec<T> {
    fn borrow(&self) -> &[T] {
        &(self[..])
    }
}

impl<T> core::borrow::BorrowMut<[T]> for MiniVec<T> {
    fn borrow_mut(&mut self) -> &mut [T] {
        &mut (self[..])
    }
}
