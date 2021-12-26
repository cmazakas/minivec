#![allow(
  unknown_lints,
  unused_must_use,
  clippy::unnecessary_operation,
  clippy::op_ref,
  clippy::verbose_bit_mask,
  clippy::reversed_empty_ranges,
  clippy::vtable_address_comparisons,
  clippy::clippy::assign_op_pattern,
  clippy::clippy::many_single_char_names,
  clippy::clippy::redundant_closure,
  clippy::unit_arg,
  clippy::unnecessary_filter_map,
  clippy::eq_op,
  clippy::redundant_slicing,
  clippy::iter_count,
  clippy::assign_op_pattern,
  clippy::redundant_closure
)]

extern crate minivec;

use minivec::mini_vec;
use minivec::MiniVec;

// This code is largely a copy-paste of the official Rust test file for `std::vec::Vec` which is
// both Apache 2.0 and MIT licensed. See the accompanying LICENSE-APACHE and LICENSE-MIT files for
// more information on those
//
// https://github.com/rust-lang/rust/blob/fa9af6a9be72e80c7c86adf656bee5964cb2f6a2/library/alloc/tests/vec.rs
// https://raw.githubusercontent.com/rust-lang/rust/fa9af6a9be72e80c7c86adf656bee5964cb2f6a2/library/alloc/tests/vec.rs
//
// ^ official source for `Vec` test file
//
// TODO:
// * implement FromIterator specialization for minivec::IntoIterator when it's stable
// * implement unsafe impl Drop<#[may_dangle] T> when it's stable
//
// Code modifications:
// * rename `Vec` to `MiniVec` and `vec!` to `mini_vec!`
// * change `size_of` test to match `size_of::<usize>()`
// * comment out yet-to-be-completed features
// * replace `box` expressions with `Box::new()`
// * comment out test assertions that require specialization
// * comment out tests that rely on Vec's Drop impl potentially dangling
//

// use std::borrow::Cow;
// use std::cell::Cell;
// use std::collections::TryReserveError::*;
use std::fmt::Debug;
// use std::iter::InPlaceIterable;
use std::mem::{size_of, swap};
use std::ops::Bound::*;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;
// use std::vec::{Drain, IntoIter};

struct DropCounter<'a> {
  count: &'a mut u32,
}

impl Drop for DropCounter<'_> {
  fn drop(&mut self) {
    *self.count += 1;
  }
}

#[test]
fn test_small_vec_struct() {
  // the only test we actually wind up changing beyond a simple renaming
  // this is because `MiniVec` is genuinely mini and is only a third the size of the genuine `Vec`
  //
  // assert_eq!(size_of::<Vec<u8>>(), size_of::<usize>() * 3);
  assert_eq!(size_of::<MiniVec<u8>>(), size_of::<usize>());
}

#[test]
fn test_double_drop() {
  struct TwoVec<T> {
    x: MiniVec<T>,
    y: MiniVec<T>,
  }

  let (mut count_x, mut count_y) = (0, 0);
  {
    let mut tv = TwoVec {
      x: MiniVec::new(),
      y: MiniVec::new(),
    };
    tv.x.push(DropCounter {
      count: &mut count_x,
    });
    tv.y.push(DropCounter {
      count: &mut count_y,
    });

    // If Vec had a drop flag, here is where it would be zeroed.
    // Instead, it should rely on its internal state to prevent
    // doing anything significant when dropped multiple times.
    drop(tv.x);

    // Here tv goes out of scope, tv.y should be dropped, but not tv.x.
  }

  assert_eq!(count_x, 1);
  assert_eq!(count_y, 1);
}

#[test]
fn test_reserve() {
  let mut v = MiniVec::new();
  assert_eq!(v.capacity(), 0);

  v.reserve(2);
  assert!(v.capacity() >= 2);

  for i in 0..16 {
    v.push(i);
  }

  assert!(v.capacity() >= 16);
  v.reserve(16);
  assert!(v.capacity() >= 32);

  v.push(16);

  v.reserve(16);
  assert!(v.capacity() >= 33)
}

// #[test]
// fn test_zst_capacity() {
//     assert_eq!(Vec::<()>::new().capacity(), usize::MAX);
// }

#[test]
fn test_indexing() {
  let v: MiniVec<isize> = mini_vec![10, 20];
  assert_eq!(v[0], 10);
  assert_eq!(v[1], 20);
  let mut x: usize = 0;
  assert_eq!(v[x], 10);
  assert_eq!(v[x + 1], 20);
  x = x + 1;
  assert_eq!(v[x], 20);
  assert_eq!(v[x - 1], 10);
}

#[test]
fn test_debug_fmt() {
  let vec1: MiniVec<isize> = mini_vec![];
  assert_eq!("[]", format!("{:?}", vec1));

  let vec2 = mini_vec![0, 1];
  assert_eq!("[0, 1]", format!("{:?}", vec2));

  let slice: &[isize] = &[4, 5];
  assert_eq!("[4, 5]", format!("{:?}", slice));
}

#[test]
fn test_push() {
  let mut v = mini_vec![];
  v.push(1);
  assert_eq!(v, [1]);
  v.push(2);
  assert_eq!(v, [1, 2]);
  v.push(3);
  assert_eq!(v, [1, 2, 3]);
}

#[test]
fn test_extend() {
  let mut v = MiniVec::new();
  let mut w = MiniVec::new();

  v.extend(w.clone());
  assert_eq!(v, &[]);

  v.extend(0..3);
  for i in 0..3 {
    w.push(i);
  }

  assert_eq!(v, w);

  v.extend(3..10);
  for i in 3..10 {
    w.push(i);
  }

  assert_eq!(v, w);

  v.extend(w.clone()); // specializes to `append`
  assert!(v.iter().eq(w.iter().chain(w.iter())));

  // Zero sized types
  // #[derive(PartialEq, Debug)]
  // struct Foo;

  // let mut a = MiniVec::new();
  // let b = mini_vec![Foo, Foo];

  // a.extend(b);
  // assert_eq!(a, &[Foo, Foo]);

  // // Double drop
  // let mut count_x = 0;
  // {
  //     let mut x = MiniVec::new();
  //     let y = mini_vec![DropCounter {
  //         count: &mut count_x,
  //     }];
  //     x.extend(y);
  // }
  // assert_eq!(count_x, 1);
}

#[test]
fn test_extend_from_slice() {
  let a: MiniVec<isize> = mini_vec![1, 2, 3, 4, 5];
  let b: MiniVec<isize> = mini_vec![6, 7, 8, 9, 0];

  let mut v: MiniVec<isize> = a;

  v.extend_from_slice(&b);

  assert_eq!(v, [1, 2, 3, 4, 5, 6, 7, 8, 9, 0]);
}

#[test]
fn test_extend_ref() {
  let mut v = mini_vec![1, 2];
  v.extend(&[3, 4, 5]);

  assert_eq!(v.len(), 5);
  assert_eq!(v, [1, 2, 3, 4, 5]);

  let w = mini_vec![6, 7];
  v.extend(&w);

  assert_eq!(v.len(), 7);
  assert_eq!(v, [1, 2, 3, 4, 5, 6, 7]);
}

#[test]
fn test_slice_from_ref() {
  let values = mini_vec![1, 2, 3, 4, 5];
  let slice = &values[1..3];

  assert_eq!(slice, [2, 3]);
}

#[test]
fn test_slice_from_mut() {
  let mut values = mini_vec![1, 2, 3, 4, 5];
  {
    let slice = &mut values[2..];
    assert!(slice == [3, 4, 5]);
    for p in slice {
      *p += 2;
    }
  }

  assert!(values == [1, 2, 5, 6, 7]);
}

#[test]
fn test_slice_to_mut() {
  let mut values = mini_vec![1, 2, 3, 4, 5];
  {
    let slice = &mut values[..2];
    assert!(slice == [1, 2]);
    for p in slice {
      *p += 1;
    }
  }

  assert!(values == [2, 3, 3, 4, 5]);
}

#[test]
fn test_split_at_mut() {
  let mut values = mini_vec![1, 2, 3, 4, 5];
  {
    let (left, right) = values.split_at_mut(2);
    {
      let left: &[_] = left;
      assert!(&left[..left.len()] == &[1, 2]);
    }
    for p in left {
      *p += 1;
    }

    {
      let right: &[_] = right;
      assert!(&right[..right.len()] == &[3, 4, 5]);
    }
    for p in right {
      *p += 2;
    }
  }

  assert_eq!(values, [2, 3, 5, 6, 7]);
}

#[test]
fn test_clone() {
  let v: MiniVec<i32> = mini_vec![];
  let w = mini_vec![1, 2, 3];

  assert_eq!(v, v.clone());

  let z = w.clone();
  assert_eq!(w, z);
  // they should be disjoint in memory.
  assert!(w.as_ptr() != z.as_ptr())
}

#[test]
fn test_clone_from() {
  let mut v = mini_vec![];
  let three: MiniVec<Box<_>> = mini_vec![Box::new(1), Box::new(2), Box::new(3)];
  let two: MiniVec<Box<_>> = mini_vec![Box::new(4), Box::new(5)];
  // zero, long
  v.clone_from(&three);
  assert_eq!(v, three);

  // equal
  v.clone_from(&three);
  assert_eq!(v, three);

  // long, short
  v.clone_from(&two);
  assert_eq!(v, two);

  // short, long
  v.clone_from(&three);
  assert_eq!(v, three)
}

