use crate::MiniVec;

impl<T, I> core::ops::Index<I> for MiniVec<T>
where
    I: core::slice::SliceIndex<[T]>,
{
    type Output = <I as core::slice::SliceIndex<[T]>>::Output;

    fn index(&self, index: I) -> &<MiniVec<T> as core::ops::Index<I>>::Output {
        let v: &[T] = &**self;
        core::ops::Index::index(v, index)
    }
}

impl<T, I> core::ops::IndexMut<I> for MiniVec<T>
where
    I: core::slice::SliceIndex<[T]>,
{
    fn index_mut(&mut self, index: I) -> &mut <MiniVec<T> as core::ops::Index<I>>::Output {
        let v: &mut [T] = &mut **self;
        core::ops::IndexMut::index_mut(v, index)
    }
}
