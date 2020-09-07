use crate::MiniVec;

use core::fmt::{Debug, Formatter, Result};

impl<T: Debug> Debug for MiniVec<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let this: &[T] = &*self;

        this.fmt(f)
    }
}
