use crate::MiniVec;

extern crate alloc;

use core::{
    iter::{DoubleEndedIterator, ExactSizeIterator, FromIterator, Iterator},
    marker::PhantomData,
    mem, ptr,
};

pub struct Splice<'a, I>
where
    I: 'a + Iterator,
{
    vec_: ptr::NonNull<MiniVec<I::Item>>,
    drain_pos_: ptr::NonNull<I::Item>,
    drain_end_: ptr::NonNull<I::Item>,
    remaining_pos_: ptr::NonNull<I::Item>,
    remaining_: usize,
    marker_: PhantomData<&'a I::Item>,
    fill_: I,
}

pub fn make_splice_iterator<'a, I: 'a + Iterator>(
    vec: &mut MiniVec<I::Item>,
    data: *mut I::Item,
    remaining: usize,
    start_idx: usize,
    end_idx: usize,
    fill: I,
) -> Splice<'a, I> {
    Splice {
        vec_: ptr::NonNull::from(vec),
        drain_pos_: unsafe { ptr::NonNull::new_unchecked(data.add(start_idx)) },
        drain_end_: unsafe { ptr::NonNull::new_unchecked(data.add(end_idx)) },
        remaining_pos_: unsafe { ptr::NonNull::new_unchecked(data.add(end_idx)) },
        remaining_: remaining,
        marker_: PhantomData,
        fill_: fill,
    }
}

impl<I> Iterator for Splice<'_, I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.drain_pos_ >= self.drain_end_ {
            return None;
        }

        let p = self.drain_pos_.as_ptr();
        let tmp = unsafe { ptr::read(p) };
        self.drain_pos_ = unsafe { ptr::NonNull::new_unchecked(p.add(1)) };
        Some(tmp)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.drain_end_.as_ptr() as *const _ as usize
            - self.drain_pos_.as_ptr() as *const _ as usize)
            / mem::size_of::<I::Item>();

        (len, Some(len))
    }
}

impl<I: Iterator> ExactSizeIterator for Splice<'_, I> {}

impl<I> DoubleEndedIterator for Splice<'_, I>
where
    I: Iterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let pos = unsafe { self.drain_end_.as_ptr().sub(1) };
        if pos < self.drain_pos_.as_ptr() {
            return None;
        }

        let tmp = unsafe { ptr::read(pos) };
        self.drain_end_ = unsafe { ptr::NonNull::new_unchecked(pos) };
        Some(tmp)
    }
}

impl<I: Iterator> Drop for Splice<'_, I> {
    fn drop(&mut self) {
        struct DropGuard<'b, 'a, I>
        where
            I: Iterator,
        {
            splice: &'b mut Splice<'a, I>,
        };

        impl<'b, 'a, I> Drop for DropGuard<'b, 'a, I>
        where
            I: Iterator,
        {
            fn drop(&mut self) {
                // much like Drain, remove the rest of the elements from the splice range if they
                // haven't already been exhausted
                //
                while let Some(_) = self.splice.next() {}

                let vec = unsafe { self.splice.vec_.as_mut() };

                // first, figure out where our draining operation started
                // this is at offset vec.len() from the start of [T]'s data
                //
                let drain_begin = unsafe { vec.as_mut_ptr().add(vec.len()) };

                // infer the number of items we drained by where the remaining_pos_ is
                //
                let num_drained = (self.splice.remaining_pos_.as_ptr() as usize
                    - drain_begin as usize)
                    / mem::size_of::<I::Item>();

                // fill the drained sub-section using the iterator the user supplied
                // if the iterator, for example, has more elements than the draiend region allows,
                // we need to know this so we can reallocate the vector accordingly
                //
                let needs_more = {
                    let mut needs_more = true;
                    for idx in 0..num_drained {
                        if let Some(val) = self.splice.fill_.next() {
                            unsafe {
                                ptr::write(drain_begin.add(idx), val);
                                vec.set_len(vec.len() + 1);
                            };
                        } else {
                            needs_more = false;
                        }
                    }

                    needs_more
                };

                // if we don't have any more elements in the iterator the user supplied, we can
                // go ahead and shift the tail down
                //
                if !needs_more {
                    // if the supplied iterator had exactly the number of elements that we drained,
                    // we don't need to memcpy and can instead just adjust the length of the vector
                    // and return
                    //
                    if unsafe { vec.as_ptr().add(vec.len()) == self.splice.remaining_pos_.as_ptr() }
                    {
                        unsafe { vec.set_len(vec.len() + self.splice.remaining_) };
                        return;
                    }

                    // we need to copy things down from Drain's "tail" to where our iterator left
                    // off in the drained range
                    // this basically downshifts the elements from right-to-left so it's safe to
                    // call `ptr::copy`
                    //
                    let src = self.splice.remaining_pos_.as_ptr();
                    let dst = unsafe { vec.as_mut_ptr().add(vec.len()) };
                    let count = self.splice.remaining_;
                    unsafe {
                        ptr::copy(src, dst, count);
                        vec.set_len(vec.len() + self.splice.remaining_);
                    };

                    return;
                }

                // we need to handle the rest of the iterator's elements now
                // pool them into a temporary vector for storage
                //
                let mut tmp = MiniVec::<I::Item>::from_iter(&mut self.splice.fill_);

                // reserve extra capacity if required
                // note, this will invalidate all of our previously cached pointers in the Splice
                // iterator so we have to store the offset of the drain tail manually
                //
                let capacity = vec.capacity();
                let remaining_offset = (self.splice.remaining_pos_.as_ptr() as usize
                    - vec.as_ptr() as usize)
                    / mem::size_of::<I::Item>();

                // if our vector's length + the remaining elements + the extra tmp length exceeds
                // our capacity we need to reallocate
                //
                let total_elements = vec.len() + self.splice.remaining_ + tmp.len();

                if total_elements > capacity {
                    vec.grow(total_elements);
                }

                // let's first move the Drain tail over to the right
                // we know our Drain's tail starts at the `remaining_offset` and we have to copy
                // self.splice.remaining_ elements over starting at the offset of our current len
                // plus the tail len
                //
                if self.splice.remaining_ > 0 {
                    unsafe {
                        let src = vec.as_ptr().add(remaining_offset);
                        let dst = vec.as_mut_ptr().add(vec.len() + tmp.len());
                        let count = self.splice.remaining_;
                        ptr::copy(src, dst, count);
                    };
                }

                // finally we copy the remaining tmp elements into the vector and then we make sure
                // to set its length to 0 to prevent any sort of double-frees
                //
                if !tmp.is_empty() {
                    unsafe {
                        let src = tmp.as_ptr();
                        let dst = vec.as_mut_ptr().add(vec.len());
                        let count = tmp.len();
                        ptr::copy(src, dst, count);
                    };
                }

                unsafe {
                    vec.set_len(vec.len() + self.splice.remaining_ + tmp.len());
                    if !tmp.is_empty() {
                        tmp.set_len(0)
                    };
                };
            }
        }

        while let Some(item) = self.next() {
            let guard = DropGuard { splice: self };
            drop(item);
            mem::forget(guard);
        }

        DropGuard { splice: self };
    }
}
