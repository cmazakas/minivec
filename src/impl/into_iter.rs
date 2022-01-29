extern crate alloc;

// we diverge pretty heavily from the stdlib here
//
// we're able to pretty much hack MiniVec into being an IntoIter type simply by
// making it a data member of the struct and then manually adjusting things in
// the Header of the MiniVec
//

/// `IntoIter` is an iterator type that consumes the `MiniVec` and transfers ownership of the contained elements to the
/// caller when iterated.
///
pub struct IntoIter<T> {
  pub(crate) v: crate::MiniVec<T>,
  pub(crate) pos: *const T,
  marker: core::marker::PhantomData<T>,
}

impl<T> IntoIter<T> {
  #[must_use]
  pub(crate) fn new(w: crate::MiniVec<T>) -> Self {
    let v = w;
    let pos = v.data();

    Self {
      v,
      pos,
      marker: core::marker::PhantomData,
    }
  }

  /// `as_slice` returns an immutable slice to the remaining elements of the iterator that have not yet been moved.
  ///
  #[must_use]
  pub fn as_slice(&self) -> &[T] {
    let data = self.pos;
    unsafe { core::slice::from_raw_parts(data, self.v.len()) }
  }

  /// `as_mut_slice` returns a mutable slice to the remaining elements of the iterator that have not yet been moved.
  ///
  pub fn as_mut_slice(&mut self) -> &mut [T] {
    let data: *mut T = self.pos as *mut T;
    unsafe { core::slice::from_raw_parts_mut(data, self.v.len()) }
  }
}

impl<T> AsRef<[T]> for IntoIter<T> {
  fn as_ref(&self) -> &[T] {
    self.as_slice()
  }
}

impl<T: Clone> Clone for IntoIter<T> {
  fn clone(&self) -> IntoIter<T> {
    let w = self.v.clone();
    let pos_cpy = self.pos;
    IntoIter {
      v: w,
      pos: pos_cpy,
      marker: core::marker::PhantomData,
    }
  }
}

impl<T: alloc::fmt::Debug> alloc::fmt::Debug for IntoIter<T> {
  fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
    f.debug_tuple("MiniVec::IntoIter")
      .field(&self.as_slice())
      .finish()
  }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
  fn next_back(&mut self) -> Option<Self::Item> {
    let header = self.v.header_mut();

    let data = self.pos;
    let count = header.len;
    let end = unsafe { data.add(count) };

    if data >= end {
      return None;
    };

    header.len -= 1;

    Some(unsafe { core::ptr::read(data.add(header.len)) })
  }
}

impl<T> Drop for IntoIter<T> {
  fn drop(&mut self) {
    for v in self {
      core::mem::drop(v);
    }
  }
}

impl<T> ExactSizeIterator for IntoIter<T> {
  fn len(&self) -> usize {
    self.v.len()
  }

  // fn is_empty(&self) -> bool {
  //     self.v.is_empty()
  // }
}

impl<T> core::iter::FusedIterator for IntoIter<T> {}

impl<T> Iterator for IntoIter<T> {
  type Item = T;

  fn next(&mut self) -> Option<Self::Item> {
    let header = self.v.header_mut();

    let data = self.pos;
    let count = header.len;
    let end = unsafe { data.add(count) };

    if data >= end {
      return None;
    }

    self.pos = unsafe { data.add(1) };
    header.len -= 1;

    Some(unsafe { core::ptr::read(data) })
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    let len = self.v.len();
    (len, Some(len))
  }
}

unsafe impl<T: Send> Send for IntoIter<T> {}
unsafe impl<T: Sync> Sync for IntoIter<T> {}

// unsafe impl<T> core::iter::InPlaceIterable for IntoIter<T> {}
