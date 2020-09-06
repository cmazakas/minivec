use crate::MiniVec;

extern crate alloc;
extern crate core;

use alloc::fmt;
use core::{
    clone::Clone,
    convert::AsRef,
    iter::{DoubleEndedIterator, ExactSizeIterator, FusedIterator, IntoIterator, Iterator},
    marker::{Send, Sync},
    ptr, slice,
};

// we diverge pretty heavily from the stdlib here
//
// we're able to pretty much hack MiniVec into being an IntoIter type simply by
// making it a data member of the struct and then manually adjusting things in
// the Header of the MiniVec
//
// to this end, we even get traits like Drop for free where the stdlib had to
// manually implement the trait for its vec::IntoIter type
//
pub struct IntoIter<T> {
    v: MiniVec<T>,
}

impl<T> IntoIter<T> {
    pub fn new(w: MiniVec<T>) -> Self {
        Self { v: w }
    }

    pub fn as_slice(&self) -> &[T] {
        self.v.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.v.as_mut_slice()
    }
}

impl<T> AsRef<[T]> for IntoIter<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T: Clone> Clone for IntoIter<T> {
    fn clone(&self) -> IntoIter<T> {
        let w = self.v.clone();
        IntoIter { v: w }
    }
}

impl<T: fmt::Debug> fmt::Debug for IntoIter<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("MiniVec::IntoIter")
            .field(&self.as_slice())
            .finish()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let header = self.v.header_mut();

        let data = header.data_;
        let end = unsafe { data.add(header.len_) };

        if data == end {
            return None;
        };

        header.len_ -= 1;

        Some(unsafe { ptr::read(data.add(header.len_)) })
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.v.len()
    }

    // fn is_empty(&self) -> bool {
    //     self.v.is_empty()
    // }
}

impl<T> FusedIterator for IntoIter<T> {}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let header = self.v.header_mut();

        let data = header.data_;
        let end = unsafe { data.add(header.len_) };

        if data == end {
            return None;
        }

        header.data_ = unsafe { data.add(1) };
        header.len_ -= 1;

        Some(unsafe { ptr::read(data) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.v.len();
        (len, Some(len))
    }
}

unsafe impl<T: Send> Send for IntoIter<T> {}
unsafe impl<T: Sync> Sync for IntoIter<T> {}

impl<T> IntoIterator for MiniVec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::<T>::new(self)
    }
}

impl<'a, T> IntoIterator for &'a MiniVec<T> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut MiniVec<T> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> slice::IterMut<'a, T> {
        self.iter_mut()
    }
}
