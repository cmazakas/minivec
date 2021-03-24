pub struct DrainFilter<'a, T, F>
where
  F: core::ops::FnMut(&mut T) -> bool,
{
  vec: &'a mut crate::MiniVec<T>,
  pred: F,
  old_len: usize,
  new_len: usize,
  pos: usize,
  panicked: bool,
}

pub fn make_drain_filter_iterator<T, F>(
  vec: &mut crate::MiniVec<T>,
  pred: F,
) -> DrainFilter<'_, T, F>
where
  F: core::ops::FnMut(&mut T) -> bool,
{
  let old_len = vec.len();
  DrainFilter {
    vec,
    pred,
    old_len,
    new_len: 0,
    pos: 0,
    panicked: false,
  }
}

impl<T, F> core::iter::Iterator for DrainFilter<'_, T, F>
where
  F: core::ops::FnMut(&mut T) -> bool,
{
  type Item = T;

  fn next(&mut self) -> Option<Self::Item> {
    while self.pos < self.old_len {
      let data = self.vec.data();
      let mut val = unsafe { &mut *data.add(self.pos) };

      self.panicked = true;

      let pred_result = (self.pred)(&mut val);

      self.panicked = false;

      if pred_result {
        self.pos += 1;
        return Some(unsafe { core::ptr::read(val as *mut T) });
      }

      if self.pos > self.new_len {
        let src = val as *mut T;
        let dst = unsafe { data.add(self.new_len) };
        unsafe { core::ptr::copy_nonoverlapping(src, dst, 1) };
      }

      self.pos += 1;
      self.new_len += 1;
    }

    None
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    (0, Some(self.old_len - self.pos))
  }
}

struct DropGuard<'a, 'b, T, F>
where
  F: core::ops::FnMut(&mut T) -> bool,
{
  drain: &'b mut DrainFilter<'a, T, F>,
}

impl<'a, 'b, T, F> Drop for DropGuard<'a, 'b, T, F>
where
  F: core::ops::FnMut(&mut T) -> bool,
{
  fn drop(&mut self) {
    let num_remaining = self.drain.old_len - self.drain.pos;
    let num_drained = self.drain.pos - self.drain.new_len;

    if num_remaining > 0 && num_drained > 0 {
      let data = self.drain.vec.as_mut_ptr();

      let src = unsafe { data.add(self.drain.pos) };
      let dst = unsafe { data.add(self.drain.new_len) };

      unsafe { core::ptr::copy(src, dst, num_remaining) };
    }

    if self.drain.old_len == 0 {
      return;
    }

    unsafe { self.drain.vec.set_len(self.drain.new_len + num_remaining) };
  }
}

impl<T, F> Drop for DrainFilter<'_, T, F>
where
  F: core::ops::FnMut(&mut T) -> bool,
{
  fn drop(&mut self) {
    let drop_guard = DropGuard { drain: self };
    if drop_guard.drain.panicked {
      return;
    }

    drop_guard.drain.for_each(drop);
  }
}
