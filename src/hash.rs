use crate::MiniVec;

extern crate core;

use core::hash::{Hash, Hasher};

impl<T> Hash for MiniVec<T>
where
    T: Hash,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        let this: &[T] = &**self;
        Hash::hash(this, state);
    }
}
