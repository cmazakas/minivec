#![no_std]
#![warn(clippy::pedantic)]
#![doc(html_playground_url = "https://play.rust-lang.org/")]

//! A space-optimized version of `std::vec::Vec` that's only the size of a single pointer!
//! Ideal for low-level APIs where ABI calling conventions will typically require most structs be
//! spilled onto the stack and copied instead of being passed solely in registers
//!
//! For example, x64 msvc ABI:
//! > There's a strict one-to-one correspondence between a function call's arguments and the
//! > registers used for those arguments. Any argument that doesn't fit in 8 bytes, or isn't
//! > 1, 2, 4, or 8 bytes, must be passed by reference. A single argument is never spread across
//! > multiple registers.
//!
//! The interface largely matches that of `Vec` but unfortunately, the stdlib always has access to
//! newer features, more unstable APIs so `MiniVec` will naturally be a bit more boring.
//!

extern crate alloc;

use core::{
    cmp::Ordering,
    marker::PhantomData,
    mem,
    ops::{Bound, FnMut, RangeBounds},
    ptr, slice,
};

mod r#impl;

mod as_mut;
mod as_ref;
mod borrow;
mod clone;
mod debug;
mod default;
mod deref;
mod drop;
mod eq;
mod extend;
mod from;
mod from_iterator;
mod hash;
mod index;
mod into_iterator;
mod ord;
mod partial_eq;
#[cfg(feature = "serde")]
mod serde;

use crate::r#impl::drain::make_drain_iterator;
use crate::r#impl::helpers::{make_layout, next_aligned, next_capacity};
use crate::r#impl::splice::make_splice_iterator;

pub use crate::r#impl::{Drain, IntoIter, Splice};

pub struct MiniVec<T> {
    buf_: *mut u8,
    phantom_: PhantomData<T>,
}

struct Header {
    len_: usize,
    cap_: usize,
}

const fn get_offset<T>() -> usize {
    next_aligned(mem::size_of::<Header>(), mem::align_of::<T>())
}

impl<T> MiniVec<T> {
    fn header(&self) -> &Header {
        #[allow(clippy::cast_ptr_alignment)]
        unsafe {
            &*(self.buf_ as *const Header)
        }
    }

    fn header_mut(&mut self) -> &mut Header {
        #[allow(clippy::cast_ptr_alignment)]
        unsafe {
            &mut *(self.buf_ as *mut Header)
        }
    }

    fn data(&self) -> *mut T {
        debug_assert!(!self.buf_.is_null());

        let count = get_offset::<T>();
        unsafe { self.buf_.add(count) as *mut T }
    }

    fn grow(&mut self, capacity: usize) {
        debug_assert!(capacity >= self.len());

        let old_capacity = self.capacity();
        let new_capacity = capacity;

        if new_capacity == old_capacity {
            return;
        }

        let new_layout = make_layout::<T>(new_capacity);

        let len = self.len();

        let new_buf = if self.buf_.is_null() {
            unsafe { alloc::alloc::alloc(new_layout) }
        } else {
            let old_layout = make_layout::<T>(old_capacity);

            unsafe { alloc::alloc::realloc(self.buf_, old_layout, new_layout.size()) }
        };

        if new_buf.is_null() {
            alloc::alloc::handle_alloc_error(new_layout);
        }

        let header = Header {
            len_: len,
            cap_: new_capacity,
        };

        #[allow(clippy::cast_ptr_alignment)]
        unsafe {
            ptr::write(new_buf as *mut Header, header)
        };

        self.buf_ = new_buf;
    }

    /// `append` moves every element from `other` to the back of `self`. `other.is_empty()` is
    /// `true` once this operation completes and its capacity is unaffected.
    ///
    /// # Example
    ///
    /// ```
    /// use minivec::mini_vec;
    /// let mut vec = mini_vec![1, 2, 3];
    /// let mut vec2 = mini_vec![4, 5, 6];
    /// vec.append(&mut vec2);
    /// assert_eq!(vec, [1, 2, 3, 4, 5, 6]);
    /// assert_eq!(vec2, []);
    /// ```
    ///
    pub fn append(&mut self, other: &mut MiniVec<T>) {
        if other.is_empty() {
            return;
        }

        let other_len = other.len();
        self.reserve(other_len);

        unsafe {
            ptr::copy_nonoverlapping(other.as_ptr(), self.as_mut_ptr().add(self.len()), other_len);
        };

        other.header_mut().len_ = 0;
        self.header_mut().len_ += other_len;
    }