#[test]
fn test_retain() {
  let mut vec = mini_vec![1, 2, 3, 4];
  vec.retain(|&x| x % 2 == 0);
  assert_eq!(vec, [2, 4]);
}

#[test]
fn test_retain_pred_panic_with_hole() {
  let v = (0..5).map(Rc::new).collect::<MiniVec<_>>();
  catch_unwind(AssertUnwindSafe(|| {
    let mut v = v.clone();
    v.retain(|r| match **r {
      0 => true,
      1 => false,
      2 => true,
      _ => panic!(),
    });
  }))
  .unwrap_err();
  // Everything is dropped when predicate panicked.
  assert!(v.iter().all(|r| Rc::strong_count(r) == 1));
}

#[test]
fn test_retain_pred_panic_no_hole() {
  let v = (0..5).map(Rc::new).collect::<MiniVec<_>>();
  catch_unwind(AssertUnwindSafe(|| {
    let mut v = v.clone();
    v.retain(|r| match **r {
      0 | 1 | 2 => true,
      _ => panic!(),
    });
  }))
  .unwrap_err();
  // Everything is dropped when predicate panicked.
  assert!(v.iter().all(|r| Rc::strong_count(r) == 1));
}

#[test]
fn test_retain_drop_panic() {
  struct Wrap(Rc<i32>);

  impl Drop for Wrap {
    fn drop(&mut self) {
      if *self.0 == 3 {
        panic!();
      }
    }
  }

  let v = (0..5).map(|x| Rc::new(x)).collect::<MiniVec<_>>();
  catch_unwind(AssertUnwindSafe(|| {
    let mut v = v.iter().map(|r| Wrap(r.clone())).collect::<MiniVec<_>>();
    v.retain(|w| match *w.0 {
      0 => true,
      1 => false,
      2 => true,
      3 => false, // Drop panic.
      _ => true,
    });
  }))
  .unwrap_err();
  // Other elements are dropped when `drop` of one element panicked.
  // The panicked wrapper also has its Rc dropped.
  assert!(v.iter().all(|r| Rc::strong_count(r) == 1));
}

#[test]
fn test_dedup() {
  fn case(a: MiniVec<i32>, b: MiniVec<i32>) {
    let mut v = a;
    v.dedup();
    assert_eq!(v, b);
  }
  case(mini_vec![], mini_vec![]);
  case(mini_vec![1], mini_vec![1]);
  case(mini_vec![1, 1], mini_vec![1]);
  case(mini_vec![1, 2, 3], mini_vec![1, 2, 3]);
  case(mini_vec![1, 1, 2, 3], mini_vec![1, 2, 3]);
  case(mini_vec![1, 2, 2, 3], mini_vec![1, 2, 3]);
  case(mini_vec![1, 2, 3, 3], mini_vec![1, 2, 3]);
  case(mini_vec![1, 1, 2, 2, 2, 3, 3], mini_vec![1, 2, 3]);
}

#[test]
fn test_dedup_by_key() {
  fn case(a: MiniVec<i32>, b: MiniVec<i32>) {
    let mut v = a;
    v.dedup_by_key(|i| *i / 10);
    assert_eq!(v, b);
  }
  case(mini_vec![], mini_vec![]);
  case(mini_vec![10], mini_vec![10]);
  case(mini_vec![10, 11], mini_vec![10]);
  case(mini_vec![10, 20, 30], mini_vec![10, 20, 30]);
  case(mini_vec![10, 11, 20, 30], mini_vec![10, 20, 30]);
  case(mini_vec![10, 20, 21, 30], mini_vec![10, 20, 30]);
  case(mini_vec![10, 20, 30, 31], mini_vec![10, 20, 30]);
  case(mini_vec![10, 11, 20, 21, 22, 30, 31], mini_vec![10, 20, 30]);
}

#[test]
fn test_dedup_by() {
  let mut vec = mini_vec!["foo", "bar", "Bar", "baz", "bar"];
  vec.dedup_by(|a, b| a.eq_ignore_ascii_case(b));

  assert_eq!(vec, ["foo", "bar", "baz", "bar"]);

  let mut vec = mini_vec![("foo", 1), ("foo", 2), ("bar", 3), ("bar", 4), ("bar", 5)];
  vec.dedup_by(|a, b| {
    a.0 == b.0 && {
      b.1 += a.1;
      true
    }
  });

  assert_eq!(vec, [("foo", 3), ("bar", 12)]);
}

#[test]
fn test_dedup_unique() {
  let mut v0: MiniVec<Box<_>> = mini_vec![Box::new(1), Box::new(1), Box::new(2), Box::new(3)];
  v0.dedup();
  let mut v1: MiniVec<Box<_>> = mini_vec![Box::new(1), Box::new(2), Box::new(2), Box::new(3)];
  v1.dedup();
  let mut v2: MiniVec<Box<_>> = mini_vec![Box::new(1), Box::new(2), Box::new(3), Box::new(3)];
  v2.dedup();
  // If the boxed pointers were leaked or otherwise misused, valgrind
  // and/or rt should raise errors.
}

// #[test]
// fn zero_sized_values() {
//     let mut v = MiniVec::new();
//     assert_eq!(v.len(), 0);
//     v.push(());
//     assert_eq!(v.len(), 1);
//     v.push(());
//     assert_eq!(v.len(), 2);
//     assert_eq!(v.pop(), Some(()));
//     assert_eq!(v.pop(), Some(()));
//     assert_eq!(v.pop(), None);

//     assert_eq!(v.iter().count(), 0);
//     v.push(());
//     assert_eq!(v.iter().count(), 1);
//     v.push(());
//     assert_eq!(v.iter().count(), 2);

//     for &() in &v {}

//     assert_eq!(v.iter_mut().count(), 2);
//     v.push(());
//     assert_eq!(v.iter_mut().count(), 3);
//     v.push(());
//     assert_eq!(v.iter_mut().count(), 4);

//     for &mut () in &mut v {}
//     unsafe {
//         v.set_len(0);
//     }
//     assert_eq!(v.iter_mut().count(), 0);
// }

#[test]
fn test_partition() {
  assert_eq!(
    mini_vec![].into_iter().partition(|x: &i32| *x < 3),
    (mini_vec![], mini_vec![])
  );
  assert_eq!(
    mini_vec![1, 2, 3].into_iter().partition(|x| *x < 4),
    (mini_vec![1, 2, 3], mini_vec![])
  );
  assert_eq!(
    mini_vec![1, 2, 3].into_iter().partition(|x| *x < 2),
    (mini_vec![1], mini_vec![2, 3])
  );
  assert_eq!(
    mini_vec![1, 2, 3].into_iter().partition(|x| *x < 0),
    (mini_vec![], mini_vec![1, 2, 3])
  );
}

#[test]
fn test_zip_unzip() {
  let z1 = mini_vec![(1, 4), (2, 5), (3, 6)];

  let (left, right): (MiniVec<_>, MiniVec<_>) = z1.iter().cloned().unzip();

  assert_eq!((1, 4), (left[0], right[0]));
  assert_eq!((2, 5), (left[1], right[1]));
  assert_eq!((3, 6), (left[2], right[2]));
}

#[test]
fn test_cmp() {
  let x: &[isize] = &[1, 2, 3, 4, 5];
  let cmp: &[isize] = &[1, 2, 3, 4, 5];
  assert_eq!(&x[..], cmp);
  let cmp: &[isize] = &[3, 4, 5];
  assert_eq!(&x[2..], cmp);
  let cmp: &[isize] = &[1, 2, 3];
  assert_eq!(&x[..3], cmp);
  let cmp: &[isize] = &[2, 3, 4];
  assert_eq!(&x[1..4], cmp);

  let x: MiniVec<isize> = mini_vec![1, 2, 3, 4, 5];
  let cmp: &[isize] = &[1, 2, 3, 4, 5];
  assert_eq!(&x[..], cmp);
  let cmp: &[isize] = &[3, 4, 5];
  assert_eq!(&x[2..], cmp);
  let cmp: &[isize] = &[1, 2, 3];
  assert_eq!(&x[..3], cmp);
  let cmp: &[isize] = &[2, 3, 4];
  assert_eq!(&x[1..4], cmp);
}

#[test]
fn test_vec_truncate_drop() {
  static mut DROPS: u32 = 0;
  struct Elem(i32);
  impl Drop for Elem {
    fn drop(&mut self) {
      unsafe {
        DROPS += 1;
      }
    }
  }

  let mut v = mini_vec![Elem(1), Elem(2), Elem(3), Elem(4), Elem(5)];
  assert_eq!(unsafe { DROPS }, 0);
  v.truncate(3);
  assert_eq!(unsafe { DROPS }, 2);
  v.truncate(0);
  assert_eq!(unsafe { DROPS }, 5);
}

#[test]
#[should_panic]
fn test_vec_truncate_fail() {
  struct BadElem(i32);
  impl Drop for BadElem {
    fn drop(&mut self) {
      let BadElem(ref mut x) = *self;
      if *x == 0xbadbeef {
        panic!("BadElem panic: 0xbadbeef")
      }
    }
  }

  let mut v = mini_vec![BadElem(1), BadElem(2), BadElem(0xbadbeef), BadElem(4)];
  v.truncate(0);
}

