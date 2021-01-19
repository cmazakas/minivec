use crate::make_layout;
use crate::Header;
use crate::MiniVec;

extern crate alloc;

use core::ptr;

impl<T> Drop for MiniVec<T> {
    fn drop(&mut self) {
        if self.buf.is_null() {
            return;
        }

        #[allow(clippy::cast_ptr_alignment)]
        let header = unsafe { ptr::read(self.buf as *const Header) };

        for i in 0..header.len {
            unsafe { ptr::read(self.data().add(i)) };
        }

        unsafe { alloc::alloc::dealloc(self.buf, make_layout::<T>(header.cap, header.alignment)) };
    }
}
