use crate::Header;

use std::{alloc::Layout, mem};

pub fn next_aligned(num_bytes: usize, alignment: usize) -> usize {
    let remaining = num_bytes % alignment;
    if remaining == 0 {
        num_bytes
    } else {
        num_bytes + (alignment - remaining)
    }
}

pub fn next_capacity<T>(capacity: usize) -> usize {
    let elem_size = mem::size_of::<T>();

    if capacity == 0 {
        return match elem_size {
            1 => 8,
            2..=1024 => 4,
            _ => 1,
        };
    }

    2 * capacity
}

pub fn max_align<T>() -> usize {
    let align_t = mem::align_of::<T>();
    let header_align = mem::align_of::<Header<T>>();
    std::cmp::max(align_t, header_align)
}

pub fn make_layout<T>(cap: usize) -> Layout {
    let alignment = max_align::<T>();

    let header_size = mem::size_of::<Header<T>>();
    let num_bytes = if cap == 0 {
        header_size
    } else {
        next_aligned(header_size, mem::align_of::<T>()) + cap * mem::size_of::<T>()
    };

    Layout::from_size_align(num_bytes, alignment).unwrap()
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
        data: [u8; 512],
    }

    #[test]
    fn max_align_test() {
        let header_alignment = mem::align_of::<Header<()>>();

        assert!(mem::align_of::<i32>() <= mem::align_of::<Header<()>>());
        assert_eq!(max_align::<i32>(), header_alignment);

        assert!(mem::align_of::<u8>() <= mem::align_of::<Header<()>>());
        assert_eq!(max_align::<u8>(), header_alignment);

        assert!(mem::align_of::<OverAligned>() > mem::align_of::<Header<()>>());
        assert_eq!(max_align::<OverAligned>(), mem::align_of::<OverAligned>());
    }

    #[test]
    fn make_layout_test() {
        // empty
        //
        let layout = make_layout::<i32>(0);

        assert_eq!(layout.align(), mem::align_of::<Header<i32>>());
        assert_eq!(layout.size(), mem::size_of::<Header<i32>>());

        // non-empty, less than
        //
        let layout = make_layout::<i32>(512);
        assert!(mem::align_of::<i32>() < mem::align_of::<Header<i32>>());
        assert_eq!(layout.align(), mem::align_of::<Header<i32>>());
        assert_eq!(
            layout.size(),
            mem::size_of::<Header<i32>>() + 512 * mem::size_of::<i32>()
        );

        // non-empty, equal
        //
        let layout = make_layout::<i64>(512);
        assert_eq!(mem::align_of::<i64>(), mem::align_of::<Header<i64>>());
        assert_eq!(layout.align(), mem::align_of::<Header<i64>>());
        assert_eq!(
            layout.size(),
            mem::size_of::<Header<i64>>() + 512 * mem::size_of::<i64>()
        );

        // non-empty, greater
        let layout = make_layout::<OverAligned>(512);
        assert!(mem::align_of::<OverAligned>() > mem::align_of::<Header<OverAligned>>());
        assert_eq!(layout.align(), mem::align_of::<OverAligned>());
        assert_eq!(
            layout.size(),
            next_aligned(
                mem::size_of::<Header<OverAligned>>(),
                mem::align_of::<OverAligned>()
            ) + 512 * mem::size_of::<OverAligned>()
        );
    }
}
