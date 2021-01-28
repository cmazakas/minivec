use crate::MiniVec;

impl<T, V> PartialEq<V> for MiniVec<T>
where
    V: AsRef<[T]>,
    T: PartialEq,
{
    fn eq(&self, other: &V) -> bool {
        let lhs: &[T] = &*self;
        let rhs: &[T] = other.as_ref();

        lhs == rhs
    }
}

impl<T, V> PartialOrd<V> for MiniVec<T>
where
    V: AsRef<[T]>,
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &V) -> Option<core::cmp::Ordering> {
        let x: &[T] = &**self;
        let y: &[T] = other.as_ref();

        x.partial_cmp(&y)
    }
}
