use crate::MiniVec;

use std::{
    ops::{Index, IndexMut},
    slice::SliceIndex,
};

impl<T, I> Index<I> for MiniVec<T>
where
    I: SliceIndex<[T]>,
{
    type Output = <I as SliceIndex<[T]>>::Output;

    fn index(&self, index: I) -> &<MiniVec<T> as Index<I>>::Output {
        let v: &[T] = &**self;
        Index::index(v, index)
    }
}

impl<T, I> IndexMut<I> for MiniVec<T>
where
    I: SliceIndex<[T]>,
{
    fn index_mut(&mut self, index: I) -> &mut <MiniVec<T> as Index<I>>::Output {
        let v: &mut [T] = &mut **self;
        IndexMut::index_mut(v, index)
    }
}
