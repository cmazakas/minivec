use crate::MiniVec;

use core::convert::AsMut;

impl<T> AsMut<[T]> for MiniVec<T> {
    fn as_mut(&mut self) -> &mut [T] {
        &mut *self
    }
}

impl<T> AsMut<MiniVec<T>> for MiniVec<T> {
    fn as_mut(&mut self) -> &mut MiniVec<T> {
        self
    }
}