#[test]
fn test_index() {
  let vec = mini_vec![1, 2, 3];
  assert!(vec[1] == 2);
}

#[test]
#[should_panic]
fn test_index_out_of_bounds() {
  let vec = mini_vec![1, 2, 3];
  let _ = vec[3];
}

#[test]
#[should_panic]
fn test_slice_out_of_bounds_1() {
  let x = mini_vec![1, 2, 3, 4, 5];
  &x[!0..];
}

#[test]
#[should_panic]
fn test_slice_out_of_bounds_2() {
  let x = mini_vec![1, 2, 3, 4, 5];
  &x[..6];
}

#[test]
#[should_panic]
fn test_slice_out_of_bounds_3() {
  let x = mini_vec![1, 2, 3, 4, 5];
  &x[!0..4];
}

#[test]
#[should_panic]
fn test_slice_out_of_bounds_4() {
  let x = mini_vec![1, 2, 3, 4, 5];
  &x[1..6];
}

#[test]
#[should_panic]
fn test_slice_out_of_bounds_5() {
  let x = mini_vec![1, 2, 3, 4, 5];
  &x[3..2];
}

#[test]
#[should_panic]
fn test_swap_remove_empty() {
  let mut vec = Vec::<i32>::new();
  vec.swap_remove(0);
}

#[test]
fn test_move_items() {
  let vec = mini_vec![1, 2, 3];
  let mut vec2 = mini_vec![];
  for i in vec {
    vec2.push(i);
  }
  assert_eq!(vec2, [1, 2, 3]);
}

#[test]
fn test_move_items_reverse() {
  let vec = mini_vec![1, 2, 3];
  let mut vec2 = mini_vec![];
  for i in vec.into_iter().rev() {
    vec2.push(i);
  }
  assert_eq!(vec2, [3, 2, 1]);
}

// #[test]
// fn test_move_items_zero_sized() {
//     let vec = mini_vec![(), (), ()];
//     let mut vec2 = mini_vec![];
//     for i in vec {
//         vec2.push(i);
//     }
//     assert_eq!(vec2, [(), (), ()]);
// }

#[test]
fn test_drain_empty_vec() {
  let mut vec: MiniVec<i32> = mini_vec![];
  let mut vec2: MiniVec<i32> = mini_vec![];
  for i in vec.drain(..) {
    vec2.push(i);
  }
  assert!(vec.is_empty());
  assert!(vec2.is_empty());
}

#[test]
fn test_drain_items() {
  let mut vec = mini_vec![1, 2, 3];
  let mut vec2 = mini_vec![];
  for i in vec.drain(..) {
    vec2.push(i);
  }
  assert_eq!(vec, []);
  assert_eq!(vec2, [1, 2, 3]);
}

#[test]
fn test_drain_items_reverse() {
  let mut vec = mini_vec![1, 2, 3];
  let mut vec2 = mini_vec![];
  for i in vec.drain(..).rev() {
    vec2.push(i);
  }
  assert_eq!(vec, []);
  assert_eq!(vec2, [3, 2, 1]);
}

// #[test]
// fn test_drain_items_zero_sized() {
//     let mut vec = mini_vec![(), (), ()];
//     let mut vec2 = mini_vec![];
//     for i in vec.drain(..) {
//         vec2.push(i);
//     }
//     assert_eq!(vec, []);
//     assert_eq!(vec2, [(), (), ()]);
// }

#[test]
#[should_panic]
fn test_drain_out_of_bounds() {
  let mut v = mini_vec![1, 2, 3, 4, 5];
  v.drain(5..6);
}

#[test]
fn test_drain_range() {
  let mut v = mini_vec![1, 2, 3, 4, 5];
  for _ in v.drain(4..) {}
  assert_eq!(v, &[1, 2, 3, 4]);

  let mut v: MiniVec<_> = (1..6).map(|x| x.to_string()).collect();
  for _ in v.drain(1..4) {}
  assert_eq!(v, &[1.to_string(), 5.to_string()]);

  let mut v: MiniVec<_> = (1..6).map(|x| x.to_string()).collect();
  for _ in v.drain(1..4).rev() {}
  assert_eq!(v, &[1.to_string(), 5.to_string()]);

  // let mut v: MiniVec<_> = mini_vec![(); 5];
  // for _ in v.drain(1..4).rev() {}
  // assert_eq!(v, &[(), ()]);
}

#[test]
fn test_drain_inclusive_range() {
  let mut v = mini_vec!['a', 'b', 'c', 'd', 'e'];
  for _ in v.drain(1..=3) {}
  assert_eq!(v, &['a', 'e']);

  let mut v: MiniVec<_> = (0..=5).map(|x| x.to_string()).collect();
  for _ in v.drain(1..=5) {}
  assert_eq!(v, &["0".to_string()]);

  let mut v: MiniVec<String> = (0..=5).map(|x| x.to_string()).collect();
  for _ in v.drain(0..=5) {}
  assert_eq!(v, Vec::<String>::new());

  let mut v: MiniVec<_> = (0..=5).map(|x| x.to_string()).collect();
  for _ in v.drain(0..=3) {}
  assert_eq!(v, &["4".to_string(), "5".to_string()]);

  let mut v: MiniVec<_> = (0..=1).map(|x| x.to_string()).collect();
  for _ in v.drain(..=0) {}
  assert_eq!(v, &["1".to_string()]);
}

// #[test]
// fn test_drain_max_vec_size() {
//     let mut v = Vec::<()>::with_capacity(usize::MAX);
//     unsafe {
//         v.set_len(usize::MAX);
//     }
//     for _ in v.drain(usize::MAX - 1..) {}
//     assert_eq!(v.len(), usize::MAX - 1);

//     let mut v = Vec::<()>::with_capacity(usize::MAX);
//     unsafe {
//         v.set_len(usize::MAX);
//     }
//     for _ in v.drain(usize::MAX - 1..=usize::MAX - 1) {}
//     assert_eq!(v.len(), usize::MAX - 1);
// }

// #[test]
// #[should_panic]
// fn test_drain_index_overflow() {
//     let mut v = MiniVec::<()>::with_capacity(usize::MAX);
//     unsafe {
//         v.set_len(usize::MAX);
//     }
//     v.drain(0..=usize::MAX);
// }

#[test]
#[should_panic]
fn test_drain_inclusive_out_of_bounds() {
  let mut v = mini_vec![1, 2, 3, 4, 5];
  v.drain(5..=5);
}

#[test]
#[should_panic]
fn test_drain_start_overflow() {
  let mut v = mini_vec![1, 2, 3];
  v.drain((Excluded(usize::MAX), Included(0)));
}

#[test]
#[should_panic]
fn test_drain_end_overflow() {
  let mut v = mini_vec![1, 2, 3];
  v.drain((Included(0), Included(usize::MAX)));
}

#[test]
fn test_drain_leak() {
  static mut DROPS: i32 = 0;

  #[derive(Debug, PartialEq)]
  struct D(u32, bool);

  impl Drop for D {
    fn drop(&mut self) {
      unsafe {
        DROPS += 1;
      }

      if self.1 {
        panic!("panic in `drop`");
      }
    }
  }

  let mut v = mini_vec![
    D(0, false),
    D(1, false),
    D(2, false),
    D(3, false),
    D(4, true),
    D(5, false),
    D(6, false),
  ];

  catch_unwind(AssertUnwindSafe(|| {
    v.drain(2..=5);
  }))
  .ok();

  assert_eq!(unsafe { DROPS }, 4);
  assert_eq!(v, mini_vec![D(0, false), D(1, false), D(6, false),]);
}

#[test]
fn test_splice() {
  let mut v = mini_vec![1, 2, 3, 4, 5];
  let a = [10, 11, 12];
  v.splice(2..4, a.iter().cloned());
  assert_eq!(v, &[1, 2, 10, 11, 12, 5]);
  v.splice(1..3, Some(20));
  assert_eq!(v, &[1, 20, 11, 12, 5]);
}

#[test]
fn test_splice_inclusive_range() {
  let mut v = mini_vec![1, 2, 3, 4, 5];
  let a = [10, 11, 12];
  let t1: MiniVec<_> = v.splice(2..=3, a.iter().cloned()).collect();
  assert_eq!(v, &[1, 2, 10, 11, 12, 5]);
  assert_eq!(t1, &[3, 4]);
  let t2: MiniVec<_> = v.splice(1..=2, Some(20)).collect();
  assert_eq!(v, &[1, 20, 11, 12, 5]);
  assert_eq!(t2, &[2, 10]);
}

#[test]
#[should_panic]
fn test_splice_out_of_bounds() {
  let mut v = mini_vec![1, 2, 3, 4, 5];
  let a = [10, 11, 12];
  v.splice(5..6, a.iter().cloned());
}

#[test]
#[should_panic]
fn test_splice_inclusive_out_of_bounds() {
  let mut v = mini_vec![1, 2, 3, 4, 5];
  let a = [10, 11, 12];
  v.splice(5..=5, a.iter().cloned());
}

// #[test]
// fn test_splice_items_zero_sized() {
//     let mut vec = mini_vec![(), (), ()];
//     let vec2 = mini_vec![];
//     let t: MiniVec<_> = vec.splice(1..2, vec2.iter().cloned()).collect();
//     assert_eq!(vec, &[(), ()]);
//     assert_eq!(t, &[()]);
// }

