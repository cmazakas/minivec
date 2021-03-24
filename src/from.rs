use crate::MiniVec;

extern crate alloc;

impl<'a, T> core::convert::From<&'a [T]> for MiniVec<T>
where
  T: Clone,
{
  fn from(s: &'a [T]) -> Self {
    let mut v = MiniVec::with_capacity(s.len());
    for x in s {
      v.push(x.clone());
    }

    v
  }
}

impl<'a, T> core::convert::From<&'a mut [T]> for MiniVec<T>
where
  T: Clone,
{
  fn from(s: &'a mut [T]) -> Self {
    let mut v = MiniVec::with_capacity(s.len());
    for x in s {
      v.push(x.clone());
    }

    v
  }
}

impl<'a> core::convert::From<&'a str> for MiniVec<u8> {
  fn from(s: &'a str) -> Self {
    let mut v = MiniVec::with_capacity(s.len());
    unsafe {
      let new_len = s.len();
      core::ptr::copy_nonoverlapping(s.as_ptr(), v.as_mut_ptr(), new_len);
      v.set_len(new_len);
    }
    v
  }
}

impl<'a, T> core::convert::From<&'a MiniVec<T>> for alloc::borrow::Cow<'a, [T]>
where
  T: Clone,
{
  fn from(v: &'a MiniVec<T>) -> alloc::borrow::Cow<'a, [T]> {
    alloc::borrow::Cow::Borrowed(v.as_slice())
  }
}
