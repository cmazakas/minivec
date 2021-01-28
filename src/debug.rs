use crate::MiniVec;

impl<T: core::fmt::Debug> core::fmt::Debug for MiniVec<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let this: &[T] = &*self;

        this.fmt(f)
    }
}