#[test]
fn test_splice_unbounded() {
  let mut vec = mini_vec![1, 2, 3, 4, 5];
  let t: MiniVec<_> = vec.splice(.., None).collect();
  assert_eq!(vec, &[]);
  assert_eq!(t, &[1, 2, 3, 4, 5]);
}

#[test]
fn test_splice_forget() {
  let mut v = mini_vec![1, 2, 3, 4, 5];
  let a = [10, 11, 12];
  std::mem::forget(v.splice(2..4, a.iter().cloned()));
  assert_eq!(v, &[1, 2]);
}

// #[test]
// fn test_into_boxed_slice() {
//     let xs = mini_vec![1, 2, 3];
//     let ys = xs.into_boxed_slice();
//     assert_eq!(&*ys, [1, 2, 3]);
// }

#[test]
fn test_append() {
  let mut vec = mini_vec![1, 2, 3];
  let mut vec2 = mini_vec![4, 5, 6];
  vec.append(&mut vec2);
  assert_eq!(vec, [1, 2, 3, 4, 5, 6]);
  assert_eq!(vec2, []);
}

#[test]
fn test_split_off() {
  let mut vec = mini_vec![1, 2, 3, 4, 5, 6];
  let orig_capacity = vec.capacity();
  let vec2 = vec.split_off(4);
  assert_eq!(vec, [1, 2, 3, 4]);
  assert_eq!(vec2, [5, 6]);
  assert_eq!(vec.capacity(), orig_capacity);
}

#[test]
fn test_split_off_take_all() {
  let mut vec = mini_vec![1, 2, 3, 4, 5, 6];
  let orig_ptr = vec.as_ptr();
  let orig_capacity = vec.capacity();
  let vec2 = vec.split_off(0);
  assert_eq!(vec, []);
  assert_eq!(vec2, [1, 2, 3, 4, 5, 6]);
  assert_eq!(vec.capacity(), orig_capacity);
  assert_eq!(vec2.as_ptr(), orig_ptr);
}

#[test]
fn test_into_iter_as_slice() {
  let vec = mini_vec!['a', 'b', 'c'];
  let mut into_iter = vec.into_iter();
  assert_eq!(into_iter.as_slice(), &['a', 'b', 'c']);
  let _ = into_iter.next().unwrap();
  assert_eq!(into_iter.as_slice(), &['b', 'c']);
  let _ = into_iter.next().unwrap();
  let _ = into_iter.next().unwrap();
  assert_eq!(into_iter.as_slice(), &[]);
}

#[test]
fn test_into_iter_as_mut_slice() {
  let vec = mini_vec!['a', 'b', 'c'];
  let mut into_iter = vec.into_iter();
  assert_eq!(into_iter.as_slice(), &['a', 'b', 'c']);
  into_iter.as_mut_slice()[0] = 'x';
  into_iter.as_mut_slice()[1] = 'y';
  assert_eq!(into_iter.next().unwrap(), 'x');
  assert_eq!(into_iter.as_slice(), &['y', 'c']);
}

#[test]
fn test_into_iter_debug() {
  let vec = mini_vec!['a', 'b', 'c'];
  let into_iter = vec.into_iter();
  let debug = format!("{:?}", into_iter);
  assert_eq!(debug, "MiniVec::IntoIter(['a', 'b', 'c'])");
}

#[test]
fn test_into_iter_count() {
  assert_eq!(mini_vec![1, 2, 3].into_iter().count(), 3);
}

#[test]
fn test_into_iter_clone() {
  fn iter_equal<I: Iterator<Item = i32>>(it: I, slice: &[i32]) {
    let v: MiniVec<i32> = it.collect();
    assert_eq!(&v[..], slice);
  }
  let mut it = mini_vec![1, 2, 3].into_iter();
  iter_equal(it.clone(), &[1, 2, 3]);
  assert_eq!(it.next(), Some(1));
  let mut it = it.rev();
  iter_equal(it.clone(), &[3, 2]);
  assert_eq!(it.next(), Some(3));
  iter_equal(it.clone(), &[2]);
  assert_eq!(it.next(), Some(2));
  iter_equal(it.clone(), &[]);
  assert_eq!(it.next(), None);
}

#[test]
fn test_into_iter_leak() {
  static mut DROPS: i32 = 0;

  struct D(bool);

  impl Drop for D {
    fn drop(&mut self) {
      unsafe {
        DROPS += 1;
      }

      if self.0 {
        panic!("panic in `drop`");
      }
    }
  }

  let v = mini_vec![D(false), D(true), D(false)];

  catch_unwind(move || drop(v.into_iter())).ok();

  assert_eq!(unsafe { DROPS }, 3);
}

// #[test]
// fn test_from_iter_specialization() {
//     let src: MiniVec<usize> = mini_vec![0usize; 1];
//     let srcptr = src.as_ptr();
//     let sink = src.into_iter().collect::<MiniVec<_>>();
//     let sinkptr = sink.as_ptr();
//     assert_eq!(srcptr, sinkptr);
// }

// #[test]
// fn test_from_iter_partially_drained_in_place_specialization() {
//     let src: MiniVec<usize> = mini_vec![0usize; 10];
//     let srcptr = src.as_ptr();
//     let mut iter = src.into_iter();
//     iter.next();
//     iter.next();
//     let sink = iter.collect::<MiniVec<_>>();
//     let sinkptr = sink.as_ptr();
//     assert_eq!(srcptr, sinkptr);
// }

// #[test]
// fn test_from_iter_specialization_with_iterator_adapters() {
//     fn assert_in_place_trait<T: InPlaceIterable>(_: &T) {}
//     let src: MiniVec<usize> = mini_vec![0usize; 256];
//     let srcptr = src.as_ptr();
//     let iter = src
//         .into_iter()
//         .enumerate()
//         .map(|i| i.0 + i.1)
//         .zip(std::iter::repeat(1usize))
//         .map(|(a, b)| a + b)
//         .map_while(Option::Some)
//         .peekable()
//         .skip(1)
//         .map(|e| std::num::NonZeroUsize::new(e));
//     assert_in_place_trait(&iter);
//     let sink = iter.collect::<MiniVec<_>>();
//     let sinkptr = sink.as_ptr();
//     assert_eq!(srcptr, sinkptr as *const usize);
// }

// #[test]
// fn test_from_iter_specialization_head_tail_drop() {
//     let drop_count: MiniVec<_> = (0..=2).map(|_| Rc::new(())).collect();
//     let src: MiniVec<_> = drop_count.iter().cloned().collect();
//     let srcptr = src.as_ptr();
//     let iter = src.into_iter();
//     let sink: MiniVec<_> = iter.skip(1).take(1).collect();
//     let sinkptr = sink.as_ptr();
//     assert_eq!(srcptr, sinkptr, "specialization was applied");
//     assert_eq!(Rc::strong_count(&drop_count[0]), 1, "front was dropped");
//     assert_eq!(
//         Rc::strong_count(&drop_count[1]),
//         2,
//         "one element was collected"
//     );
//     assert_eq!(Rc::strong_count(&drop_count[2]), 1, "tail was dropped");
//     assert_eq!(sink.len(), 1);
// }

#[test]
fn test_from_iter_specialization_panic_drop() {
  let drop_count: MiniVec<_> = (0..=2).map(|_| Rc::new(())).collect();
  let src: MiniVec<_> = drop_count.iter().cloned().collect();
  let iter = src.into_iter();

  let _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
    let _ = iter
      .enumerate()
      .filter_map(|(i, e)| {
        if i == 1 {
          std::panic!("aborting iteration");
        }
        Some(e)
      })
      .collect::<MiniVec<_>>();
  }));

  assert!(
    drop_count
      .iter()
      .map(Rc::strong_count)
      .all(|count| count == 1),
    "all items were dropped once"
  );
}

// #[test]
// fn test_cow_from() {
//     let borrowed: &[_] = &["borrowed", "(slice)"];
//     let owned = mini_vec!["owned", "(vec)"];
//     match (Cow::from(owned.clone()), Cow::from(borrowed)) {
//         (Cow::Owned(o), Cow::Borrowed(b)) => assert!(o == owned && b == borrowed),
//         _ => panic!("invalid `Cow::from`"),
//     }
// }

// #[test]
// fn test_from_cow() {
//     let borrowed: &[_] = &["borrowed", "(slice)"];
//     let owned = mini_vec!["owned", "(vec)"];
//     assert_eq!(
//         Vec::from(Cow::Borrowed(borrowed)),
//         mini_vec!["borrowed", "(slice)"]
//     );
//     assert_eq!(Vec::from(Cow::Owned(owned)), mini_vec!["owned", "(vec)"]);
// }

#[allow(dead_code)]
fn assert_covariance() {
  fn drain<'new>(d: minivec::Drain<'static, &'static str>) -> minivec::Drain<'new, &'new str> {
    d
  }
  fn into_iter<'new>(i: minivec::IntoIter<&'static str>) -> minivec::IntoIter<&'new str> {
    i
  }
}

