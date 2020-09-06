use crate::MiniVec;

extern crate alloc;
extern crate core;

use alloc::borrow::Cow;
use core::{convert::From, ptr};

impl<'a, T> From<&'a [T]> for MiniVec<T>
where
    T: Clone,
{
    fn from(s: &'a [T]) -> Self {
        let mut v = MiniVec::with_capacity(s.len());
        for x in s {
            v.push(x.clone());
        }

        v
    }
}

impl<'a, T> From<&'a mut [T]> for MiniVec<T>
where
    T: Clone,
{
    fn from(s: &'a mut [T]) -> Self {
        let mut v = MiniVec::with_capacity(s.len());
        for x in s {
            v.push(x.clone());
        }

        v
    }
}

impl<'a> From<&'a str> for MiniVec<u8> {
    fn from(s: &'a str) -> Self {
        let mut v = MiniVec::with_capacity(s.len());
        unsafe {
            let new_len = s.len();
            ptr::copy_nonoverlapping(s.as_ptr(), v.as_mut_ptr(), new_len);
            v.set_len(new_len);
        }
        v
    }
}

impl<'a, T> From<&'a MiniVec<T>> for Cow<'a, [T]>
where
    T: Clone,
{
    fn from(v: &'a MiniVec<T>) -> Cow<'a, [T]> {
        Cow::Borrowed(v.as_slice())
    }
}
