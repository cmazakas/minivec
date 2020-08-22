use crate::MiniVec;

use std::{
    iter::{Extend, IntoIterator},
    marker::Copy,
};

impl<'a, T> Extend<&'a T> for MiniVec<T>
where
    T: 'a + Copy,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = &'a T>,
    {
        for &x in iter {
            self.push(x);
        }
    }
}

impl<T> Extend<T> for MiniVec<T> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        for x in iter {
            self.push(x);
        }
    }
}