#[test]
fn from_into_inner() {
  let vec = mini_vec![1, 2, 3];
  // let ptr = vec.as_ptr();
  let vec = vec.into_iter().collect::<MiniVec<_>>();
  assert_eq!(vec, [1, 2, 3]);
  // assert_eq!(vec.as_ptr(), ptr);

  let ptr = &vec[1] as *const _;
  let mut it = vec.into_iter();
  it.next().unwrap();
  let vec = it.collect::<MiniVec<_>>();
  assert_eq!(vec, [2, 3]);
  assert!(ptr != vec.as_ptr());
}

#[test]
fn overaligned_allocations() {
  #[repr(align(256))]
  struct Foo(usize);
  let mut v = mini_vec![Foo(273)];
  for i in 0..0x1000 {
    v.reserve_exact(i);
    assert!(v[0].0 == 273);
    assert!(v.as_ptr() as usize & 0xff == 0);
    v.shrink_to_fit();
    assert!(v[0].0 == 273);
    assert!(v.as_ptr() as usize & 0xff == 0);
  }
}

#[test]
fn drain_filter_empty() {
  let mut vec: MiniVec<i32> = mini_vec![];

  {
    let mut iter = vec.drain_filter(|_| true);
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.size_hint(), (0, Some(0)));
  }
  assert_eq!(vec.len(), 0);
  assert_eq!(vec, mini_vec![]);
}

// #[test]
// fn drain_filter_zst() {
//     let mut vec = mini_vec![(), (), (), (), ()];
//     let initial_len = vec.len();
//     let mut count = 0;
//     {
//         let mut iter = vec.drain_filter(|_| true);
//         assert_eq!(iter.size_hint(), (0, Some(initial_len)));
//         while let Some(_) = iter.next() {
//             count += 1;
//             assert_eq!(iter.size_hint(), (0, Some(initial_len - count)));
//         }
//         assert_eq!(iter.size_hint(), (0, Some(0)));
//         assert_eq!(iter.next(), None);
//         assert_eq!(iter.size_hint(), (0, Some(0)));
//     }

//     assert_eq!(count, initial_len);
//     assert_eq!(vec.len(), 0);
//     assert_eq!(vec, mini_vec![]);
// }

#[test]
fn drain_filter_false() {
  let mut vec = mini_vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

  let initial_len = vec.len();
  let mut count = 0;
  {
    let mut iter = vec.drain_filter(|_| false);
    assert_eq!(iter.size_hint(), (0, Some(initial_len)));
    for _ in iter.by_ref() {
      count += 1;
    }
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.size_hint(), (0, Some(0)));
  }

  assert_eq!(count, 0);
  assert_eq!(vec.len(), initial_len);
  assert_eq!(vec, mini_vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
}

#[test]
fn drain_filter_true() {
  let mut vec = mini_vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

  let initial_len = vec.len();
  let mut count = 0;
  {
    let mut iter = vec.drain_filter(|_| true);
    assert_eq!(iter.size_hint(), (0, Some(initial_len)));
    while let Some(_) = iter.next() {
      count += 1;
      assert_eq!(iter.size_hint(), (0, Some(initial_len - count)));
    }
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.size_hint(), (0, Some(0)));
  }

  assert_eq!(count, initial_len);
  assert_eq!(vec.len(), 0);
  assert_eq!(vec, mini_vec![]);
}

#[test]
fn drain_filter_complex() {
  {
    //                [+xxx++++++xxxxx++++x+x++]
    let mut vec = mini_vec![
      1, 2, 4, 6, 7, 9, 11, 13, 15, 17, 18, 20, 22, 24, 26, 27, 29, 31, 33, 34, 35, 36, 37, 39,
    ];

    let removed = vec.drain_filter(|x| *x % 2 == 0).collect::<MiniVec<_>>();
    assert_eq!(removed.len(), 10);
    assert_eq!(removed, mini_vec![2, 4, 6, 18, 20, 22, 24, 26, 34, 36]);

    assert_eq!(vec.len(), 14);
    assert_eq!(
      vec,
      mini_vec![1, 7, 9, 11, 13, 15, 17, 27, 29, 31, 33, 35, 37, 39]
    );
  }

  {
    //                [xxx++++++xxxxx++++x+x++]
    let mut vec = mini_vec![
      2, 4, 6, 7, 9, 11, 13, 15, 17, 18, 20, 22, 24, 26, 27, 29, 31, 33, 34, 35, 36, 37, 39,
    ];

    let removed = vec.drain_filter(|x| *x % 2 == 0).collect::<MiniVec<_>>();
    assert_eq!(removed.len(), 10);
    assert_eq!(removed, mini_vec![2, 4, 6, 18, 20, 22, 24, 26, 34, 36]);

    assert_eq!(vec.len(), 13);
    assert_eq!(
      vec,
      mini_vec![7, 9, 11, 13, 15, 17, 27, 29, 31, 33, 35, 37, 39]
    );
  }

  {
    //                [xxx++++++xxxxx++++x+x]
    let mut vec =
      mini_vec![2, 4, 6, 7, 9, 11, 13, 15, 17, 18, 20, 22, 24, 26, 27, 29, 31, 33, 34, 35, 36,];

    let removed = vec.drain_filter(|x| *x % 2 == 0).collect::<MiniVec<_>>();
    assert_eq!(removed.len(), 10);
    assert_eq!(removed, mini_vec![2, 4, 6, 18, 20, 22, 24, 26, 34, 36]);

    assert_eq!(vec.len(), 11);
    assert_eq!(vec, mini_vec![7, 9, 11, 13, 15, 17, 27, 29, 31, 33, 35]);
  }

  {
    //                [xxxxxxxxxx+++++++++++]
    let mut vec = mini_vec![2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 1, 3, 5, 7, 9, 11, 13, 15, 17, 19,];

    let removed = vec.drain_filter(|x| *x % 2 == 0).collect::<MiniVec<_>>();
    assert_eq!(removed.len(), 10);
    assert_eq!(removed, mini_vec![2, 4, 6, 8, 10, 12, 14, 16, 18, 20]);

    assert_eq!(vec.len(), 10);
    assert_eq!(vec, mini_vec![1, 3, 5, 7, 9, 11, 13, 15, 17, 19]);
  }

  {
    //                [+++++++++++xxxxxxxxxx]
    let mut vec = mini_vec![1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20,];

    let removed = vec.drain_filter(|x| *x % 2 == 0).collect::<MiniVec<_>>();
    assert_eq!(removed.len(), 10);
    assert_eq!(removed, mini_vec![2, 4, 6, 8, 10, 12, 14, 16, 18, 20]);

    assert_eq!(vec.len(), 10);
    assert_eq!(vec, mini_vec![1, 3, 5, 7, 9, 11, 13, 15, 17, 19]);
  }
}

// FIXME: re-enable emscripten once it can unwind again
#[test]
#[cfg(not(target_os = "emscripten"))]
fn drain_filter_consumed_panic() {
  use std::rc::Rc;
  use std::sync::Mutex;

  struct Check {
    index: usize,
    drop_counts: Rc<Mutex<MiniVec<usize>>>,
  }

  impl Drop for Check {
    fn drop(&mut self) {
      self.drop_counts.lock().unwrap()[self.index] += 1;
      println!("drop: {}", self.index);
    }
  }

  let check_count = 10;
  let drop_counts = Rc::new(Mutex::new(mini_vec![0_usize; check_count]));
  let mut data: MiniVec<Check> = (0..check_count)
    .map(|index| Check {
      index,
      drop_counts: Rc::clone(&drop_counts),
    })
    .collect();

  let _ = std::panic::catch_unwind(move || {
    let filter = |c: &mut Check| {
      if c.index == 2 {
        panic!("panic at index: {}", c.index);
      }
      // Verify that if the filter could panic again on another element
      // that it would not cause a double panic and all elements of the
      // vec would still be dropped exactly once.
      if c.index == 4 {
        panic!("panic at index: {}", c.index);
      }
      c.index < 6
    };
    let drain = data.drain_filter(filter);

    // NOTE: The DrainFilter is explicitly consumed
    drain.for_each(drop);
  });

  let drop_counts = drop_counts.lock().unwrap();
  assert_eq!(check_count, drop_counts.len());

  for (index, count) in drop_counts.iter().cloned().enumerate() {
    assert_eq!(
      1, count,
      "unexpected drop count at index: {} (count: {})",
      index, count
    );
  }
}

// FIXME: Re-enable emscripten once it can catch panics
#[test]
#[cfg(not(target_os = "emscripten"))]
fn drain_filter_unconsumed_panic() {
  use std::rc::Rc;
  use std::sync::Mutex;

  struct Check {
    index: usize,
    drop_counts: Rc<Mutex<MiniVec<usize>>>,
  }

  impl Drop for Check {
    fn drop(&mut self) {
      self.drop_counts.lock().unwrap()[self.index] += 1;
      println!("drop: {}", self.index);
    }
  }

  let check_count = 10;
  let drop_counts = Rc::new(Mutex::new(mini_vec![0_usize; check_count]));
  let mut data: MiniVec<Check> = (0..check_count)
    .map(|index| Check {
      index,
      drop_counts: Rc::clone(&drop_counts),
    })
    .collect();

  let _ = std::panic::catch_unwind(move || {
    let filter = |c: &mut Check| {
      if c.index == 2 {
        panic!("panic at index: {}", c.index);
      }
      // Verify that if the filter could panic again on another element
      // that it would not cause a double panic and all elements of the
      // vec would still be dropped exactly once.
      if c.index == 4 {
        panic!("panic at index: {}", c.index);
      }
      c.index < 6
    };
    let _drain = data.drain_filter(filter);

    // NOTE: The DrainFilter is dropped without being consumed
  });

  let drop_counts = drop_counts.lock().unwrap();
  assert_eq!(check_count, drop_counts.len());

  for (index, count) in drop_counts.iter().cloned().enumerate() {
    assert_eq!(
      1, count,
      "unexpected drop count at index: {} (count: {})",
      index, count
    );
  }
}

