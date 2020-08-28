#![allow(dead_code)]

use std::{
    alloc,
    cmp::Ordering,
    marker::PhantomData,
    mem,
    ops::{FnMut, RangeBounds},
    ptr, slice,
};

mod r#impl;

pub mod as_mut;
pub mod as_ref;
pub mod borrow;
pub mod clone;
pub mod debug;
pub mod default;
pub mod deref;
pub mod drop;
pub mod eq;
pub mod extend;
pub mod from;
pub mod from_iterator;
pub mod hash;
pub mod index;
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

    pub fn drain<R>(&mut self, range: R) -> Drain<T>
    where
        R: RangeBounds<usize>,
    {
        let len = self.len();

        let start_idx = match range.start_bound() {
            std::ops::Bound::Included(&n) => n,
            std::ops::Bound::Excluded(&n) => n + 1,
            std::ops::Bound::Unbounded => 0,
        };

        let end_idx = match range.end_bound() {
            std::ops::Bound::Included(&n) => n + 1,
            std::ops::Bound::Excluded(&n) => n,
            std::ops::Bound::Unbounded => len,
        };

        if start_idx > end_idx {
            panic!(
                "start drain index (is {}) should be <= end drain index (is {})",
                start_idx, end_idx
            );
        }

        if end_idx > len {
            panic!(
                "end drain index (is {}) should be <= len (is {})",
                end_idx, len
            );
        }

        let data = self.as_mut_ptr();

        unsafe { self.set_len(start_idx) };

        Drain {
            vec_: ptr::NonNull::from(self),
            drain_pos_: unsafe { ptr::NonNull::new_unchecked(data.add(start_idx)) },
            drain_end_: unsafe { ptr::NonNull::new_unchecked(data.add(end_idx)) },
            remaining_pos_: unsafe { ptr::NonNull::new_unchecked(data.add(end_idx)) },
            remaining_: len - end_idx,
            marker_: std::marker::PhantomData,
        }
    }

    /// # Safety
    ///
    /// from_raw_part is incredibly unsafe and can only be used with the value of `MiniVec::as_mut_ptr`
    /// This function takes the previous result of a `MiniVec::as_mut_ptr` call and recreates a new `MiniVec` from it
    ///
    #[allow(clippy::cast_ptr_alignment)]
    pub unsafe fn from_raw_part(ptr: *mut T) -> MiniVec<T> {
        let header_size = mem::size_of::<Header<T>>();
        let aligned = next_aligned(header_size, mem::align_of::<T>());

        let p = ptr as *mut u8;
        let buf = p.sub(aligned);

        debug_assert!((*(buf as *mut Header<T>)).data_ == ptr);

        MiniVec {
            buf_: buf,
            phantom_: PhantomData,
        }
    }

    /// # Safety
    ///
    /// from_raw_parts is incredibly unsafe and can only be used with the value of `MiniVec::as_mut_ptr`
    ///
    /// The length and capacity parameters aren't explicitly needed for this function to work as our internal
    /// implementation stores these in the same allocation we use for the actual [T]
    /// They are used in debug mode to assert the correct parameters and are kept for API compatibility with the
    /// existing Vec signature
    ///
    #[allow(clippy::cast_ptr_alignment)]
    pub unsafe fn from_raw_parts(ptr: *mut T, length: usize, capacity: usize) -> MiniVec<T> {
        let header_size = mem::size_of::<Header<T>>();
        let aligned = next_aligned(header_size, mem::align_of::<T>());

        let p = ptr as *mut u8;
        let buf = p.sub(aligned);

        debug_assert!((*(buf as *mut Header<T>)).len_ == length);
        debug_assert!((*(buf as *mut Header<T>)).cap_ == capacity);
        debug_assert!((*(buf as *mut Header<T>)).data_ == ptr);

        MiniVec {
            buf_: buf,
            phantom_: PhantomData,
        }
    }

    pub fn insert(&mut self, index: usize, element: T) {
        let len = self.len();

        if index > len {
            panic!(
                "insertion index (is {}) should be <= len (is {})",
                index, len
            );
        }

        if len == self.capacity() {
            self.reserve(1);
        }

        let p = unsafe { self.as_mut_ptr().add(index) };
        unsafe {
            ptr::copy(p, p.add(1), len - index);
            ptr::write(p, element);
            self.set_len(len + 1);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn leak<'a>(vec: MiniVec<T>) -> &'a mut [T]
    where
        T: 'a,
    {
        let len = vec.len();
        let mut vec = mem::ManuallyDrop::new(vec);
        let vec: &mut MiniVec<T> = &mut *vec;
        unsafe { slice::from_raw_parts_mut(vec.as_mut_ptr(), len) }
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

    pub fn pop(&mut self) -> Option<T> {
        let len = self.len();

        if len == 0 {
            return None;
        }

        let v = unsafe { ptr::read(self.as_ptr().add(len - 1)) };
        unsafe { self.set_len(len - 1) };
        Some(v)
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

    pub fn remove(&mut self, index: usize) -> T {
        let len = self.len();
        if index >= len {
            panic!("removal index (is {}) should be < len (is {})", index, len);
        }

        unsafe {
            let p = self.as_mut_ptr().add(index);

            let x = ptr::read(p);

            let src = p.add(1);
            let dst = p;
            let count = len - index - 1;
            ptr::copy(src, dst, count);

            self.set_len(len - 1);

            x
        }
    }

    pub fn remove_item<V>(&mut self, item: &V) -> Option<T>
    where
        T: PartialEq<V>,
    {
        let len = self.len();
        for i in 0..len {
            if self[i] == *item {
                return Some(self.remove(i));
            }
        }
        None
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

    pub fn resize(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
        let len = self.len();
        match new_len.cmp(&len) {
            Ordering::Equal => {}
            Ordering::Greater => {
                let num_elems = new_len - len;
                self.reserve(num_elems);
                for _i in 0..num_elems {
                    self.push(value.clone());
                }
            }
            Ordering::Less => {
                self.truncate(new_len);
            }
        }
    }

    pub fn resize_with<F>(&mut self, new_len: usize, mut f: F)
    where
        F: FnMut() -> T,
    {
        let len = self.len();
        match new_len.cmp(&len) {
            Ordering::Equal => {}
            Ordering::Greater => {
                let num_elems = new_len - len;
                self.reserve(num_elems);
                for _i in 0..num_elems {
                    self.push(f());
                }
            }
            Ordering::Less => {
                self.truncate(new_len);
            }
        }
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        let len = self.len();

        let data = self.as_mut_ptr();

        let mut read = data;
        let mut write = read;

        let last = unsafe { data.add(len) };

        while read < last {
            let should_retain = unsafe { f(&mut *read) };
            if should_retain {
                if read != write {
                    unsafe { mem::swap(&mut *read, &mut *write) };
                }
                write = unsafe { write.add(1) };
            }

            read = unsafe { read.add(1) };
        }

        self.truncate((write as usize - data as usize) / mem::size_of::<T>());
    }

    /// # Safety
    ///
    /// This function is unsafe in the sense that it will NOT call `.drop()` on the elements excluded from the new len
    ///
    pub unsafe fn set_len(&mut self, len: usize) {
        self.header_mut().len_ = len;
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        let (len, capacity) = (self.len(), self.capacity());

        if min_capacity < len {
            self.shrink_to_fit();
            return;
        }

        if capacity == min_capacity {
            return;
        }

        if capacity < min_capacity {
            panic!("Tried to shrink to a larger capacity");
        }

        self.grow(min_capacity);
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

    pub fn with_capacity(capacity: usize) -> MiniVec<T> {
        let mut v = MiniVec::new();
        v.reserve_exact(capacity);
        v
    }
}

impl<T: Clone> MiniVec<T> {
    pub fn extend_from_slice(&mut self, elems: &[T]) {
        self.reserve(elems.len());
        for x in elems {
            self.push((*x).clone());
        }
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

pub struct Drain<'a, T: 'a> {
    vec_: ptr::NonNull<MiniVec<T>>,
    drain_pos_: ptr::NonNull<T>,
    drain_end_: ptr::NonNull<T>,
    remaining_pos_: ptr::NonNull<T>,
    remaining_: usize,
    marker_: std::marker::PhantomData<&'a T>,
}

impl<T> Iterator for Drain<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.drain_pos_ >= self.drain_end_ {
            return None;
        }

        let p = self.drain_pos_.as_ptr();
        let tmp = unsafe { ptr::read(p) };
        self.drain_pos_ = unsafe { ptr::NonNull::new_unchecked(p.add(1)) };
        Some(tmp)
    }
}

impl<T> DoubleEndedIterator for Drain<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let pos = unsafe { self.drain_end_.as_ptr().sub(1) };
        if pos < self.drain_pos_.as_ptr() {
            return None;
        }

        let tmp = unsafe { ptr::read(pos) };
        self.drain_end_ = unsafe { ptr::NonNull::new_unchecked(pos) };
        Some(tmp)
    }
}

impl<T> Drop for Drain<'_, T> {
    fn drop(&mut self) {
        struct DropGuard<'b, 'a, T> {
            drain: &'b mut Drain<'a, T>,
        };

        impl<'b, 'a, T> Drop for DropGuard<'b, 'a, T> {
            fn drop(&mut self) {
                while let Some(_) = self.drain.next() {}

                if self.drain.remaining_ > 0 {
                    let v = unsafe { self.drain.vec_.as_mut() };
                    let v_len = v.len();

                    let src = self.drain.remaining_pos_.as_ptr();
                    let dst = unsafe { v.as_mut_ptr().add(v_len) };

                    if src == dst {
                        return;
                    }

                    unsafe {
                        ptr::copy(src, dst, self.drain.remaining_);
                        v.set_len(v_len + self.drain.remaining_);
                    };
                }
            }
        }

        while let Some(item) = self.next() {
            let guard = DropGuard { drain: self };
            drop(item);
            mem::forget(guard);
        }

        DropGuard { drain: self };
    }
}
