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