#[test]
fn drain_filter_unconsumed() {
  let mut vec = mini_vec![1, 2, 3, 4];
  let drain = vec.drain_filter(|&mut x| x % 2 != 0);
  drop(drain);
  assert_eq!(vec, [2, 4]);
}

#[test]
fn test_reserve_exact() {
  // This is all the same as test_reserve

  let mut v = MiniVec::new();
  assert_eq!(v.capacity(), 0);

  v.reserve_exact(2);
  assert!(v.capacity() >= 2);

  for i in 0..16 {
    v.push(i);
  }

  assert!(v.capacity() >= 16);
  v.reserve_exact(16);
  assert!(v.capacity() >= 32);

  v.push(16);

  v.reserve_exact(16);
  assert!(v.capacity() >= 33)
}

#[test]
#[cfg_attr(miri, ignore)] // Miri does not support signalling OOM
#[cfg_attr(target_os = "android", ignore)] // Android used in CI has a broken dlmalloc
fn test_try_reserve() {
  use minivec::TryReserveErrorKind::*;

  // These are the interesting cases:
  // * exactly isize::MAX should never trigger a CapacityOverflow (can be OOM)
  // * > isize::MAX should always fail
  //    * On 16/32-bit should CapacityOverflow
  //    * On 64-bit should OOM
  // * overflow may trigger when adding `len` to `cap` (in number of elements)
  // * overflow may trigger when multiplying `new_cap` by size_of::<T> (to get bytes)

  const MAX_CAP: usize = isize::MAX as usize;
  const MAX_USIZE: usize = usize::MAX;

  // On 16/32-bit, we check that allocations don't exceed isize::MAX,
  // on 64-bit, we assume the OS will give an OOM for such a ridiculous size.
  // Any platform that succeeds for these requests is technically broken with
  // ptr::offset because LLVM is the worst.
  let guards_against_isize = usize::BITS < 64;

  {
    // Note: basic stuff is checked by test_reserve
    let mut empty_bytes: MiniVec<u8> = MiniVec::new();

    // Check isize::MAX doesn't count as an overflow
    if let Err(CapacityOverflow) = empty_bytes.try_reserve(MAX_CAP).map_err(|e| e.kind()) {
      panic!("isize::MAX shouldn't trigger an overflow!");
    }
    // Play it again, frank! (just to be sure)
    if let Err(CapacityOverflow) = empty_bytes.try_reserve(MAX_CAP).map_err(|e| e.kind()) {
      panic!("isize::MAX shouldn't trigger an overflow!");
    }

    if guards_against_isize {
      // Check isize::MAX + 1 does count as overflow
      if let Err(CapacityOverflow) = empty_bytes.try_reserve(MAX_CAP + 1).map_err(|e| e.kind()) {
      } else {
        panic!("isize::MAX + 1 should trigger an overflow!")
      }

      // Check usize::MAX does count as overflow
      if let Err(CapacityOverflow) = empty_bytes.try_reserve(MAX_USIZE).map_err(|e| e.kind()) {
      } else {
        panic!("usize::MAX should trigger an overflow!")
      }
    } else {
      // Check isize::MAX + 1 is an OOM
      if let Err(AllocError { .. }) = empty_bytes.try_reserve(MAX_CAP + 1).map_err(|e| e.kind()) {
      } else {
        panic!("isize::MAX + 1 should trigger an OOM!")
      }

      // Check usize::MAX is an OOM
      if let Err(AllocError { .. }) = empty_bytes.try_reserve(MAX_USIZE).map_err(|e| e.kind()) {
      } else {
        panic!("usize::MAX should trigger an OOM!")
      }
    }
  }

  {
    // Same basic idea, but with non-zero len
    let mut ten_bytes: MiniVec<u8> = mini_vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    if let Err(CapacityOverflow) = ten_bytes.try_reserve(MAX_CAP - 10).map_err(|e| e.kind()) {
      panic!("isize::MAX shouldn't trigger an overflow!");
    }
    if let Err(CapacityOverflow) = ten_bytes.try_reserve(MAX_CAP - 10).map_err(|e| e.kind()) {
      panic!("isize::MAX shouldn't trigger an overflow!");
    }
    if guards_against_isize {
      if let Err(CapacityOverflow) = ten_bytes.try_reserve(MAX_CAP - 9).map_err(|e| e.kind()) {
      } else {
        panic!("isize::MAX + 1 should trigger an overflow!");
      }
    } else {
      if let Err(AllocError { .. }) = ten_bytes.try_reserve(MAX_CAP - 9).map_err(|e| e.kind()) {
      } else {
        panic!("isize::MAX + 1 should trigger an OOM!")
      }
    }
    // Should always overflow in the add-to-len
    if let Err(CapacityOverflow) = ten_bytes.try_reserve(MAX_USIZE).map_err(|e| e.kind()) {
    } else {
      panic!("usize::MAX should trigger an overflow!")
    }
  }

  {
    // Same basic idea, but with interesting type size
    let mut ten_u32s: MiniVec<u32> = mini_vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    if let Err(CapacityOverflow) = ten_u32s.try_reserve(MAX_CAP / 4 - 10).map_err(|e| e.kind()) {
      panic!("isize::MAX shouldn't trigger an overflow!");
    }
    if let Err(CapacityOverflow) = ten_u32s.try_reserve(MAX_CAP / 4 - 10).map_err(|e| e.kind()) {
      panic!("isize::MAX shouldn't trigger an overflow!");
    }
    if guards_against_isize {
      if let Err(CapacityOverflow) = ten_u32s.try_reserve(MAX_CAP / 4 - 9).map_err(|e| e.kind()) {
      } else {
        panic!("isize::MAX + 1 should trigger an overflow!");
      }
    } else {
      if let Err(AllocError { .. }) = ten_u32s.try_reserve(MAX_CAP / 4 - 9).map_err(|e| e.kind()) {
      } else {
        panic!("isize::MAX + 1 should trigger an OOM!")
      }
    }
    // Should fail in the mul-by-size
    if let Err(CapacityOverflow) = ten_u32s.try_reserve(MAX_USIZE - 20).map_err(|e| e.kind()) {
    } else {
      panic!("usize::MAX should trigger an overflow!");
    }
  }
}

#[test]
#[cfg_attr(miri, ignore)] // Miri does not support signalling OOM
#[cfg_attr(target_os = "android", ignore)] // Android used in CI has a broken dlmalloc
fn test_try_reserve_exact() {
  use minivec::TryReserveErrorKind::*;

  // This is exactly the same as test_try_reserve with the method changed.
  // See that test for comments.

  const MAX_CAP: usize = isize::MAX as usize;
  const MAX_USIZE: usize = usize::MAX;

  let guards_against_isize = size_of::<usize>() < 8;

  {
    let mut empty_bytes: MiniVec<u8> = MiniVec::new();

    if let Err(CapacityOverflow) = empty_bytes.try_reserve_exact(MAX_CAP).map_err(|e| e.kind()) {
      panic!("isize::MAX shouldn't trigger an overflow!");
    }
    if let Err(CapacityOverflow) = empty_bytes.try_reserve_exact(MAX_CAP).map_err(|e| e.kind()) {
      panic!("isize::MAX shouldn't trigger an overflow!");
    }

    if guards_against_isize {
      if let Err(CapacityOverflow) = empty_bytes
        .try_reserve_exact(MAX_CAP + 1)
        .map_err(|e| e.kind())
      {
      } else {
        panic!("isize::MAX + 1 should trigger an overflow!")
      }

      if let Err(CapacityOverflow) = empty_bytes
        .try_reserve_exact(MAX_USIZE)
        .map_err(|e| e.kind())
      {
      } else {
        panic!("usize::MAX should trigger an overflow!")
      }
    } else {
      if let Err(AllocError { .. }) = empty_bytes
        .try_reserve_exact(MAX_CAP + 1)
        .map_err(|e| e.kind())
      {
      } else {
        panic!("isize::MAX + 1 should trigger an OOM!")
      }

      if let Err(AllocError { .. }) = empty_bytes
        .try_reserve_exact(MAX_USIZE)
        .map_err(|e| e.kind())
      {
      } else {
        panic!("usize::MAX should trigger an OOM!")
      }
    }
  }

  {
    let mut ten_bytes: MiniVec<u8> = mini_vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    if let Err(CapacityOverflow) = ten_bytes
      .try_reserve_exact(MAX_CAP - 10)
      .map_err(|e| e.kind())
    {
      panic!("isize::MAX shouldn't trigger an overflow!");
    }
    if let Err(CapacityOverflow) = ten_bytes
      .try_reserve_exact(MAX_CAP - 10)
      .map_err(|e| e.kind())
    {
      panic!("isize::MAX shouldn't trigger an overflow!");
    }
    if guards_against_isize {
      if let Err(CapacityOverflow) = ten_bytes
        .try_reserve_exact(MAX_CAP - 9)
        .map_err(|e| e.kind())
      {
      } else {
        panic!("isize::MAX + 1 should trigger an overflow!");
      }
    } else {
      if let Err(AllocError { .. }) = ten_bytes
        .try_reserve_exact(MAX_CAP - 9)
        .map_err(|e| e.kind())
      {
      } else {
        panic!("isize::MAX + 1 should trigger an OOM!")
      }
    }
    if let Err(CapacityOverflow) = ten_bytes.try_reserve_exact(MAX_USIZE).map_err(|e| e.kind()) {
    } else {
      panic!("usize::MAX should trigger an overflow!")
    }
  }

  {
    let mut ten_u32s: MiniVec<u32> = mini_vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    if let Err(CapacityOverflow) = ten_u32s
      .try_reserve_exact(MAX_CAP / 4 - 10)
      .map_err(|e| e.kind())
    {
      panic!("isize::MAX shouldn't trigger an overflow!");
    }
    if let Err(CapacityOverflow) = ten_u32s
      .try_reserve_exact(MAX_CAP / 4 - 10)
      .map_err(|e| e.kind())
    {
      panic!("isize::MAX shouldn't trigger an overflow!");
    }
    if guards_against_isize {
      if let Err(CapacityOverflow) = ten_u32s
        .try_reserve_exact(MAX_CAP / 4 - 9)
        .map_err(|e| e.kind())
      {
      } else {
        panic!("isize::MAX + 1 should trigger an overflow!");
      }
    } else {
      if let Err(AllocError { .. }) = ten_u32s
        .try_reserve_exact(MAX_CAP / 4 - 9)
        .map_err(|e| e.kind())
      {
      } else {
        panic!("isize::MAX + 1 should trigger an OOM!")
      }
    }
    if let Err(CapacityOverflow) = ten_u32s
      .try_reserve_exact(MAX_USIZE - 20)
      .map_err(|e| e.kind())
    {
    } else {
      panic!("usize::MAX should trigger an overflow!")
    }
  }
}

