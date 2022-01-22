use crate::Header;

extern crate alloc;

// copy what the great glen fernandes does in Boost.Align for `align_up`
//
pub const fn next_aligned(n: usize, alignment: usize) -> usize {
  (n + (alignment - 1)) & !(alignment - 1)
}

pub const fn next_capacity<T>(capacity: usize) -> usize {
  let elem_size = core::mem::size_of::<T>();

  if capacity == 0 {
    return match elem_size {
      1 => 8,
      2..=1024 => 4,
      _ => 1,
    };
  }

  capacity.saturating_mul(2)
}

pub const fn max_align<T>() -> usize {
  let align_t = core::mem::align_of::<T>();
  let header_align = core::mem::align_of::<Header>();

  if align_t > header_align {
    align_t
  } else {
    header_align
  }
}

pub const fn make_layout<T>(capacity: usize) -> alloc::alloc::Layout {
  let alignment = max_align::<T>();
  let header_size = core::mem::size_of::<Header>();

  let num_bytes = next_aligned(header_size, alignment)
    + next_aligned(capacity * core::mem::size_of::<T>(), alignment);

  unsafe { alloc::alloc::Layout::from_size_align_unchecked(num_bytes, alignment) }
}

pub const fn max_elems<T>() -> usize {
  let alignment = max_align::<T>();
  let header_bytes = next_aligned(core::mem::size_of::<Header>(), alignment);
  let max = usize::MAX;
  let m = max - (max % alignment) - header_bytes;

  m / core::mem::size_of::<T>()
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn next_aligned_test() {
    assert_eq!(next_aligned(9, 4), 12);
    assert_eq!(next_aligned(13, 4), 16);
    assert_eq!(next_aligned(12, 4), 12);
    assert_eq!(next_aligned(13, 1), 13);
    assert_eq!(next_aligned(8, 8), 8);
    assert_eq!(next_aligned(16, 32), 32);
    assert_eq!(next_aligned(16, 512), 512);
  }

  #[repr(align(512))]
  struct OverAligned {
    _data: [u8; 512],
  }

  #[test]
  fn max_align_test() {
    let header_alignment = core::mem::align_of::<Header>();

    assert!(core::mem::align_of::<i32>() <= core::mem::align_of::<Header>());
    assert_eq!(max_align::<i32>(), header_alignment);

    assert!(core::mem::align_of::<u8>() <= core::mem::align_of::<Header>());
    assert_eq!(max_align::<u8>(), header_alignment);

    assert!(core::mem::align_of::<OverAligned>() > core::mem::align_of::<Header>());
    assert_eq!(
      max_align::<OverAligned>(),
      core::mem::align_of::<OverAligned>()
    );
  }

  #[test]
  fn make_layout_test() {
    // empty
    //
    let layout = make_layout::<i32>(0);

    assert_eq!(layout.align(), core::mem::align_of::<Header>());
    assert_eq!(layout.size(), core::mem::size_of::<Header>());

    // non-empty, less than
    //
    let layout = make_layout::<i32>(512);
    assert!(core::mem::align_of::<i32>() < core::mem::align_of::<Header>());
    assert_eq!(layout.align(), core::mem::align_of::<Header>());
    assert_eq!(
      layout.size(),
      core::mem::size_of::<Header>() + 512 * core::mem::size_of::<i32>()
    );

    // non-empty, equal
    //
    let layout = make_layout::<i64>(512);
    assert_eq!(
      core::mem::align_of::<i64>(),
      core::mem::align_of::<Header>()
    );
    assert_eq!(layout.align(), core::mem::align_of::<Header>());
    assert_eq!(
      layout.size(),
      core::mem::size_of::<Header>() + 512 * core::mem::size_of::<i64>()
    );

    // non-empty, greater
    let layout = make_layout::<OverAligned>(512);
    assert!(core::mem::align_of::<OverAligned>() > core::mem::align_of::<Header>());
    assert_eq!(layout.align(), core::mem::align_of::<OverAligned>());
    assert_eq!(
      layout.size(),
      next_aligned(
        core::mem::size_of::<Header>(),
        core::mem::align_of::<OverAligned>()
      ) + 512 * core::mem::size_of::<OverAligned>()
    );
  }
}
