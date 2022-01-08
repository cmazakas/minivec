use crate::MiniVec;

// We take tons of inspiration from the stdlib here but also choose to optimize
// based on the underlying type needing to be dropped or not
//
// This enables us to take advantage of types that aren't Copy but don't implement Drop
// by avoiding the store to the `DropGuard { len }` data member.
//

fn to_vec<T: Clone>(xs: &[T]) -> MiniVec<T> {
  struct DropGuard<'a, T> {
    pub vec: &'a mut MiniVec<T>,
    pub len: usize,
  }

  impl<'a, T> Drop for DropGuard<'a, T> {
    fn drop(&mut self) {
      unsafe { self.vec.set_len(self.len) };
    }
  }

  if xs.is_empty() {
    return MiniVec::<T>::new();
  }

  let len = xs.len();
  let mut cpy = MiniVec::<T>::with_capacity(len);

  if core::mem::needs_drop::<T>() {
    let mut guard = DropGuard {
      vec: &mut cpy,
      len: 0,
    };

    let dst = guard.vec.spare_capacity_mut();
    let cnt = &mut guard.len;

    xs.iter().zip(dst.iter_mut()).for_each(|(x, p)| {
      *p = core::mem::MaybeUninit::new(x.clone());
      *cnt += 1;
    });

    unsafe { guard.vec.set_len(len) };
    core::mem::forget(guard);
  } else {
    let dst = cpy.spare_capacity_mut();

    xs.iter().zip(dst.iter_mut()).for_each(|(x, p)| {
      *p = core::mem::MaybeUninit::new(x.clone());
    });

    unsafe { cpy.set_len(len) };
  }

  cpy
}

#[cfg(feature = "minivec_nightly")]
impl<T: Clone> Clone for MiniVec<T> {
  default fn clone(&self) -> Self {
    to_vec(self.as_slice())
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
    to_vec(self.as_slice())
  }
}