#[test]
fn test_stable_pointers() {
  /// Pull an element from the iterator, then drop it.
  /// Useful to cover both the `next` and `drop` paths of an iterator.
  fn next_then_drop<I: Iterator>(mut i: I) {
    i.next().unwrap();
    drop(i);
  }

  // Test that, if we reserved enough space, adding and removing elements does not
  // invalidate references into the vector (such as `v0`).  This test also
  // runs in Miri, which would detect such problems.
  // Note that this test does *not* constitute a stable guarantee that all these functions do not
  // reallocate! Only what is explicitly documented at
  // <https://doc.rust-lang.org/nightly/std/vec/struct.Vec.html#guarantees> is stably guaranteed.
  let mut v = MiniVec::with_capacity(128);
  v.push(13);

  // Laundering the lifetime -- we take care that `v` does not reallocate, so that's okay.
  let v0 = &mut v[0];
  let v0 = unsafe { &mut *(v0 as *mut _) };
  // Now do a bunch of things and occasionally use `v0` again to assert it is still valid.

  // Pushing/inserting and popping/removing
  v.push(1);
  v.push(2);
  v.insert(1, 1);
  assert_eq!(*v0, 13);
  v.remove(1);
  v.pop().unwrap();
  assert_eq!(*v0, 13);
  v.push(1);
  v.swap_remove(1);
  assert_eq!(v.len(), 2);
  v.swap_remove(1); // swap_remove the last element
  assert_eq!(*v0, 13);

  // Appending
  v.append(&mut mini_vec![27, 19]);
  assert_eq!(*v0, 13);

  // Extending
  v.extend_from_slice(&[1, 2]);
  v.extend(&[1, 2]); // `slice::Iter` (with `T: Copy`) specialization
  v.extend(mini_vec![2, 3]); // `vec::IntoIter` specialization
  v.extend(std::iter::once(3)); // `TrustedLen` specialization
  v.extend(std::iter::empty::<i32>()); // `TrustedLen` specialization with empty iterator
  v.extend(std::iter::once(3).filter(|_| true)); // base case
  v.extend(std::iter::once(&3)); // `cloned` specialization
  assert_eq!(*v0, 13);

  // Truncation
  v.truncate(2);
  assert_eq!(*v0, 13);

  // Resizing
  v.resize_with(v.len() + 10, || 42);
  assert_eq!(*v0, 13);
  v.resize_with(2, || panic!());
  assert_eq!(*v0, 13);

  // No-op reservation
  v.reserve(32);
  v.reserve_exact(32);
  assert_eq!(*v0, 13);

  // Partial draining
  v.resize_with(10, || 42);
  next_then_drop(v.drain(5..));
  assert_eq!(*v0, 13);

  // Splicing
  v.resize_with(10, || 42);
  next_then_drop(v.splice(5.., mini_vec![1, 2, 3, 4, 5])); // empty tail after range
  assert_eq!(*v0, 13);
  next_then_drop(v.splice(5..8, mini_vec![1])); // replacement is smaller than original range
  assert_eq!(*v0, 13);
  next_then_drop(v.splice(5..6, mini_vec![1; 10].into_iter().filter(|_| true))); // lower bound not exact
  assert_eq!(*v0, 13);

  // Smoke test that would fire even outside Miri if an actual relocation happened.
  *v0 -= 13;
  assert_eq!(v[0], 0);
}

// https://github.com/rust-lang/rust/pull/49496 introduced specialization based on:
//
// ```
// unsafe impl<T: ?Sized> IsZero for *mut T {
//     fn is_zero(&self) -> bool {
//         (*self).is_null()
//     }
// }
// ```
//
// … to call `RawVec::with_capacity_zeroed` for creating `Vec<*mut T>`,
// which is incorrect for fat pointers since `<*mut T>::is_null` only looks at the data component.
// That is, a fat pointer can be “null” without being made entirely of zero bits.
#[test]
fn vec_macro_repeating_null_raw_fat_pointer() {
  let raw_dyn = &mut (|| ()) as &mut dyn Fn() as *mut dyn Fn();
  let vtable = dbg!(ptr_metadata(raw_dyn));
  let null_raw_dyn = ptr_from_raw_parts(std::ptr::null_mut(), vtable);
  assert!(null_raw_dyn.is_null());

  let vec = mini_vec![null_raw_dyn; 1];
  dbg!(ptr_metadata(vec[0]));
  assert!(vec[0] == null_raw_dyn);

  // Polyfill for https://github.com/rust-lang/rfcs/pull/2580

  fn ptr_metadata(ptr: *mut dyn Fn()) -> *mut () {
    unsafe { std::mem::transmute::<*mut dyn Fn(), DynRepr>(ptr).vtable }
  }

  fn ptr_from_raw_parts(data: *mut (), vtable: *mut ()) -> *mut dyn Fn() {
    unsafe { std::mem::transmute::<DynRepr, *mut dyn Fn()>(DynRepr { data, vtable }) }
  }

  #[repr(C)]
  struct DynRepr {
    data: *mut (),
    vtable: *mut (),
  }
}

// This test will likely fail if you change the capacities used in
// `RawVec::grow_amortized`.
#[test]
fn test_push_growth_strategy() {
  // If the element size is 1, we jump from 0 to 8, then double.
  {
    let mut v1: MiniVec<u8> = mini_vec![];
    assert_eq!(v1.capacity(), 0);

    for _ in 0..8 {
      v1.push(0);
      assert_eq!(v1.capacity(), 8);
    }

    for _ in 8..16 {
      v1.push(0);
      assert_eq!(v1.capacity(), 16);
    }

    for _ in 16..32 {
      v1.push(0);
      assert_eq!(v1.capacity(), 32);
    }

    for _ in 32..64 {
      v1.push(0);
      assert_eq!(v1.capacity(), 64);
    }
  }

  // If the element size is 2..=1024, we jump from 0 to 4, then double.
  {
    let mut v2: MiniVec<u16> = mini_vec![];
    let mut v1024: MiniVec<[u8; 1024]> = mini_vec![];
    assert_eq!(v2.capacity(), 0);
    assert_eq!(v1024.capacity(), 0);

    for _ in 0..4 {
      v2.push(0);
      v1024.push([0; 1024]);
      assert_eq!(v2.capacity(), 4);
      assert_eq!(v1024.capacity(), 4);
    }

    for _ in 4..8 {
      v2.push(0);
      v1024.push([0; 1024]);
      assert_eq!(v2.capacity(), 8);
      assert_eq!(v1024.capacity(), 8);
    }

    for _ in 8..16 {
      v2.push(0);
      v1024.push([0; 1024]);
      assert_eq!(v2.capacity(), 16);
      assert_eq!(v1024.capacity(), 16);
    }

    for _ in 16..32 {
      v2.push(0);
      v1024.push([0; 1024]);
      assert_eq!(v2.capacity(), 32);
      assert_eq!(v1024.capacity(), 32);
    }

    for _ in 32..64 {
      v2.push(0);
      v1024.push([0; 1024]);
      assert_eq!(v2.capacity(), 64);
      assert_eq!(v1024.capacity(), 64);
    }
  }

  // If the element size is > 1024, we jump from 0 to 1, then double.
  {
    let mut v1025: MiniVec<[u8; 1025]> = mini_vec![];
    assert_eq!(v1025.capacity(), 0);

    for _ in 0..1 {
      v1025.push([0; 1025]);
      assert_eq!(v1025.capacity(), 1);
    }

    for _ in 1..2 {
      v1025.push([0; 1025]);
      assert_eq!(v1025.capacity(), 2);
    }

    for _ in 2..4 {
      v1025.push([0; 1025]);
      assert_eq!(v1025.capacity(), 4);
    }

    for _ in 4..8 {
      v1025.push([0; 1025]);
      assert_eq!(v1025.capacity(), 8);
    }

    for _ in 8..16 {
      v1025.push([0; 1025]);
      assert_eq!(v1025.capacity(), 16);
    }

    for _ in 16..32 {
      v1025.push([0; 1025]);
      assert_eq!(v1025.capacity(), 32);
    }

    for _ in 32..64 {
      v1025.push([0; 1025]);
      assert_eq!(v1025.capacity(), 64);
    }
  }
}