    /// `as_mut_ptr` returns a `*mut T` to the underlying array.
    ///
    /// * May return a null pointer.
    /// * May be invalidated by calls to `reserve()`
    /// * Can outlive its backing `MiniVec`
    ///
    /// # Example
    /// ```
    /// use minivec::mini_vec;
    /// let mut vec = mini_vec![1, 2, 3, 4];
    /// let mut p = vec.as_mut_ptr();
    ///
    /// for idx in 0..vec.len() {
    ///     unsafe {
    ///         *p.add(idx) = *p.add(idx) + 3;
    ///     }
    /// }
    ///
    /// assert_eq!(vec, [4, 5, 6, 7]);
    /// ```
    ///
    pub fn as_mut_ptr(&mut self) -> *mut T {
        if self.buf_.is_null() {
            return ptr::null_mut();
        }

        self.data()
    }

    /// `as_mut_slice` obtains a mutable reference to a slice that's attached to the backing array.
    ///
    /// # Example
    /// ```
    /// use minivec::mini_vec;
    ///
    /// let mut vec = mini_vec![1, 2, 3];
    /// {
    ///     let as_slice: &mut [_] = vec.as_mut_slice();
    ///     as_slice[0] = 1337;
    /// }
    /// assert_eq!(vec[0], 1337);
    /// ```
    ///
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self
    }

    /// `as_ptr` obtains a `*const T` to the underlying allocation.
    ///
    /// * May return a null pointer.
    /// * May be invalidated by calls to `reserve()`
    /// * Can outlive its backing `MiniVec`
    ///
    /// # Example
    /// ```
    /// use minivec::mini_vec;
    /// let mut vec = mini_vec![1, 2, 3, 4];
    /// let mut p = vec.as_mut_ptr();
    ///
    /// let mut sum = 0;
    /// for idx in 0..vec.len() {
    ///     unsafe {
    ///         sum += *p.add(idx);
    ///     }
    /// }
    ///
    /// assert_eq!(sum, 1 + 2 + 3 + 4);
    /// ```
    ///
    #[must_use]
    pub fn as_ptr(&self) -> *const T {
        if self.buf_.is_null() {
            return ptr::null();
        }

        self.data()
    }

    /// `as_slice` obtains a reference to the backing array as an immutable slice of `T`.
    ///
    /// # Example
    /// ```
    /// use minivec::mini_vec;
    ///
    /// let vec = mini_vec![1, 2, 3, 4];
    /// let mut sum = 0;
    ///
    /// let as_slice : &[_] = vec.as_slice();
    ///
    /// for idx in 0..vec.len() {
    ///     sum += as_slice[idx];
    /// }
    ///
    /// assert_eq!(sum, 1 + 2 + 3 + 4);
    /// ```
    ///
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        self
    }

    /// `capacity` obtains the number of elements that can be inserted into the `MiniVec` before a
    /// reallocation will be required.
    ///
    /// Note: `MiniVec` aims to use the same reservation policy as `std::vec::Vec`.
    ///
    /// # Example
    ///
    /// ```
    /// let vec = minivec::MiniVec::<i32>::with_capacity(128);
    ///
    /// assert_eq!(vec.len(), 0);
    /// assert_eq!(vec.capacity(), 128);
    /// ```
    ///
    #[must_use]
    pub fn capacity(&self) -> usize {
        if self.buf_.is_null() {
            0
        } else {
            self.header().cap_
        }
    }

    /// `clear` clears the current contents of the `MiniVec`. Afterwards, `len()` will return 0.
    /// `capacity()` is not affected.
    ///
    /// Logically equivalent to calling `minivec::MiniVec::truncate(0)`.
    ///
    /// Note: destruction order of the contained elements is not guaranteed.
    ///
    /// # Example
    ///
    /// ```
    /// let mut vec = minivec::mini_vec![-1; 256];
    ///
    /// let cap = vec.capacity();
    ///
    /// assert_eq!(vec.len(), 256);
    ///
    /// vec.clear();
    ///
    /// assert_eq!(vec.len(), 0);
    /// assert_eq!(vec.capacity(), cap);
    /// ```
    ///
    pub fn clear(&mut self) {
        self.truncate(0);
    }

    /// `dedeup` "de-duplicates" all adjacent identical values in the vector.
    ///
    /// Logically equivalent to calling `minivec::MiniVec::dedup_by(|x, y| x == y)`.
    ///
    /// # Example
    ///
    /// ```
    /// let mut v = minivec::mini_vec![1, 2, 1, 1, 3, 3, 3, 4, 5, 4];
    /// v.dedup();
    ///
    /// assert_eq!(v, [1, 2, 1, 3, 4, 5, 4]);
    /// ```
    ///
    pub fn dedup(&mut self)
    where
        T: PartialEq,
    {
        self.dedup_by(|x, y| x == y);
    }

    /// `dedup_by` "de-duplicates" all adjacent elements for which the supplied binary predicate
    /// returns true.
    ///
    /// # Example
    ///
    /// ```
    /// let mut vec = minivec::mini_vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    ///
    /// vec.dedup_by(|x, y| *x + *y < 8);
    ///
    /// assert_eq!(vec, [1, 7, 8, 9, 10]);
    /// ```
    ///
    pub fn dedup_by<F>(&mut self, mut pred: F)
    where
        F: FnMut(&mut T, &mut T) -> bool,
    {
        // In essence copy what the C++ stdlib does:
        // https://github.com/llvm/llvm-project/blob/032810f58986cd568980227c9531de91d8bcb1cd/libcxx/include/algorithm#L2174-L2191
        //
        let len = self.len();
        if len < 2 {
            return;
        }

        let data = self.as_mut_ptr();

        let mut read = unsafe { data.add(1) };
        let mut write = read;

        let last = unsafe { data.add(len) };

        while read < last {
            let matches = unsafe { pred(&mut *read, &mut *write.sub(1)) };
            if !matches {
                if read != write {
                    unsafe { mem::swap(&mut *read, &mut *write) };
                }
                write = unsafe { write.add(1) };
            }

            read = unsafe { read.add(1) };
        }

        self.truncate((write as usize - data as usize) / mem::size_of::<T>());
    }

    /// `dedup_by_key` "de-duplicates" all adjacent elements where `key(elem1) == key(elem2)`.
    ///
    /// # Example
    ///
    /// ```
    /// let mut vec = minivec::mini_vec!["a", "b", "c", "aa", "bbb", "cc", "dd"];
    ///
    /// vec.dedup_by_key(|x| x.len());
    ///
    /// assert_eq!(vec, ["a", "aa", "bbb", "cc"]);
    /// ```
    ///
    pub fn dedup_by_key<F, K>(&mut self, mut key: F)
    where
        F: FnMut(&mut T) -> K,
        K: PartialEq<K>,
    {
        self.dedup_by(|a, b| key(a) == key(b));
    }

    /// `drain` returns a `minivec::Drain` iterator which lazily removes elements from the supplied
    /// `range`.
    ///
    /// If the returned iterator is not iterated until exhaustion then the `Drop` implementation
    /// for `Drain` will remove the remaining elements.
    ///
    /// Note: panics if the supplied range would be outside the vector
    ///
    /// # Example
    ///
    /// ```
    /// let mut vec = minivec::mini_vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    ///
    /// let other_vec : minivec::MiniVec<_> = vec.drain(1..7).map(|x| x + 2).collect();
    ///
    /// assert_eq!(vec, [1, 8, 9, 10]);
    /// assert_eq!(other_vec, [4, 5, 6, 7, 8, 9]);
    /// ```
    ///
    pub fn drain<R>(&mut self, range: R) -> Drain<T>
    where
        R: RangeBounds<usize>,
    {
        let len = self.len();

        let start_idx = match range.start_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => n + 1,
            Bound::Unbounded => 0,
        };

        let end_idx = match range.end_bound() {
            Bound::Included(&n) => n + 1,
            Bound::Excluded(&n) => n,
            Bound::Unbounded => len,
        };

        if start_idx > end_idx {
            panic!(
                "start drain index (is {}) should be <= end drain index (is {})",
                start_idx, end_idx
            );
        }

        if end_idx > len {
            panic!(
                "end drain index (is {}) should be <= len (is {})",
                end_idx, len
            );
        }

        let data = self.as_mut_ptr();

        unsafe { self.set_len(start_idx) };

        make_drain_iterator(self, data, len - end_idx, start_idx, end_idx)
    }

    /// `from_raw_part` reconstructs a `MiniVec` from a previous call to `MiniVec::as_mut_ptr`.
    ///
    /// # Safety
    ///
    /// `from_raw_part` is incredibly unsafe and can only be used with the value of
    /// `MiniVec::as_mut_ptr`. This is because the allocation for the backing array stores metadata
    /// at its head and is not guaranteed to be stable so users are discouraged from attempting to
    /// support this directly.
    ///
    /// # Example
    ///
    /// ```
    /// let mut vec = minivec::mini_vec![1, 2, 3, 4];
    ///
    /// let ptr = vec.as_mut_ptr();
    ///
    /// std::mem::forget(vec);
    ///
    /// let new_vec = unsafe { minivec::MiniVec::from_raw_part(ptr) };
    ///
    /// assert_eq!(new_vec, [1, 2, 3, 4]);
    /// ```
    ///
    #[allow(clippy::cast_ptr_alignment)]
    pub unsafe fn from_raw_part(ptr: *mut T) -> MiniVec<T> {
        debug_assert!(!ptr.is_null());

        let header_size = mem::size_of::<Header>();
        let aligned = next_aligned(header_size, mem::align_of::<T>());

        let p = ptr as *mut u8;
        let buf = p.sub(aligned);

        MiniVec {
            buf_: buf,
            phantom_: PhantomData,
        }
    }

    /// `from_raw_parts` is an API-compatible version of `std::vec::Vec::from_raw_parts`. Because of
    /// `MiniVec`'s optimized layout, it's not strictly required for a user to pass the length and
    /// capacity explicitly.
    ///
    /// Like `MiniVec::from_raw_part`, this function is only safe to use with the result of a call
    /// to `MiniVec::as_mut_ptr()`.
    ///
    /// # Safety
    ///
    /// A very unsafe function that should only really be used when passing the vector to a C API.
    ///
    /// # Example
    ///
    /// ```
    /// let mut vec = minivec::mini_vec![1, 2, 3, 4];
    /// let len = vec.len();
    /// let cap = vec.capacity();
    ///
    /// let ptr = vec.as_mut_ptr();
    ///
    /// std::mem::forget(vec);
    ///
    /// let new_vec = unsafe { minivec::MiniVec::from_raw_parts(ptr, len, cap) };
    ///
    /// assert_eq!(new_vec, [1, 2, 3, 4]);
    /// ```
    ///
    #[allow(clippy::cast_ptr_alignment)]
    pub unsafe fn from_raw_parts(ptr: *mut T, length: usize, capacity: usize) -> MiniVec<T> {
        debug_assert!(!ptr.is_null());

        let header_size = mem::size_of::<Header>();
        let aligned = next_aligned(header_size, mem::align_of::<T>());

        let p = ptr as *mut u8;
        let buf = p.sub(aligned);

        debug_assert!((*(buf as *mut Header)).len_ == length);
        debug_assert!((*(buf as *mut Header)).cap_ == capacity);

        MiniVec {
            buf_: buf,
            phantom_: PhantomData,
        }
    }

    /// `insert` places an element at the specified index, subsequently shifting all elements to the
    /// right of the insertion index by 1
    ///
    /// Note: will panic when `index > vec.len()`
    ///
    /// # Example
    ///
    /// ```
    /// let mut vec = minivec::mini_vec![0, 1, 2, 3];
    /// vec.insert(1, 1337);
    /// assert_eq!(vec, [0, 1337, 1, 2, 3]);
    ///
    /// vec.insert(vec.len(), 7331);
    /// assert_eq!(vec, [0, 1337, 1, 2, 3, 7331]);
    /// ```
    ///
    pub fn insert(&mut self, index: usize, element: T) {
        let len = self.len();

        if index > len {
            panic!(
                "insertion index (is {}) should be <= len (is {})",
                index, len
            );
        }

        if len == self.capacity() {
            self.reserve(1);
        }

        let p = unsafe { self.as_mut_ptr().add(index) };
        unsafe {
            ptr::copy(p, p.add(1), len - index);
            ptr::write(p, element);
            self.set_len(len + 1);
        }
    }

    /// `is_empty()` returns whether or not the `MiniVec` has a length greater than 0.
    ///
    /// Logically equivalent to manually writing: `v.len() == 0`.
    ///
    /// # Example
    ///
    /// ```
    /// let vec = minivec::MiniVec::<i32>::with_capacity(256);
    /// assert!(vec.is_empty());
    /// assert!(vec.capacity() > 0);
    /// ```
    ///
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// `leak` "leaks" the supplied `MiniVec`, i.e. turn it into a `ManuallyDrop` instance and
    /// return a reference to the backing array via `&'a [T]` where `'a` is a user-supplied
    /// lifetime.
    ///
    /// Most useful for turning an allocation with dynamic duration into one with static duration.
    ///
    /// # Example
    ///
    /// ```
    /// let vec = minivec::mini_vec![1, 2, 3];
    /// let static_ref: &'static mut [i32] = minivec::MiniVec::leak(vec);
    /// static_ref[0] += 1;
    /// assert_eq!(static_ref, &[2, 2, 3]);
    /// ```
    ///
    #[must_use]
    pub fn leak<'a>(vec: MiniVec<T>) -> &'a mut [T]
    where
        T: 'a,
    {
        let len = vec.len();
        let mut vec = mem::ManuallyDrop::new(vec);
        let vec: &mut MiniVec<T> = &mut *vec;
        unsafe { slice::from_raw_parts_mut(vec.as_mut_ptr(), len) }
    }

    /// `len` returns the current lenght of the vector, i.e. the number of actual elements in it
    ///
    /// `capacity() >= len()` is true for all cases
    ///
    /// # Example
    ///
    /// ```
    /// let mut vec = minivec::mini_vec![-1; 256];
    /// assert_eq!(vec.len(), 256);
    /// ```
    ///
    #[must_use]
    pub fn len(&self) -> usize {
        if self.buf_.is_null() {
            0
        } else {
            self.header().len_
        }
    }

    /// `MiniVec::new` constructs an empty `MiniVec`.
    ///
    /// Note: does not allocate any memory.
    ///
    /// # Example
    ///
    /// ```
    /// let mut vec = minivec::MiniVec::<i32>::new();
    ///
    /// assert_eq!(vec.as_mut_ptr(), std::ptr::null_mut());
    /// assert_eq!(vec.len(), 0);
    /// assert_eq!(vec.capacity(), 0);
    /// ```
    ///
    #[must_use]
    pub fn new() -> MiniVec<T> {
        assert!(mem::size_of::<T>() > 0, "ZSTs currently not supported");

        MiniVec {
            buf_: ptr::null_mut(),
            phantom_: PhantomData,
        }
    }

    /// `pop` removes the last element from the vector, should it exist, and returns an `Optional`
    /// which owns the removed element.
    ///
    /// # Example
    ///
    /// ```
    /// let mut vec = minivec::mini_vec![Box::new(1)];
    /// let ptr = vec.pop().unwrap();
    /// assert_eq!(*ptr, 1);
    ///
    /// assert_eq!(vec.pop(), None);
    /// ```
    ///
    pub fn pop(&mut self) -> Option<T> {
        let len = self.len();

        if len == 0 {
            return None;
        }

        let v = unsafe { ptr::read(self.as_ptr().add(len - 1)) };
        unsafe { self.set_len(len - 1) };
        Some(v)
    }

    /// `push` appends an element `value` to the end of the vector. `push` automatically reallocates
    /// if the vector does not have sufficient capacity.
    ///
    /// # Example
    ///
    /// ```
    /// let mut vec = minivec::MiniVec::<i32>::with_capacity(64);
    ///
    /// for idx in 0..128 {
    ///     vec.push(idx);
    /// }
    ///
    /// assert_eq!(vec.len(), 128);
    /// ```
    ///
    pub fn push(&mut self, value: T) {
        let (len, capacity) = (self.len(), self.capacity());
        if len == capacity {
            self.grow(next_capacity::<T>(capacity));
        }

        let len = self.len();
        let data = self.data();

        let dst = unsafe { data.add(len) };

        unsafe {
            ptr::write(dst, value);
        };

        let mut header = self.header_mut();
        header.len_ += 1;
    }

    /// `remove` moves the element at the specified `index` and then returns it to the user. This
    /// operation shifts all elements to the right `index` to the left by one so it has a linear
    /// time complexity of `vec.len() - index`.
    ///
    /// Panics if `index >= len()`.
    ///
    /// # Example
    ///
    /// ```
    /// let mut vec = minivec::mini_vec![0, 1, 2, 3];
    /// vec.remove(0);
    ///
    /// assert_eq!(vec, [1, 2, 3]);
    /// ```
    ///
    pub fn remove(&mut self, index: usize) -> T {
        let len = self.len();
        if index >= len {
            panic!("removal index (is {}) should be < len (is {})", index, len);
        }

        unsafe {
            let p = self.as_mut_ptr().add(index);

            let x = ptr::read(p);

            let src = p.add(1);
            let dst = p;
            let count = len - index - 1;
            ptr::copy(src, dst, count);

            self.set_len(len - 1);

            x
        }
    }

    /// `remove_item` removes the first element identical to the supplied `item` using a
    /// left-to-right traversal of the elements.
    ///
    /// # Example
    ///
    /// ```
    /// let mut vec = minivec::mini_vec![0, 1, 1, 1, 2, 3, 4];
    /// vec.remove_item(&1);
    ///
    /// assert_eq!(vec, [0, 1, 1, 2, 3, 4]);
    /// ```
    ///
    pub fn remove_item<V>(&mut self, item: &V) -> Option<T>
    where
        T: PartialEq<V>,
    {
        let len = self.len();
        for i in 0..len {
            if self[i] == *item {
                return Some(self.remove(i));
            }
        }
        None
    }

    /// `reserve` ensures there is sufficient capacity for `additional` extra elements to be either
    /// inserted or appended to the end of the vector. Will reallocate if needed otherwise this
    /// function is a no-op.
    ///
    /// Guarantees that the new capacity is greater than or equal to `len() + additional`.
    ///
    /// # Example
    ///
    /// ```
    /// let mut vec = minivec::MiniVec::<i32>::new();
    ///
    /// assert_eq!(vec.capacity(), 0);
    ///
    /// vec.reserve(128);
    ///
    /// assert!(vec.capacity() >= 128);
    /// ```
    ///
    pub fn reserve(&mut self, additional: usize) {
        let capacity = self.capacity();
        let total_required = self.len() + additional;

        if total_required <= capacity {
            return;
        }

        let mut new_capacity = next_capacity::<T>(capacity);
        while new_capacity < total_required {
            new_capacity = next_capacity::<T>(new_capacity);
        }

        self.grow(new_capacity);
    }

    /// `reserve_exact` ensures that the capacity of the vector is exactly equal to
    /// `len() + additional` unless the capacity is already sufficient in which case no operation is
    /// performed.
    ///
    /// # Example
    ///
    /// ```
    /// let mut vec = minivec::MiniVec::<i32>::new();
    /// vec.reserve_exact(57);
    ///
    /// assert_eq!(vec.capacity(), 57);
    /// ```
    ///
    pub fn reserve_exact(&mut self, additional: usize) {
        let capacity = self.capacity();
        let len = self.len();

        let total_required = len + additional;
        if capacity >= total_required {
            return;
        }

        self.grow(total_required);
    }

    /// `resize` will clone the supplied `value` as many times as required until `len()` becomes
    /// `new_len`. If the current `len()` is greater than `new_len` then the vector is truncated
    /// in a way that's identical to calling `vec.truncate(new_len)`. If the `len()` and `new_len`
    /// match then no operation is performed.
    ///
    /// # Example
    ///
    /// ```
    /// let mut vec = minivec::mini_vec![-1; 256];
    ///
    /// vec.resize(512, -1);
    /// assert_eq!(vec.len(), 512);
    ///
    /// vec.resize(64, -1);
    /// assert_eq!(vec.len(), 64);
    /// ```
    ///
    pub fn resize(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
        let len = self.len();
        match new_len.cmp(&len) {
            Ordering::Equal => {}
            Ordering::Greater => {
                let num_elems = new_len - len;
                self.reserve(num_elems);
                for _i in 0..num_elems {
                    self.push(value.clone());
                }
            }
            Ordering::Less => {
                self.truncate(new_len);
            }
        }
    }

    /// `resize_with` will invoke the supplied callable `f` as many times as is required until
    /// `len() == new_len` is true. If the `new_len` exceeds the current `len()` then the vector
    /// will be resized via a call to `truncate(new_len)`. If the `new_len` and `len()` are equal
    /// then no operation is performed.
    ///
    /// # Example
    ///
    /// ```
    /// let mut vec = minivec::MiniVec::<i32>::new();
    ///
    /// vec.resize_with(128, || 1337);
    /// assert_eq!(vec.len(), 128);
    /// ```
    ///
    pub fn resize_with<F>(&mut self, new_len: usize, mut f: F)
    where
        F: FnMut() -> T,
    {
        let len = self.len();
        match new_len.cmp(&len) {
            Ordering::Equal => {}
            Ordering::Greater => {
                let num_elems = new_len - len;
                self.reserve(num_elems);
                for _i in 0..num_elems {
                    self.push(f());
                }
            }
            Ordering::Less => {
                self.truncate(new_len);
            }
        }
    }

    /// `retain` removes all elements from the vector for with `f(elem)` is `false` using a
    /// left-to-right traversal.
    ///
    /// # Example
    ///
    /// ```
    /// let mut vec = minivec::mini_vec![1, 2, 3, 4, 5, 6];
    ///
    /// let is_even = |x: &i32| *x % 2 == 0;
    /// vec.retain(is_even);
    /// assert_eq!(vec, [2, 4, 6]);
    /// ```
    ///
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        let len = self.len();

        let data = self.as_mut_ptr();

        let mut read = data;
        let mut write = read;

        let last = unsafe { data.add(len) };

        while read < last {
            let should_retain = unsafe { f(&mut *read) };
            if should_retain {
                if read != write {
                    unsafe { mem::swap(&mut *read, &mut *write) };
                }
                write = unsafe { write.add(1) };
            }

            read = unsafe { read.add(1) };
        }

        self.truncate((write as usize - data as usize) / mem::size_of::<T>());
    }

    /// `set_len` reassigns the internal `len_` data member to the user-supplied `len`.
    ///
    /// # Safety
    ///
    /// This function is unsafe in the sense that it will NOT call `.drop()` on the elements
    /// excluded from the new len so this function should only be called when `T` is a `Copy` type.
    ///
    /// # Example
    ///
    /// ```
    /// let mut vec = minivec::mini_vec![1, 2, 3, 4];
    /// unsafe { vec.set_len(2) };
    ///
    /// assert_eq!(vec.len(), 2);
    /// ```
    ///
    pub unsafe fn set_len(&mut self, len: usize) {
        self.header_mut().len_ = len;
    }

    /// `shrink_to` will attempt to adjust the backing allocation such that it has space for at
    /// least `min_capacity` elements.
    ///
    /// If the `min_capacity` is smaller than the current length of the vector then the capacity
    /// will be shrunk down to `len()`.
    ///
    /// If the `capacity()` is identical to `min_capacity` then this function does nothing.
    ///
    /// If the `min_capacity` is larger than the current capacity this function will panic.
    ///
    /// Otherwise, the allocation is reallocated with the new `min_capacity` kept in mind.
    ///
    /// # Example
    ///
    /// ```
    /// let mut vec = minivec::MiniVec::<i32>::with_capacity(128);
    /// assert!(vec.capacity() >= 128);
    ///
    /// vec.shrink_to(64);
    /// assert_eq!(vec.capacity(), 64);
    /// ```
    ///
    pub fn shrink_to(&mut self, min_capacity: usize) {
        let (len, capacity) = (self.len(), self.capacity());

        if min_capacity < len {
            self.shrink_to_fit();
            return;
        }

        if capacity == min_capacity {
            return;
        }

        if capacity < min_capacity {
            panic!("Tried to shrink to a larger capacity");
        }

        self.grow(min_capacity);
    }

    /// `shrink_to_fit` will re-adjust the backing allocation such that its capacity is now equal
    /// to its length
    ///
    /// # Example
    ///
    /// ```
    /// let mut vec = minivec::MiniVec::with_capacity(512);
    ///
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    ///
    /// vec.shrink_to_fit();
    ///
    /// assert_eq!(vec.capacity(), 3);
    /// ```
    ///
    pub fn shrink_to_fit(&mut self) {
        let len = self.len();
        if len == self.capacity() {
            return;
        }

        let capacity = len;
        self.grow(capacity);
    }

    /// `spare_capacity_mut` returns a mutable slice to `MaybeUninit<T>`. This is a more
    /// structured way of interacting with `MiniVec` as an unitialized allocation vs simply creating
    /// a vector with capacity and then mutating its contents directly via `as_mut_ptr`.
    ///
    /// Once manipulation of the unitialized elements has been completed, a call to `set_len` is
    /// required otherwise the contained elements cannot be accessed by `MiniVec`'s normal methods
    /// nor will the elements be dropped.
    ///
    /// # Example
    ///
    /// ```
    /// let mut vec = minivec::MiniVec::<i32>::with_capacity(24);
    /// let mut buf = vec.spare_capacity_mut();
    ///
    /// for idx in 0..4 {
    ///     unsafe { buf[idx].as_mut_ptr().write(idx as i32) };
    /// }
    ///
    /// unsafe { vec.set_len(4) };
    ///
    /// assert_eq!(vec, [0, 1, 2, 3]);
    /// ```
    ///
    pub fn spare_capacity_mut(&mut self) -> &mut [core::mem::MaybeUninit<T>] {
        let count = self.len();
        if count == 0 {
            return &mut [];
        }

        let data = unsafe { self.data().add(count) as *mut core::mem::MaybeUninit<T> };
        let len = self.capacity() - self.len();

        unsafe { core::slice::from_raw_parts_mut(data, len) }
    }

    /// `splice` returns a `Splice` iterator. `Splice` is similar in spirit to `Drain` but instead
    /// of simply shifting the remaining elements from the vector after it's been drained, the range
    /// is replaced with the `Iterator` specified by `replace_with`.
    ///
    /// Much like `Drain`, if the `Splice` iterator is not iterated until exhaustion then the
    /// remaining elements will be removed when the iterator is dropped.
    ///
    /// `Splice` only fills the removed region when it is dropped.
    ///
    /// Note: panics if the supplied `range` is outside of the vector's bounds.
    ///
    /// # Example
    ///
    /// ```
    /// let mut x = minivec::mini_vec![1, 2, 3, 4, 5, 6];
    /// let new = [7, 8];
    ///
    /// let y: minivec::MiniVec<_> = x.splice(1..4, new.iter().cloned()).collect();
    ///
    /// assert_eq!(x, &[1, 7, 8, 5, 6]);
    /// assert_eq!(y, &[2, 3, 4]);
    /// ```
    ///
    pub fn splice<R, I>(
        &mut self,
        range: R,
        replace_with: I,
    ) -> Splice<<I as IntoIterator>::IntoIter>
    where
        I: IntoIterator<Item = T>,
        R: RangeBounds<usize>,
    {
        let len = self.len();

        let start_idx = match range.start_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => n + 1,
            Bound::Unbounded => 0,
        };

        let end_idx = match range.end_bound() {
            Bound::Included(&n) => n + 1,
            Bound::Excluded(&n) => n,
            Bound::Unbounded => len,
        };

        if start_idx > end_idx {
            panic!(
                "start splice index (is {}) should be <= end splice index (is {})",
                start_idx, end_idx
            );
        }

        if end_idx > len {
            panic!(
                "end splice index (is {}) should be <= len (is {})",
                end_idx, len
            );
        }

        let data = self.as_mut_ptr();

        unsafe { self.set_len(start_idx) };

        make_splice_iterator(
            self,
            data,
            len - end_idx,
            start_idx,
            end_idx,
            replace_with.into_iter(),
        )
    }

    /// `split_off` will segment the vector into two, returning the new segment to the user.
    ///
    /// After this function call, `self` will have kept elements `[0, at)` while the new segment
    /// contains elements `[at, len)`.
    ///
    /// Note: panics if `at` is greater than `len()`.
    ///
    /// # Example
    ///
    /// ```
    /// let mut vec = minivec::mini_vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    ///
    /// let tail = vec.split_off(7);
    ///
    /// assert_eq!(vec, [0, 1, 2, 3, 4, 5, 6]);
    /// assert_eq!(tail, [7, 8, 9, 10]);
    /// ```
    ///
    pub fn split_off(&mut self, at: usize) -> MiniVec<T> {
        let len = self.len();
        if at > len {
            panic!("`at` split index (is {}) should be <= len (is {})", at, len);
        }

        let mut other = MiniVec::with_capacity(self.capacity());

        unsafe { self.set_len(at) }
        unsafe { other.set_len(len - at) }

        let src = unsafe { self.as_ptr().add(at) };
        let dst = other.as_mut_ptr();
        let count = len - at;

        unsafe { ptr::copy_nonoverlapping(src, dst, count) }

        other
    }

    /// `swap_remove` removes the element located at `index` and replaces it with the last value
    /// in the vector, returning the removed element to the caller.
    ///
    /// Note: panics if `index >= len()`.
    ///
    /// # Example
    ///
    /// ```
    /// let mut vec = minivec::mini_vec![1, 2, 3, 4];
    ///
    /// let num = vec.swap_remove(0);
    /// assert_eq!(num, 1);
    /// assert_eq!(vec, [4, 2, 3]);
    /// ```
    ///
    pub fn swap_remove(&mut self, index: usize) -> T {
        let len = self.len();
        if index >= len {
            panic!(
                "swap_remove index (is {}) should be < len (is {})",
                index, len
            );
        }

        let src = unsafe { ptr::read(self.as_ptr().add(len - 1)) };
        self.header_mut().len_ -= 1;

        let dst = unsafe { self.as_mut_ptr().add(index) };
        unsafe { ptr::replace(dst, src) }
    }

    /// `truncate` adjusts the length of the vector to be `len`. If `len` is greater than or equal
    /// to the current length no operation is performed. Otherwise, the vector's length is
    /// readjusted to `len` and any remaining elements to the right of `len` are dropped.
    ///
    /// # Example
    ///
    /// ```
    /// let mut vec = minivec::mini_vec![1, 2, 3, 4, 5];
    /// vec.truncate(2);
    ///
    /// assert_eq!(vec, [1, 2]);
    /// ```
    ///
    pub fn truncate(&mut self, len: usize) {
        let self_len = self.len();

        if len >= self_len {
            return;
        }

        self.header_mut().len_ = len;

        if !mem::needs_drop::<T>() {
            return;
        }

        let s = unsafe { slice::from_raw_parts_mut(self.data().add(len), self_len - len) };

        unsafe { ptr::drop_in_place(s) };
    }

    /// `with_capacity` is a static factory function that returns a `MiniVec` that contains space
    /// for `capacity` elements.
    ///
    /// This function is logically equivalent to calling `.reserve_exact()` on a vector with 0
    /// capacity.
    ///
    /// # Example
    ///
    /// ```
    /// let mut vec = minivec::MiniVec::<i32>::with_capacity(128);
    ///
    /// assert_eq!(vec.len(), 0);
    /// assert_eq!(vec.capacity(), 128);
    /// ```
    ///
    #[must_use]
    pub fn with_capacity(capacity: usize) -> MiniVec<T> {
        let mut v = MiniVec::new();
        v.reserve_exact(capacity);
        v
    }

    #[doc(hidden)]
    pub unsafe fn unsafe_write(&mut self, idx: usize, elem: T) {
        self.data().add(idx).write(elem);
    }
}

