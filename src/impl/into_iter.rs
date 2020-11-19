extern crate alloc;

// we diverge pretty heavily from the stdlib here
//
// we're able to pretty much hack MiniVec into being an IntoIter type simply by
// making it a data member of the struct and then manually adjusting things in
// the Header of the MiniVec
//
pub struct IntoIter<T> {
    v: crate::MiniVec<T>,
    pos: *const T,
    marker: core::marker::PhantomData<T>,
}

impl<T> IntoIter<T> {
    #[must_use]
    pub fn new(w: crate::MiniVec<T>) -> Self {
        let v = w;
        let pos = if v.buf_.is_null() {
            core::ptr::null_mut()
        } else {
            v.data()
        };

        Self {
            v,
            pos,
            marker: core::marker::PhantomData,
        }
    }

    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        if self.v.buf_.is_null() {
            &[]
        } else {
            let data = self.pos;
            unsafe { core::slice::from_raw_parts(data, self.v.len()) }
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        if self.v.buf_.is_null() {
            &mut []
        } else {
            let data: *mut T = self.pos as *mut T;
            unsafe { core::slice::from_raw_parts_mut(data, self.v.len()) }
        }
    }
}

impl<T> AsRef<[T]> for IntoIter<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T: Clone> Clone for IntoIter<T> {
    fn clone(&self) -> IntoIter<T> {
        let w = self.v.clone();
        let pos_cpy = self.pos;
        IntoIter {
            v: w,
            pos: pos_cpy,
            marker: core::marker::PhantomData,
        }
    }
}

impl<T: alloc::fmt::Debug> alloc::fmt::Debug for IntoIter<T> {
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        f.debug_tuple("MiniVec::IntoIter")
            .field(&self.as_slice())
            .finish()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.v.buf_.is_null() {
            return None;
        }

        let header = self.v.header_mut();

        let data = self.pos;
        let count = header.len_;
        let end = unsafe { data.add(count) };

        if data >= end {
            return None;
        };

        header.len_ -= 1;

        let count = header.len_;
        let src = unsafe { data.add(count) };
        Some(unsafe { core::ptr::read(src) })
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        for _ in self {}
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.v.len()
    }

    // fn is_empty(&self) -> bool {
    //     self.v.is_empty()
    // }
}

impl<T> core::iter::FusedIterator for IntoIter<T> {}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.v.buf_.is_null() {
            return None;
        }

        let header = self.v.header_mut();

        let data = self.pos;
        let count = header.len_;
        let end = unsafe { data.add(count) };

        if data >= end {
            return None;
        }

        let count = 1;
        self.pos = unsafe { data.add(count) };
        header.len_ -= 1;

        Some(unsafe { core::ptr::read(data) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.v.len();
        (len, Some(len))
    }
}

unsafe impl<T: Send> Send for IntoIter<T> {}
unsafe impl<T: Sync> Sync for IntoIter<T> {}