macro_rules! generate_assert_eq_vec_and_prim {
  ($name:ident<$B:ident>($type:ty)) => {
    fn $name<A: PartialEq<$B> + Debug, $B: Debug>(a: MiniVec<A>, b: $type) {
      assert!(a == b);
      assert_eq!(a, b);
    }
  };
}

generate_assert_eq_vec_and_prim! { assert_eq_vec_and_slice  <B>(&[B])   }
generate_assert_eq_vec_and_prim! { assert_eq_vec_and_array_3<B>([B; 3]) }

#[test]
fn partialeq_vec_and_prim() {
  assert_eq_vec_and_slice(mini_vec![1, 2, 3], &[1, 2, 3]);
  assert_eq_vec_and_array_3(mini_vec![1, 2, 3], [1, 2, 3]);
}

macro_rules! assert_partial_eq_valid {
  ($a2:expr, $a3:expr; $b2:expr, $b3: expr) => {
    assert!($a2 == $b2);
    assert!($a2 != $b3);
    assert!($a3 != $b2);
    assert!($a3 == $b3);
    assert_eq!($a2, $b2);
    assert_ne!($a2, $b3);
    assert_ne!($a3, $b2);
    assert_eq!($a3, $b3);
  };
}

#[test]
fn partialeq_vec_full() {
  let vec2: MiniVec<_> = mini_vec![1, 2];
  let vec3: MiniVec<_> = mini_vec![1, 2, 3];
  let slice2: &[_] = &[1, 2];
  let slice3: &[_] = &[1, 2, 3];
  let slicemut2: &[_] = &mut [1, 2];
  let slicemut3: &[_] = &mut [1, 2, 3];
  let array2: [_; 2] = [1, 2];
  let array3: [_; 3] = [1, 2, 3];
  let arrayref2: &[_; 2] = &[1, 2];
  let arrayref3: &[_; 3] = &[1, 2, 3];

  assert_partial_eq_valid!(vec2,vec3; vec2,vec3);
  assert_partial_eq_valid!(vec2,vec3; slice2,slice3);
  assert_partial_eq_valid!(vec2,vec3; slicemut2,slicemut3);
  assert_partial_eq_valid!(slice2,slice3; vec2,vec3);
  assert_partial_eq_valid!(slicemut2,slicemut3; vec2,vec3);
  assert_partial_eq_valid!(vec2,vec3; array2,array3);
  assert_partial_eq_valid!(vec2,vec3; arrayref2,arrayref3);
  assert_partial_eq_valid!(vec2,vec3; arrayref2[..],arrayref3[..]);
}

// #[test]
// fn test_vec_cycle() {
//     #[derive(Debug)]
//     struct C<'a> {
//         v: MiniVec<Cell<Option<&'a C<'a>>>>,
//     }

//     impl<'a> C<'a> {
//         fn new() -> C<'a> {
//             C { v: MiniVec::new() }
//         }
//     }

//     let mut c1 = C::new();
//     let mut c2 = C::new();
//     let mut c3 = C::new();

//     // Push
//     c1.v.push(Cell::new(None));
//     c1.v.push(Cell::new(None));

//     c2.v.push(Cell::new(None));
//     c2.v.push(Cell::new(None));

//     c3.v.push(Cell::new(None));
//     c3.v.push(Cell::new(None));

//     // Set
//     c1.v[0].set(Some(&c2));
//     c1.v[1].set(Some(&c3));

//     c2.v[0].set(Some(&c2));
//     c2.v[1].set(Some(&c3));

//     c3.v[0].set(Some(&c1));
//     c3.v[1].set(Some(&c2));
// }

// #[test]
// fn test_vec_cycle_wrapped() {
//     struct Refs<'a> {
//         v: MiniVec<Cell<Option<&'a C<'a>>>>,
//     }

//     struct C<'a> {
//         refs: Refs<'a>,
//     }

//     impl<'a> Refs<'a> {
//         fn new() -> Refs<'a> {
//             Refs { v: MiniVec::new() }
//         }
//     }

//     impl<'a> C<'a> {
//         fn new() -> C<'a> {
//             C { refs: Refs::new() }
//         }
//     }

//     let mut c1 = C::new();
//     let mut c2 = C::new();
//     let mut c3 = C::new();

//     c1.refs.v.push(Cell::new(None));
//     c1.refs.v.push(Cell::new(None));
//     c2.refs.v.push(Cell::new(None));
//     c2.refs.v.push(Cell::new(None));
//     c3.refs.v.push(Cell::new(None));
//     c3.refs.v.push(Cell::new(None));

//     c1.refs.v[0].set(Some(&c2));
//     c1.refs.v[1].set(Some(&c3));
//     c2.refs.v[0].set(Some(&c2));
//     c2.refs.v[1].set(Some(&c3));
//     c3.refs.v[0].set(Some(&c1));
//     c3.refs.v[1].set(Some(&c2));
// }

#[test]
fn test_zero_sized_vec_push() {
  const N: usize = 8;

  for len in 0..N {
    let mut tester = Vec::with_capacity(len);
    assert_eq!(tester.len(), 0);
    assert!(tester.capacity() >= len);
    for _ in 0..len {
      tester.push(());
    }
    assert_eq!(tester.len(), len);
    assert_eq!(tester.iter().count(), len);
    tester.clear();
  }
}

#[test]
fn test_vec_macro_repeat() {
  assert_eq!(mini_vec![1; 3], mini_vec![1, 1, 1]);
  assert_eq!(mini_vec![1; 2], mini_vec![1, 1]);
  assert_eq!(mini_vec![1; 1], mini_vec![1]);
  assert_eq!(mini_vec![1; 0], mini_vec![]);

  // from_elem syntax (see RFC 832)
  let el = Box::new(1);
  let n = 3;
  assert_eq!(
    mini_vec![el; n],
    mini_vec![Box::new(1), Box::new(1), Box::new(1)]
  );
}

#[test]
fn test_vec_swap() {
  let mut a: MiniVec<isize> = mini_vec![0, 1, 2, 3, 4, 5, 6];
  a.swap(2, 4);
  assert_eq!(a[2], 4);
  assert_eq!(a[4], 2);
  let mut n = 42;
  swap(&mut n, &mut a[0]);
  assert_eq!(a[0], 42);
  assert_eq!(n, 0);
}

// #[test]
// fn test_extend_from_within_spec() {
//     #[derive(Copy)]
//     struct CopyOnly;

//     impl Clone for CopyOnly {
//         fn clone(&self) -> Self {
//             panic!("extend_from_within must use specialization on copy");
//         }
//     }

//     mini_vec![CopyOnly, CopyOnly].extend_from_within(..);
// }

#[test]
fn test_extend_from_within_clone() {
  let mut v = mini_vec![
    String::from("sssss"),
    String::from("12334567890"),
    String::from("c"),
  ];
  v.extend_from_within(1..);

  assert_eq!(v, ["sssss", "12334567890", "c", "12334567890", "c"]);
}

#[test]
fn test_extend_from_within_complete_rande() {
  let mut v = mini_vec![0, 1, 2, 3];
  v.extend_from_within(..);

  assert_eq!(v, [0, 1, 2, 3, 0, 1, 2, 3]);
}

#[test]
fn test_extend_from_within_empty_rande() {
  let mut v = mini_vec![0, 1, 2, 3];
  v.extend_from_within(1..1);

  assert_eq!(v, [0, 1, 2, 3]);
}

#[test]
#[should_panic]
fn test_extend_from_within_out_of_rande() {
  let mut v = mini_vec![0, 1];
  v.extend_from_within(..3);
}

// #[test]
// fn test_extend_from_within_zst() {
//   let mut v = mini_vec![(); 8];
//   v.extend_from_within(3..7);

//   assert_eq!(v, [(); 12]);
// }

#[test]
fn test_extend_from_within_empty_vec() {
  let mut v = MiniVec::<i32>::new();
  v.extend_from_within(..);

  assert_eq!(v, []);
}

#[test]
fn test_extend_from_within() {
  let mut v = mini_vec![String::from("a"), String::from("b"), String::from("c")];
  v.extend_from_within(1..=2);
  v.extend_from_within(..=1);

  assert_eq!(v, ["a", "b", "c", "b", "c", "a", "b"]);
}
