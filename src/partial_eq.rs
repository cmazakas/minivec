use crate::MiniVec;

macro_rules! minivec_eq_impl {
    ($lhs:ty, $rhs:ty) => {
        impl<T, U> PartialEq<$rhs> for $lhs
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

minivec_eq_impl! { MiniVec<T>, MiniVec<U> }
minivec_eq_impl! { MiniVec<T>, [U] }
minivec_eq_impl! { MiniVec<T>, &[U] }
minivec_eq_impl! { MiniVec<T>, &mut [U] }
minivec_eq_impl! { &[T], MiniVec<U> }
minivec_eq_impl! { &mut [T], MiniVec<U> }
minivec_eq_impl! { MiniVec<T>, alloc::vec::Vec<U> }

minivec_eq_impl! { MiniVec<T>, [U; 0] }
minivec_eq_impl! { MiniVec<T>, [U; 1] }
minivec_eq_impl! { MiniVec<T>, [U; 2] }
minivec_eq_impl! { MiniVec<T>, [U; 3] }
minivec_eq_impl! { MiniVec<T>, [U; 4] }
minivec_eq_impl! { MiniVec<T>, [U; 5] }
minivec_eq_impl! { MiniVec<T>, [U; 6] }
minivec_eq_impl! { MiniVec<T>, [U; 7] }
minivec_eq_impl! { MiniVec<T>, [U; 8] }
minivec_eq_impl! { MiniVec<T>, [U; 9] }
minivec_eq_impl! { MiniVec<T>, [U; 10] }
minivec_eq_impl! { MiniVec<T>, [U; 11] }
minivec_eq_impl! { MiniVec<T>, [U; 12] }
minivec_eq_impl! { MiniVec<T>, [U; 13] }
minivec_eq_impl! { MiniVec<T>, [U; 14] }
minivec_eq_impl! { MiniVec<T>, [U; 15] }
minivec_eq_impl! { MiniVec<T>, [U; 16] }
minivec_eq_impl! { MiniVec<T>, [U; 17] }
minivec_eq_impl! { MiniVec<T>, [U; 18] }
minivec_eq_impl! { MiniVec<T>, [U; 19] }
minivec_eq_impl! { MiniVec<T>, [U; 20] }
minivec_eq_impl! { MiniVec<T>, [U; 21] }
minivec_eq_impl! { MiniVec<T>, [U; 22] }
minivec_eq_impl! { MiniVec<T>, [U; 23] }
minivec_eq_impl! { MiniVec<T>, [U; 24] }
minivec_eq_impl! { MiniVec<T>, [U; 25] }
minivec_eq_impl! { MiniVec<T>, [U; 26] }
minivec_eq_impl! { MiniVec<T>, [U; 27] }
minivec_eq_impl! { MiniVec<T>, [U; 28] }
minivec_eq_impl! { MiniVec<T>, [U; 29] }
minivec_eq_impl! { MiniVec<T>, [U; 30] }
minivec_eq_impl! { MiniVec<T>, [U; 31] }
minivec_eq_impl! { MiniVec<T>, [U; 32] }

minivec_eq_impl! { MiniVec<T>, &[U; 0] }
minivec_eq_impl! { MiniVec<T>, &[U; 1] }
minivec_eq_impl! { MiniVec<T>, &[U; 2] }
minivec_eq_impl! { MiniVec<T>, &[U; 3] }
minivec_eq_impl! { MiniVec<T>, &[U; 4] }
minivec_eq_impl! { MiniVec<T>, &[U; 5] }
minivec_eq_impl! { MiniVec<T>, &[U; 6] }
minivec_eq_impl! { MiniVec<T>, &[U; 7] }
minivec_eq_impl! { MiniVec<T>, &[U; 8] }
minivec_eq_impl! { MiniVec<T>, &[U; 9] }
minivec_eq_impl! { MiniVec<T>, &[U; 10] }
minivec_eq_impl! { MiniVec<T>, &[U; 11] }
minivec_eq_impl! { MiniVec<T>, &[U; 12] }
minivec_eq_impl! { MiniVec<T>, &[U; 13] }
minivec_eq_impl! { MiniVec<T>, &[U; 14] }
minivec_eq_impl! { MiniVec<T>, &[U; 15] }
minivec_eq_impl! { MiniVec<T>, &[U; 16] }
minivec_eq_impl! { MiniVec<T>, &[U; 17] }
minivec_eq_impl! { MiniVec<T>, &[U; 18] }
minivec_eq_impl! { MiniVec<T>, &[U; 19] }
minivec_eq_impl! { MiniVec<T>, &[U; 20] }
minivec_eq_impl! { MiniVec<T>, &[U; 21] }
minivec_eq_impl! { MiniVec<T>, &[U; 22] }
minivec_eq_impl! { MiniVec<T>, &[U; 23] }
minivec_eq_impl! { MiniVec<T>, &[U; 24] }
minivec_eq_impl! { MiniVec<T>, &[U; 25] }
minivec_eq_impl! { MiniVec<T>, &[U; 26] }
minivec_eq_impl! { MiniVec<T>, &[U; 27] }
minivec_eq_impl! { MiniVec<T>, &[U; 28] }
minivec_eq_impl! { MiniVec<T>, &[U; 29] }
minivec_eq_impl! { MiniVec<T>, &[U; 30] }
minivec_eq_impl! { MiniVec<T>, &[U; 31] }
minivec_eq_impl! { MiniVec<T>, &[U; 32] }

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
