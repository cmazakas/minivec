use crate::MiniVec;

extern crate core;

use core::{
    ops::{Deref, DerefMut},
    slice,
};

impl<T> Deref for MiniVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        if self.buf_.is_null() {
            return &[];
        }

        let header = self.header();
        let data = header.data_;
        let len = header.len_;
        unsafe { slice::from_raw_parts(data, len) }
    }
}

impl<T> DerefMut for MiniVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if self.buf_.is_null() {
            return &mut [];
        }

        let header = self.header();
        let data = header.data_;
        let len = header.len_;
        unsafe { slice::from_raw_parts_mut(data, len) }
    }
}
