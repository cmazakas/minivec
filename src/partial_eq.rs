use crate::MiniVec;

macro_rules! minivec_eq_impl {
  ([$($args:tt)*] $lhs:ty, $rhs:ty) => {
    impl<T, U, $($args)*> PartialEq<$rhs> for $lhs
    where
      T: PartialEq<U>,
    {
      #[inline]
      fn eq(&self, other: &$rhs) -> bool {
        self[..] == other[..]
      }
    }
  };
}

minivec_eq_impl! { [] MiniVec<T>, MiniVec<U> }
minivec_eq_impl! { [] MiniVec<T>, [U] }
minivec_eq_impl! { [] MiniVec<T>, &[U] }
minivec_eq_impl! { [] MiniVec<T>, &mut [U] }
minivec_eq_impl! { [] &[T], MiniVec<U> }
minivec_eq_impl! { [] &mut [T], MiniVec<U> }
minivec_eq_impl! { [] MiniVec<T>, alloc::vec::Vec<U> }
minivec_eq_impl! { [const N: usize] MiniVec<T>, [U; N] }
minivec_eq_impl! { [const N: usize] MiniVec<T>, &[U; N] }

impl<T> PartialOrd for MiniVec<T>
where
  T: PartialOrd,
{
  fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
    let x: &[T] = &**self;
    let y: &[T] = &**other;
    PartialOrd::partial_cmp(x, y)
  }
}
