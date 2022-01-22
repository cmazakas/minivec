use crate::MiniVec;

#[cfg(not(feature = "minivec_nightly"))]
impl<T> core::iter::FromIterator<T> for MiniVec<T> {
  fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
    struct DropGuard<'a, T> {
      v: &'a mut MiniVec<T>,
      len: usize,
    }

    impl<'a, T> Drop for DropGuard<'a, T> {
      fn drop(&mut self) {
        unsafe { self.v.set_len(self.len) };
      }
    }

    let iter = iter.into_iter();
    let (lower_bound, _) = iter.size_hint();
    let mut v = MiniVec::<T>::with_capacity(lower_bound.saturating_add(1));

    let mut guard = DropGuard { v: &mut v, len: 0 };

    let mut capacity = guard.v.capacity();

    iter.for_each(|item| {
      if guard.len >= capacity {
        guard.v.grow(crate::next_capacity::<T>(capacity)).unwrap();
        capacity = guard.v.capacity();
      }

      unsafe { core::ptr::write(guard.v.as_mut_ptr().add(guard.len), item) };
      guard.len += 1;
    });

    unsafe { guard.v.set_len(guard.len) };
    core::mem::forget(guard);

    v
  }
}

#[cfg(feature = "minivec_nightly")]
trait MiniVecFromIter<T, I>
where
  I: Iterator<Item = T>,
{
  fn from_iter(iter: I) -> MiniVec<T>;
}

#[cfg(feature = "minivec_nightly")]
struct SpecFromIterator<T, I: Iterator<Item = T>> {
  _a: core::marker::PhantomData<T>,
  _b: core::marker::PhantomData<I>,
}

#[cfg(feature = "minivec_nightly")]
impl<T, I: Iterator<Item = T>> MiniVecFromIter<T, I> for SpecFromIterator<T, I> {
  default fn from_iter(iter: I) -> MiniVec<T> {
    struct DropGuard<'a, T> {
      v: &'a mut MiniVec<T>,
      len: usize,
    }

    impl<'a, T> Drop for DropGuard<'a, T> {
      fn drop(&mut self) {
        unsafe { self.v.set_len(self.len) };
      }
    }

    let (lower_bound, _) = iter.size_hint();
    let mut v = MiniVec::<T>::with_capacity(lower_bound.saturating_add(1));

    let mut guard = DropGuard { v: &mut v, len: 0 };

    let mut capacity = guard.v.capacity();

    iter.for_each(|item| {
      if guard.len >= capacity {
        guard.v.grow(crate::next_capacity::<T>(capacity)).unwrap();
        capacity = guard.v.capacity();
      }

      unsafe { core::ptr::write(guard.v.as_mut_ptr().add(guard.len), item) };
      guard.len += 1;
    });

    unsafe { guard.v.set_len(guard.len) };
    core::mem::forget(guard);

    v
  }
}

#[cfg(feature = "minivec_nightly")]
impl<T, I: core::iter::TrustedLen<Item = T>> MiniVecFromIter<T, I> for SpecFromIterator<T, I> {
  fn from_iter(iter: I) -> MiniVec<T> {
    let (lower_bound, _) = iter.size_hint();
    let mut v = MiniVec::<T>::with_capacity(lower_bound);

    iter.enumerate().for_each(|(idx, item)| {
      unsafe { core::ptr::write(v.as_mut_ptr().add(idx), item) };
    });

    unsafe { v.set_len(lower_bound) };

    v
  }
}

#[cfg(feature = "minivec_nightly")]
impl<T> core::iter::FromIterator<T> for MiniVec<T> {
  fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
    let iter = iter.into_iter();
    <SpecFromIterator<T, I::IntoIter> as MiniVecFromIter<T, I::IntoIter>>::from_iter(iter)
  }
}
