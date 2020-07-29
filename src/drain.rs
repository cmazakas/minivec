use crate::MiniVec;

use std::ptr;

pub struct Drain<'a, T> {
    vec_: &'a MiniVec<T>,
    drain_pos_: ptr::NonNull<T>,
    drain_end_: ptr::NonNull<T>,
    len_: usize,
}

impl<T> Iterator for Drain<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.drain_pos_ == self.drain_end_ {
            return None;
        }

        let p = self.drain_pos_.as_ptr();
        let tmp = unsafe { ptr::read(p) };
        self.drain_pos_ = unsafe { ptr::NonNull::new_unchecked(p.add(1)) };
        Some(tmp)
    }
}

impl<T> Drop for Drain<'_, T> {
    fn drop(&mut self) {
        for x in self {
            drop(x);
        }
    }
}
