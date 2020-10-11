use crate::make_layout;
use crate::Header;
use crate::MiniVec;

extern crate alloc;

use core::ptr;

impl<T> Drop for MiniVec<T> {
    fn drop(&mut self) {
        if self.buf_.is_null() {
            return;
        }

        #[allow(clippy::cast_ptr_alignment)]
        let header = unsafe { ptr::read(self.buf_ as *const Header) };

        for i in 0..header.len_ {
            unsafe { ptr::read(self.data().add(i)) };
        }

        unsafe { alloc::alloc::dealloc(self.buf_, make_layout::<T>(header.cap_)) };
    }
}
