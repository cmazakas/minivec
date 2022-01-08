use crate::MiniVec;

#[cfg(feature = "minivec_nightly")]
impl<T: Clone> Clone for MiniVec<T> {
  default fn clone(&self) -> Self {
    struct DropGuard<'a, T> {
      vec: &'a mut MiniVec<T>,
      len: usize,
    }

    impl<'a, T> Drop for DropGuard<'a, T> {
      fn drop(&mut self) {
        unsafe { self.vec.set_len(self.len) };
      }
    }

    impl<'a, T: Clone> DropGuard<'a, T> {
      fn init(&mut self, xs: &[T]) {
        let len = &mut self.len;
        let vec = &mut self.vec;

        xs.iter()
          .zip(vec.spare_capacity_mut().iter_mut())
          .for_each(|(v, p)| {
            *p = core::mem::MaybeUninit::new(v.clone());
            *len += 1;
          });
      }
    }

    if self.is_empty() {
      return MiniVec::<T>::new();
    }

    let len = self.len();
    let mut cpy = MiniVec::<T>::with_capacity(len);

    if !core::mem::needs_drop::<T>() {
      self
        .as_slice()
        .iter()
        .zip(cpy.spare_capacity_mut().iter_mut())
        .for_each(|(v, p)| {
          *p = core::mem::MaybeUninit::new(v.clone());
        });

      unsafe { cpy.set_len(len) };
    } else {
      let mut guard = DropGuard {
        vec: &mut cpy,
        len: 0,
      };

      guard.init(self.as_slice());
    }

    cpy
  }
}

#[cfg(feature = "minivec_nightly")]
impl<T: Copy> Clone for MiniVec<T> {
  fn clone(&self) -> Self {
    if self.is_empty() {
      return MiniVec::<T>::new();
    }

    let len = self.len();
    let mut cpy = MiniVec::<T>::with_capacity(len);

    let src = self.as_ptr();
    let dst = cpy.as_mut_ptr();
    let count = len;

    unsafe { core::ptr::copy_nonoverlapping(src, dst, count) };
    unsafe { cpy.set_len(len) };

    cpy
  }
}

#[cfg(not(feature = "minivec_nightly"))]
impl<T: Clone> Clone for MiniVec<T> {
  fn clone(&self) -> Self {
    if self.is_default() {
      return MiniVec::new();
    }

    let mut copy = MiniVec::<T>::new();

    copy.reserve(self.len());
    for i in 0..self.len() {
      copy.push(self[i].clone());
    }

    copy
  }
}
