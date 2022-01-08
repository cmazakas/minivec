use crate::make_layout;
use crate::Header;
use crate::MiniVec;

extern crate alloc;

// TODO: someday update this impl to be:
// unsafe impl<#[may_dangle] T> for MiniVec<T>
//
// so that tests will pass for `test_vec_cycle`
//

impl<T> Drop for MiniVec<T> {
  fn drop(&mut self) {
    unsafe {
      #[allow(clippy::cast_ptr_alignment)]
      let Header { len, cap } = core::ptr::read(self.buf.as_ptr().cast::<Header>());

      core::ptr::drop_in_place(core::ptr::slice_from_raw_parts_mut(self.data(), len));
      alloc::alloc::dealloc(self.buf.as_ptr(), make_layout::<T>(cap));
    };
  }
}
