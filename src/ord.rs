use crate::MiniVec;

impl<T: Ord> core::cmp::Ord for MiniVec<T> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let x: &[T] = &**self;
        let y: &[T] = &**other;

        x.cmp(y)
    }
}
