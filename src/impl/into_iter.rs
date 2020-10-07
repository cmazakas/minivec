use crate::MiniVec;

extern crate alloc;

use alloc::fmt;
use core::{
    clone::Clone,
    convert::AsRef,
    iter::{DoubleEndedIterator, ExactSizeIterator, FusedIterator, Iterator},
    marker::{Send, Sync},
    ptr, slice,
};

// we diverge pretty heavily from the stdlib here
//
// we're able to pretty much hack MiniVec into being an IntoIter type simply by
// making it a data member of the struct and then manually adjusting things in
// the Header of the MiniVec
//
pub struct IntoIter<T> {
    v: MiniVec<T>,
    pos: *mut T,
}

impl<T> IntoIter<T> {
    #[must_use]
    pub fn new(w: MiniVec<T>) -> Self {
        let pos_cpy = w.data();
        Self { v: w, pos: pos_cpy }
    }

    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.pos, self.v.len()) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.pos, self.v.len()) }
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
        let pos_cpy = self.pos;
        IntoIter { v: w, pos: pos_cpy }
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
        if self.v.buf_.is_null() {
            return None;
        }

        let header = self.v.header_mut();

        let data = self.pos;
        let end = unsafe { data.add(header.len_) };

        if data == end {
            return None;
        };

        header.len_ -= 1;

        Some(unsafe { ptr::read(data.add(header.len_)) })
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        for _ in self {}
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
        if self.v.buf_.is_null() {
            return None;
        }

        let header = self.v.header_mut();

        let data = self.pos;
        let end = unsafe { data.add(header.len_) };

        if data == end {
            return None;
        }

        self.pos = unsafe { data.add(1) };
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
