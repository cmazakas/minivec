#![allow(dead_code)]

use std::alloc;
use std::alloc::Layout;
use std::marker::PhantomData;
use std::mem;
use std::ptr;

struct Header<T> {
    data_: *mut T,
    len_: usize,
    cap_: usize,
}

struct MiniVec<T> {
    buf_: *mut u8,
    phantom_: PhantomData<T>,
}

fn max_align<T>() -> usize {
    let align_t = std::mem::align_of::<T>();
    let header_align = std::mem::align_of::<Header<T>>();
    std::cmp::max(align_t, header_align)
}

fn next_aligned(num_bytes: usize, alignment: usize) -> usize {
    let remaining = num_bytes % alignment;
    if remaining == 0 {
        num_bytes
    } else {
        num_bytes + (alignment - remaining)
    }
}

impl<T> MiniVec<T> {
    fn new() -> MiniVec<T> {
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

fn main() {
    assert_eq!(mem::size_of::<MiniVec<i64>>(), mem::size_of::<*const ()>());

    let _: MiniVec<i64> = MiniVec::new();
}
