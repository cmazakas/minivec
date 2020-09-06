use crate::MiniVec;

extern crate core;

use core::borrow::{Borrow, BorrowMut};

impl<T> Borrow<[T]> for MiniVec<T> {
    fn borrow(&self) -> &[T] {
        &(self[..])
    }
}

impl<T> BorrowMut<[T]> for MiniVec<T> {
    fn borrow_mut(&mut self) -> &mut [T] {
        &mut (self[..])
    }
}
