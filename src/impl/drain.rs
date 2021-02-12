use crate::MiniVec;

extern crate alloc;

pub struct Drain<'a, T: 'a> {
    vec_: core::ptr::NonNull<MiniVec<T>>,
    data: *mut T,
    drain_pos_: usize,
    drain_end_: usize,
    remaining_pos_: usize,
    remaining_: usize,
    marker_: core::marker::PhantomData<&'a T>,
}

pub fn make_drain_iterator<'a, T>(
    vec: &mut MiniVec<T>,
    data: *mut T,
    remaining: usize,
    start_idx: usize,
    end_idx: usize,
) -> Drain<'a, T> {
    Drain {
        vec_: core::ptr::NonNull::from(vec),
        data,
        drain_pos_: start_idx,
        drain_end_: end_idx,
        remaining_pos_: end_idx,
        remaining_: remaining,
        marker_: core::marker::PhantomData,
    }
}

impl<T> Iterator for Drain<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.drain_pos_ >= self.drain_end_ {
            return None;
        }
        let p =
            unsafe { core::ptr::NonNull::new_unchecked(self.data.add(self.drain_pos_)).as_ptr() };
        let tmp = unsafe { core::ptr::read(p) };
        self.drain_pos_ += 1;
        Some(tmp)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let drain_end_ =
            unsafe { core::ptr::NonNull::new_unchecked(self.data.add(self.drain_end_)) };
        let drain_pos_ =
            unsafe { core::ptr::NonNull::new_unchecked(self.data.add(self.drain_pos_)) };
        let len = (drain_end_.as_ptr() as *const _ as usize
            - drain_pos_.as_ptr() as *const _ as usize)
            / core::mem::size_of::<T>();

        (len, Some(len))
    }
}

impl<T> ExactSizeIterator for Drain<'_, T> {}

impl<T> DoubleEndedIterator for Drain<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.drain_end_ == 0 {
            return None;
        }

        self.drain_end_ -= 1;

        if self.drain_end_ < self.drain_pos_ {
            return None;
        }

        let drain_end_ =
            unsafe { core::ptr::NonNull::new_unchecked(self.data.add(self.drain_end_)).as_ptr() };

        let tmp = unsafe { core::ptr::read(drain_end_) };

        Some(tmp)
    }
}

impl<T> Drop for Drain<'_, T> {
    fn drop(&mut self) {
        struct DropGuard<'b, 'a, T> {
            drain: &'b mut Drain<'a, T>,
        };

        impl<'b, 'a, T> Drop for DropGuard<'b, 'a, T> {
            fn drop(&mut self) {
                while let Some(_) = self.drain.next() {}

                if self.drain.remaining_ > 0 {
                    let v = unsafe { self.drain.vec_.as_mut() };
                    let v_len = v.len();

                    let src = unsafe {
                        core::ptr::NonNull::new_unchecked(
                            self.drain.data.add(self.drain.remaining_pos_),
                        )
                        .as_ptr()
                    };
                    let dst = unsafe { v.as_mut_ptr().add(v_len) };

                    if src == dst {
                        return;
                    }

                    unsafe {
                        core::ptr::copy(src, dst, self.drain.remaining_);
                        v.set_len(v_len + self.drain.remaining_);
                    };
                }
            }
        }

        // Rust's borrow system can be a little lame at times
        // we need the for-loop with the nested DropGuard because `DropGuard` mutably borrows
        // and so does .next()
        //
        // By scoping the DropGuard inside the for-loop and then forgetting it before the next
        // iterator, we get panic! safety
        //
        // We then use the final DropGuard for relocating the elements to where they should be
        //
        while let Some(item) = self.next() {
            let guard = DropGuard { drain: self };
            drop(item);
            core::mem::forget(guard);
        }

        DropGuard { drain: self };
    }
}