impl<T: Clone> MiniVec<T> {
    /// `extend_from_slice` will append each element from `elems` in a left-to-right order, cloning
    /// each value in `elems`.
    ///
    /// # Example
    ///
    /// ```
    /// let mut vec = minivec::mini_vec![1, 2];
    ///
    /// let s : &[i32] = &[3, 4];
    ///
    /// vec.extend_from_slice(s);
    ///
    /// assert_eq!(vec, [1, 2, 3, 4]);
    /// ```
    ///
    pub fn extend_from_slice(&mut self, elems: &[T]) {
        self.reserve(elems.len());
        for x in elems {
            self.push((*x).clone());
        }
    }
}

unsafe impl<T: core::marker::Send> core::marker::Send for MiniVec<T> {}
unsafe impl<T: core::marker::Sync> core::marker::Sync for MiniVec<T> {}

/// `mini_vec!` is a macro similar in spirit to the stdlib's `vec!`.
///
/// It supports the creation of `MiniVec` with:
/// * `mini_vec!()`
/// * `mini_vec![val1, val2, val3, ...]`
/// * `mini_vec![val; num_elems]`
///
#[macro_export]
macro_rules! mini_vec {
    () => (
        $crate::MiniVec::new()
    );
    ($elem:expr; $n:expr) => {
        {
            let mut tmp = $crate::MiniVec::with_capacity($n);

            for idx in 0..$n {
                unsafe { tmp.unsafe_write(idx, $elem.clone()) };
            }

            unsafe { tmp.set_len($n) };

            tmp
        }
     };
    ($($x:expr),+ $(,)?) => {
        {
            let mut tmp = $crate::MiniVec::new();
            $(
                tmp.push($x);
            )*
            tmp
        }
    };
}
