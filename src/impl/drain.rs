use crate::MiniVec;

extern crate alloc;

use core::{marker::PhantomData, mem, ptr};

pub struct Drain<'a, T: 'a> {
    vec_: ptr::NonNull<MiniVec<T>>,
    drain_pos_: ptr::NonNull<T>,
    drain_end_: ptr::NonNull<T>,
    remaining_pos_: ptr::NonNull<T>,
    remaining_: usize,
    marker_: PhantomData<&'a T>,
}

pub fn make_drain<'a, T>(
    vec: &mut MiniVec<T>,
    data: *mut T,
    remaining: usize,
    start_idx: usize,
    end_idx: usize,
) -> Drain<'a, T> {
    Drain {
        vec_: ptr::NonNull::from(vec),
        drain_pos_: unsafe { ptr::NonNull::new_unchecked(data.add(start_idx)) },
        drain_end_: unsafe { ptr::NonNull::new_unchecked(data.add(end_idx)) },
        remaining_pos_: unsafe { ptr::NonNull::new_unchecked(data.add(end_idx)) },
        remaining_: remaining,
        marker_: PhantomData,
    }
}

impl<T> Iterator for Drain<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.drain_pos_ >= self.drain_end_ {
            return None;
        }

        let p = self.drain_pos_.as_ptr();
        let tmp = unsafe { ptr::read(p) };
        self.drain_pos_ = unsafe { ptr::NonNull::new_unchecked(p.add(1)) };
        Some(tmp)
    }
}

impl<T> DoubleEndedIterator for Drain<'_, T> {
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

                    let src = self.drain.remaining_pos_.as_ptr();
                    let dst = unsafe { v.as_mut_ptr().add(v_len) };

                    if src == dst {
                        return;
                    }

                    unsafe {
                        ptr::copy(src, dst, self.drain.remaining_);
                        v.set_len(v_len + self.drain.remaining_);
                    };
                }
            }
        }

        while let Some(item) = self.next() {
            let guard = DropGuard { drain: self };
            drop(item);
            mem::forget(guard);
        }

        DropGuard { drain: self };
    }
}
