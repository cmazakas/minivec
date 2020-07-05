#![allow(dead_code)]

use std::alloc;
use std::alloc::Layout;
use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;

struct Header<T> {
    data_: *mut T,
    len_: usize,
    cap_: usize,
}

fn next_aligned(num_bytes: usize, alignment: usize) -> usize {
    let remaining = num_bytes % alignment;
    if remaining == 0 {
        num_bytes
    } else {
        num_bytes + (alignment - remaining)
    }
}

fn next_capacity<T>(capacity: usize) -> usize {
    let elem_size = mem::size_of::<T>();

    if capacity == 0 {
        return match elem_size {
            1 => 8,
            2..=1024 => 4,
            _ => 1,
        };
    }

    2 * capacity
}

fn max_align<T>() -> usize {
    let align_t = mem::align_of::<T>();
    let header_align = mem::align_of::<Header<T>>();
    std::cmp::max(align_t, header_align)
}

fn make_layout<T>(cap: usize) -> Layout {
    let alignment = max_align::<T>();

    let header_size = mem::size_of::<Header<T>>();
    let num_bytes = if cap == 0 {
        header_size
    } else {
        next_aligned(header_size, mem::align_of::<T>()) + cap * mem::size_of::<T>()
    };

    Layout::from_size_align(num_bytes, alignment).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn next_aligned_test() {
        assert_eq!(next_aligned(9, 4), 12);
        assert_eq!(next_aligned(13, 4), 16);
        assert_eq!(next_aligned(12, 4), 12);
        assert_eq!(next_aligned(13, 1), 13);
        assert_eq!(next_aligned(8, 8), 8);
        assert_eq!(next_aligned(16, 32), 32);
        assert_eq!(next_aligned(16, 512), 512);
    }

    #[repr(align(512))]
    struct OverAligned {
        data: [u8; 512],
    }

    #[test]
    fn max_align_test() {
        let header_alignment = mem::align_of::<Header<()>>();

        assert!(mem::align_of::<i32>() <= mem::align_of::<Header<()>>());
        assert_eq!(max_align::<i32>(), header_alignment);

        assert!(mem::align_of::<u8>() <= mem::align_of::<Header<()>>());
        assert_eq!(max_align::<u8>(), header_alignment);

        assert!(mem::align_of::<OverAligned>() > mem::align_of::<Header<()>>());
        assert_eq!(max_align::<OverAligned>(), mem::align_of::<OverAligned>());
    }

    #[test]
    fn make_layout_test() {
        // empty
        //
        let layout = make_layout::<i32>(0);

        assert_eq!(layout.align(), mem::align_of::<Header<i32>>());
        assert_eq!(layout.size(), mem::size_of::<Header<i32>>());

        // non-empty, less than
        //
        let layout = make_layout::<i32>(512);
        assert!(mem::align_of::<i32>() < mem::align_of::<Header<i32>>());
        assert_eq!(layout.align(), mem::align_of::<Header<i32>>());
        assert_eq!(
            layout.size(),
            mem::size_of::<Header<i32>>() + 512 * mem::size_of::<i32>()
        );

        // non-empty, equal
        //
        let layout = make_layout::<i64>(512);
        assert_eq!(mem::align_of::<i64>(), mem::align_of::<Header<i64>>());
        assert_eq!(layout.align(), mem::align_of::<Header<i64>>());
        assert_eq!(
            layout.size(),
            mem::size_of::<Header<i64>>() + 512 * mem::size_of::<i64>()
        );

        // non-empty, greater
        let layout = make_layout::<OverAligned>(512);
        assert!(mem::align_of::<OverAligned>() > mem::align_of::<Header<OverAligned>>());
        assert_eq!(layout.align(), mem::align_of::<OverAligned>());
        assert_eq!(
            layout.size(),
            next_aligned(
                mem::size_of::<Header<OverAligned>>(),
                mem::align_of::<OverAligned>()
            ) + 512 * mem::size_of::<OverAligned>()
        );
    }
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
        #[allow(clippy::cast_ptr_alignment)]
        let old_header = unsafe { ptr::read(self.buf_ as *mut Header<T>) };
        let old_capacity = old_header.cap_;
        let old_layout = make_layout::<T>(old_capacity);

        let new_capacity = capacity;
        let new_layout = make_layout::<T>(new_capacity);

        let new_buf = unsafe { alloc::alloc(new_layout) };
        if new_buf.is_null() {
            alloc::handle_alloc_error(new_layout);
        }

        let data = unsafe {
            new_buf.add(next_aligned(
                mem::size_of::<Header<T>>(),
                mem::align_of::<T>(),
            )) as *mut T
        };

        let header = Header::<T> {
            data_: data,
            len_: old_header.len_,
            cap_: new_capacity,
        };

        #[allow(clippy::cast_ptr_alignment)]
        unsafe {
            ptr::write(new_buf as *mut Header<T>, header)
        };

        if old_header.len_ > 0 {
            unsafe { ptr::copy_nonoverlapping(old_header.data_, data, old_header.len_) };
        }

        unsafe {
            alloc::dealloc(self.buf_, old_layout);
        };

        self.buf_ = new_buf;
    }

    pub fn len(&self) -> usize {
        self.header().len_
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn capacity(&self) -> usize {
        self.header().cap_
    }

    pub fn new() -> MiniVec<T> {
        assert!(mem::size_of::<T>() > 0, "ZSTs currently not supported");

        let layout = make_layout::<T>(0);

        let p = unsafe { alloc::alloc(layout) };
        if p.is_null() {
            alloc::handle_alloc_error(layout);
        }

        let header = Header::<T> {
            data_: ptr::null_mut::<T>(),
            len_: 0,
            cap_: 0,
        };

        debug_assert_eq!(p.align_offset(mem::align_of::<Header<T>>()), 0);

        #[allow(clippy::cast_ptr_alignment)]
        unsafe {
            ptr::write(p as *mut Header<T>, header)
        };

        MiniVec {
            buf_: p,
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
        loop {
            let capacity = self.capacity();
            let total_required = self.len() + additional;
            if total_required <= capacity {
                return;
            }

            self.grow(next_capacity::<T>(capacity));
        }
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
}

impl<T> Drop for MiniVec<T> {
    fn drop(&mut self) {
        #[allow(clippy::cast_ptr_alignment)]
        let header = unsafe { ptr::read(self.buf_ as *const Header<T>) };

        for i in 0..header.len_ {
            unsafe { ptr::read(header.data_.add(i)) };
        }

        let layout = make_layout::<T>(header.cap_);
        unsafe { alloc::dealloc(self.buf_, layout) };
    }
}

impl<T> Default for MiniVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for MiniVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        let header = self.header();
        let data = header.data_;
        let len = header.len_;
        unsafe { std::slice::from_raw_parts(data, len) }
    }
}

impl<T> DerefMut for MiniVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let header = self.header();
        let data = header.data_;
        let len = header.len_;
        unsafe { std::slice::from_raw_parts_mut(data, len) }
    }
}

impl<T: Clone> Clone for MiniVec<T> {
    fn clone(&self) -> Self {
        let mut copy = MiniVec::<T>::new();

        copy.reserve(self.len());
        for i in 0..self.len() {
            copy.push(self[i].clone());
        }

        copy
    }
}

impl<T, V> PartialEq<V> for MiniVec<T>
where
    V: std::convert::AsRef<[T]>,
    T: PartialEq,
{
    fn eq(&self, other: &V) -> bool {
        &self[..] == AsRef::<[T]>::as_ref(other)
    }

    fn ne(&self, other: &V) -> bool {
        &self[..] != AsRef::<[T]>::as_ref(other)
    }
}

impl<T: fmt::Debug> fmt::Debug for MiniVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (&&*self).fmt(f)
    }
}

impl<T> AsRef<[T]> for MiniVec<T> {
    fn as_ref(&self) -> &[T] {
        &*self
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
