use crate::MiniVec;

use core::cmp::{Ord, Ordering};

impl<T: Ord> Ord for MiniVec<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        let x: &[T] = &**self;
        let y: &[T] = &**other;

        x.cmp(y)
    }
}
