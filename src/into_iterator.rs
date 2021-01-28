use crate::MiniVec;

use crate::r#impl::into_iter::IntoIter;

impl<T> core::iter::IntoIterator for MiniVec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::<T>::new(self)
    }
}

impl<'a, T> core::iter::IntoIterator for &'a MiniVec<T> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> core::iter::IntoIterator for &'a mut MiniVec<T> {
    type Item = &'a mut T;
    type IntoIter = core::slice::IterMut<'a, T>;

    fn into_iter(self) -> core::slice::IterMut<'a, T> {
        self.iter_mut()
    }
}
