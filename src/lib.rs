#![allow(dead_code)]

use std::alloc;
use std::alloc::Layout;
use std::marker::PhantomData;
use std::mem;
use std::ptr;

fn next_aligned(num_bytes: usize, alignment: usize) -> usize {
    let remaining = num_bytes % alignment;
    if remaining == 0 {
        num_bytes
    } else {
        num_bytes + (alignment - remaining)
    }
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
}

struct Header<T> {
    data_: *mut T,
    len_: usize,
    cap_: usize,
}

fn get_layout<T>(cap: usize) -> Layout {
    let header_size = mem::size_of::<Header<T>>();
    let num_bytes = if cap == 0 {
        header_size
    } else {
        next_aligned(header_size, mem::align_of::<T>()) + cap * mem::size_of::<T>()
    };

    Layout::from_size_align(num_bytes, max_align::<T>()).unwrap()
}

struct MiniVec<T> {
    buf_: *mut u8,
    phantom_: PhantomData<T>,
}

fn max_align<T>() -> usize {
    let align_t = mem::align_of::<T>();
    let header_align = mem::align_of::<Header<T>>();
    std::cmp::max(align_t, header_align)
}

impl<T> MiniVec<T> {
    fn header(&self) -> &Header<T> {
        #[allow(clippy::cast_ptr_alignment)]
        let header = unsafe { &*(self.buf_ as *const Header<T>) };
        header
    }

    fn len(&self) -> usize {
        self.header().len_
    }

    fn capacity(&self) -> usize {
        self.header().cap_
    }

    fn new() -> MiniVec<T> {
        assert!(mem::size_of::<T>() > 0, "ZSTs currently not supported");

        let size = mem::size_of::<Header<T>>();
        let layout = Layout::from_size_align(size, max_align::<T>()).unwrap();

        let p = unsafe { alloc::alloc(layout) };

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
}

impl<T> Drop for MiniVec<T> {
    fn drop(&mut self) {
        #[allow(clippy::cast_ptr_alignment)]
        let header = unsafe { ptr::read(self.buf_ as *const Header<T>) };

        let size = next_aligned(mem::size_of::<Header<T>>(), mem::align_of::<T>())
            + header.len_ * mem::size_of::<T>();

        let layout = Layout::from_size_align(size, max_align::<T>()).unwrap();

        unsafe { alloc::dealloc(self.buf_, layout) };
    }
}
