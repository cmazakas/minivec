#![allow(dead_code)]

use std::{alloc, marker::PhantomData, mem, ptr, slice};

mod r#impl;

pub mod as_ref;
pub mod clone;
pub mod debug;
pub mod default;
pub mod deref;
pub mod drain;
pub mod drop;
pub mod partial_eq;

use crate::r#impl::helpers::*;

struct Header<T> {
    data_: *mut T,
    len_: usize,
    cap_: usize,
}

pub struct MiniVec<T> {
    buf_: *mut u8,
    phantom_: PhantomData<T>,
}

impl<T> MiniVec<T> {
    fn header(&self) -> &Header<T> {
        #[allow(clippy::cast_ptr_alignment)]
        unsafe {
            &*(self.buf_ as *const Header<T>)
        }
    }

    fn header_mut(&mut self) -> &mut Header<T> {
        #[allow(clippy::cast_ptr_alignment)]
        unsafe {
            &mut *(self.buf_ as *mut Header<T>)
        }
    }

    fn grow(&mut self, capacity: usize) {
        debug_assert!(capacity >= self.len());

        let new_capacity = capacity;
        let new_layout = make_layout::<T>(new_capacity);
        let len = self.len();

        let new_buf = unsafe { alloc::alloc(new_layout) };
        if new_buf.is_null() {
            alloc::handle_alloc_error(new_layout);
        }

        let new_data = unsafe {
            let offset = next_aligned(mem::size_of::<Header<T>>(), mem::align_of::<T>());
            new_buf.add(offset) as *mut T
        };

        let header = Header::<T> {
            data_: new_data,
            len_: len,
            cap_: new_capacity,
        };

        #[allow(clippy::cast_ptr_alignment)]
        unsafe {
            ptr::write(new_buf as *mut Header<T>, header)
        };

        if !self.buf_.is_null() {
            #[allow(clippy::cast_ptr_alignment)]
            let old_header = unsafe { ptr::read(self.buf_ as *mut Header<T>) };

            let old_layout = make_layout::<T>(old_header.cap_);

            if len > 0 {
                unsafe { ptr::copy_nonoverlapping(old_header.data_, new_data, len) };
            }

            unsafe {
                alloc::dealloc(self.buf_, old_layout);
            };
        }

        self.buf_ = new_buf;
    }

    pub fn append(&mut self, other: &mut MiniVec<T>) {
        if other.is_empty() {
            return;
        }

        let other_len = other.len();
        self.reserve(other_len);

        unsafe {
            ptr::copy_nonoverlapping(other.as_ptr(), self.as_mut_ptr().add(self.len()), other_len);
        };

        other.header_mut().len_ = 0;
        self.header_mut().len_ += other_len;
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        if self.buf_.is_null() {
            return ptr::null_mut();
        }

        self.header_mut().data_
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self
    }

    pub fn as_ptr(&self) -> *const T {
        if self.buf_.is_null() {
            return ptr::null();
        }

        self.header().data_
    }

    pub fn as_slice(&self) -> &[T] {
        self
    }

    pub fn capacity(&self) -> usize {
        if self.buf_.is_null() {
            0
        } else {
            self.header().cap_
        }
    }

    pub fn clear(&mut self) {
        self.truncate(0);
    }

    pub fn dedup(&mut self)
    where
        T: PartialEq,
    {
        self.dedup_by(|x, y| x == y);
    }

    // basically just copy what's here:
    // https://github.com/llvm/llvm-project/blob/032810f58986cd568980227c9531de91d8bcb1cd/libcxx/include/algorithm#L2174-L2191
    //
    pub fn dedup_by<F>(&mut self, mut pred: F)
    where
        F: FnMut(&mut T, &mut T) -> bool,
    {
        let len = self.len();
        if len < 2 {
            return;
        }

        let data = self.as_mut_ptr();

        let mut read = unsafe { data.add(1) };
        let mut write = read;

        let last = unsafe { data.add(len) };

        while read < last {
            let matches = unsafe { pred(&mut *read, &mut *write.sub(1)) };
            if !matches {
                if read != write {
                    unsafe { mem::swap(&mut *read, &mut *write) };
                }
                write = unsafe { write.add(1) };
            }

            read = unsafe { read.add(1) };
        }

        self.truncate((write as usize - data as usize) / mem::size_of::<T>());
    }

    pub fn dedup_by_key<F, K>(&mut self, mut key: F)
    where
        F: FnMut(&mut T) -> K,
        K: PartialEq<K>,
    {
        self.dedup_by(|a, b| key(a) == key(b));
    }

    // pub fn drain<R>(&mut self, range: R) -> Drain<T>
    // where
    //     R: RangeBounds<usize>,
    // {
    // }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        if self.buf_.is_null() {
            0
        } else {
            self.header().len_
        }
    }

    pub fn new() -> MiniVec<T> {
        assert!(mem::size_of::<T>() > 0, "ZSTs currently not supported");

        MiniVec {
            buf_: ptr::null_mut(),
            phantom_: std::marker::PhantomData,
        }
    }

    pub fn push(&mut self, value: T) {
        let (len, capacity) = (self.len(), self.capacity());
        if len == capacity {
            self.grow(next_capacity::<T>(capacity));
        }

        let len = self.len();
        let mut header = self.header_mut();

        let data = header.data_;
        let dst = unsafe { data.add(len) };

        unsafe {
            ptr::write(dst, value);
        };

        header.len_ += 1;
    }

    pub fn reserve(&mut self, additional: usize) {
        let capacity = self.capacity();
        let total_required = self.len() + additional;

        if total_required <= capacity {
            return;
        }

        let mut new_capacity = next_capacity::<T>(capacity);
        while new_capacity < total_required {
            new_capacity = next_capacity::<T>(new_capacity);
        }

        self.grow(new_capacity);
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        let capacity = self.capacity();
        let len = self.len();

        let total_required = len + additional;
        if capacity >= total_required {
            return;
        }

        self.grow(total_required);
    }

    pub fn shrink_to_fit(&mut self) {
        let (len, capacity) = (self.len(), self.capacity());
        if len == capacity {
            return;
        }

        self.grow(len);
    }

    pub fn truncate(&mut self, len: usize) {
        let self_len = self.len();

        if len >= self_len {
            return;
        }

        self.header_mut().len_ = len;

        let s =
            unsafe { slice::from_raw_parts_mut(self.header_mut().data_.add(len), self_len - len) };

        unsafe { ptr::drop_in_place(s) };
    }
}

#[macro_export]
macro_rules! mini_vec {
    () => (
        $crate::MiniVec::new()
    );
    ($($x:expr),+ $(,)?) => {
        {
            let mut tmp = $crate::MiniVec::new();
            $(
                tmp.push($x);
            )*
            tmp
        }
    };
}
