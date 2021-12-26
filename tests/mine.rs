extern crate minivec;

use minivec::{mini_vec, MiniVec};
use std::{
  collections::hash_map::DefaultHasher,
  convert::From,
  hash::{Hash, Hasher},
  ops::{Index, IndexMut},
};

#[test]
fn minivec_default_constructed() {
  let v: MiniVec<i32> = MiniVec::new();
  assert_eq!(v.capacity(), 0);
  assert_eq!(v.len(), 0);
  assert!(v.is_empty());

  let v: MiniVec<i32> = Default::default();
  assert_eq!(v.capacity(), 0);
  assert_eq!(v.len(), 0);
  assert!(v.is_empty());
}

#[test]
fn minivec_clone_empty() {
  let v = MiniVec::<i32>::new();
  let w = v.clone();

  assert_eq!(w.capacity(), v.capacity());
  assert_eq!(w.len(), v.len());
}

#[test]
fn minivec_as_slice() {
  let mut v = MiniVec::<i32>::new();

  let xs = v.as_slice();
  assert!(xs.is_empty());

  let xs = v.as_mut_slice();
  assert!(xs.is_empty());
}

#[test]
fn minivec_drain_vec() {
  let mut first = mini_vec![1, 2, 3];
  let mut second = first.drain_vec();

  assert!(first.is_empty());
  assert_eq!(first.len(), 0);
  assert_eq!(first.capacity(), 0);
  assert_eq!(second, [1, 2, 3]);

  first.extend_from_slice(&[4, 5, 6]);
  assert_eq!(first.len(), 3);
  assert_eq!(first, [4, 5, 6]);

  second.extend_from_slice(&first[..]);
  assert_eq!(second.len(), 6);
  assert_eq!(second, [1, 2, 3, 4, 5, 6]);
}

#[test]
fn minivec_as_mut() {
  let mut v = mini_vec![1, 2, 3];
  let x: &mut [i32] = v.as_mut();
  assert_eq!(x, [1, 2, 3]);

  let mut v = mini_vec![1, 2, 3];
  let mut vv: &mut MiniVec<i32> = &mut v;
  let x: &mut MiniVec<_> = std::convert::AsMut::<MiniVec<i32>>::as_mut(&mut vv);
  assert_eq!(x, &mut [1, 2, 3].as_mut());
}

#[test]
fn minivec_push() {
  let mut v: MiniVec<i32> = MiniVec::new();

  assert_eq!(v.len(), v.capacity());

  v.push(1);
  v.push(2);
  v.push(3);

  assert_eq!(v.len(), 3);
  assert!(v.capacity() >= v.len());

  let i = v.push(4);
  *i = 1337;
  assert_eq!(v[3], 1337);

  let _: &i32 = v.push(5);

  let mut v: MiniVec<String> = MiniVec::new();

  assert_eq!(v.len(), v.capacity());

  v.push(String::from("Hello"));
  v.push(String::from("Rust"));
  v.push(String::from("World!"));

  assert_eq!(v.len(), 3);
  assert!(v.capacity() >= v.len());

  let mut v: MiniVec<String> = MiniVec::new();

  assert_eq!(v.len(), v.capacity());

  for _ in 0..32 {
    v.push(String::from("Hello, world!"));
  }

  assert_eq!(v.len(), 32);
  assert!(v.capacity() >= v.len());
}

#[test]
fn minivec_deref_test() {
  let mut v: MiniVec<i32> = MiniVec::new();
  v.push(1);
  v.push(2);
  v.push(3);

  assert_eq!(v[0], 1);
  assert_eq!(v[1], 2);
  assert_eq!(v[2], 3);

  let v = MiniVec::<i32>::new();
  let xs: &[i32] = &*v;
  assert_eq!(xs.len(), 0);

  let mut v = MiniVec::<i32>::new();
  let xs: &mut [i32] = &mut *v;
  assert_eq!(xs.len(), 0);
}

#[test]
fn minivec_drop_empty() {
  let _v = MiniVec::<i32>::new();
}

#[test]
fn minivec_dedup_by_test() {
  let mut v = mini_vec![1, 2, 1, 1, 3, 3, 3, 4, 5, 4];
  v.dedup_by(|x, y| x == y);

  assert_eq!(v, [1, 2, 1, 3, 4, 5, 4]);
}

#[test]
fn minivec_dedup_needs_drop() {
  let mut v: MiniVec<Box<_>> = mini_vec![Box::new(1), Box::new(1), Box::new(2), Box::new(3)];
  v.dedup();

  assert_eq!(v.len(), 3);
}

#[test]
fn minivec_drain_into_vec() {
  let mut vec: MiniVec<usize> = mini_vec![];

  let drain = vec.drain(..).collect::<MiniVec<_>>();

  assert_eq!(drain.len(), 0);
}

#[test]
fn minvec_drain() {
  let mut vec = mini_vec![1, 2, 3];

  let mut drain = vec.drain(..);

  assert_eq!(drain.size_hint(), (3, Some(3)));

  drain.next();

  assert_eq!(drain.size_hint(), (2, Some(2)));
}

#[test]
fn minivec_with_capacity() {
  let size = 128;
  let mut v: MiniVec<i32> = MiniVec::with_capacity(size);

  assert_eq!(v.len(), 0);
  assert_eq!(v.capacity(), size);

  v.push(1);
  v.push(2);
  v.push(3);

  assert_eq!(v.len(), 3);
  assert_eq!(v.capacity(), size);
}

#[test]
fn minivec_extend_from_slice() {
  let a = [2, 3, 4, 5];
  let mut v = mini_vec![1];
  v.extend_from_slice(&a);

  assert_eq!(v.len(), 5);

  v.extend_from_slice(&[6, 7, 8]);
  assert_eq!(v.len(), 8);

  assert_eq!(a.len(), 4);
  assert_eq!(a, [2, 3, 4, 5]);

  let a: MiniVec<_> = [2, 3, 4, 5].iter().map(|x| x.to_string()).collect();
  let mut v = mini_vec![String::from("1")];
  v.extend_from_slice(&a);

  assert_eq!(v.len(), 5);

  v.extend_from_slice(&[6.to_string(), 7.to_string(), 8.to_string()]);
  assert_eq!(v.len(), 8);

  assert_eq!(a.len(), 4);
  assert_eq!(
    a,
    [2.to_string(), 3.to_string(), 4.to_string(), 5.to_string()]
  );
}

#[test]
fn minivec_from_raw_part() {
  use std::{mem, ptr};

  let v = mini_vec![1, 2, 3];
  let mut v = mem::ManuallyDrop::new(v);

  let p = v.as_mut_ptr();
  let len = v.len();
  let cap = v.capacity();

  unsafe {
    for i in 0..len as isize {
      ptr::write(p.offset(i), 4 + i);
    }

    let rebuilt = MiniVec::from_raw_part(p);
    assert_eq!(rebuilt, [4, 5, 6]);
    assert_eq!(rebuilt.capacity(), cap);
    assert_eq!(rebuilt.len(), len);
  }
}

#[test]
fn minivec_from_raw_parts() {
  use std::{mem, ptr};

  let v = mini_vec![1, 2, 3];
  let mut v = mem::ManuallyDrop::new(v);

  let p = v.as_mut_ptr();
  let len = v.len();
  let cap = v.capacity();

  unsafe {
    for i in 0..len as isize {
      ptr::write(p.offset(i), 4 + i);
    }

    let rebuilt = MiniVec::from_raw_parts(p, len, cap);
    assert_eq!(rebuilt, [4, 5, 6]);
    assert_eq!(rebuilt.capacity(), cap);
    assert_eq!(rebuilt.len(), len);
  }
}

#[test]
fn minivec_insert() {
  let mut vec = mini_vec![1, 2, 3];

  vec.insert(1, 4);
  assert_eq!(vec, [1, 4, 2, 3]);

  vec.insert(4, 5);
  assert_eq!(vec, [1, 4, 2, 3, 5]);

  let mut vec: MiniVec<String> = mini_vec![1.to_string(), 2.to_string(), 3.to_string()];

  vec.insert(1, 4.to_string());
  assert_eq!(
    vec,
    [1.to_string(), 4.to_string(), 2.to_string(), 3.to_string()]
  );

  vec.insert(4, 5.to_string());
  assert_eq!(
    vec,
    [
      1.to_string(),
      4.to_string(),
      2.to_string(),
      3.to_string(),
      5.to_string()
    ]
  );
}

#[test]
// This is quite heavy-handed, but stops MIRI complaining about the leak.
#[cfg_attr(miri, ignore)]
fn minivec_leak() {
  let x = mini_vec![1, 2, 3];
  let static_ref: &'static mut [usize] = MiniVec::leak(x);
  static_ref[0] += 1;
  assert_eq!(static_ref, &[2, 2, 3]);
}

#[test]
fn minivec_pop() {
  let mut vec = mini_vec![1, 2, 3];
  assert_eq!(vec.pop(), Some(3));
  assert_eq!(vec, [1, 2]);

  let mut vec = mini_vec![1.to_string(), 2.to_string(), 3.to_string()];
  assert_eq!(vec.pop(), Some(String::from("3")));
  assert_eq!(vec, [String::from("1"), String::from("2")]);
}

#[test]
fn minivec_remove() {
  let mut v = mini_vec![1, 2, 3];
  assert_eq!(v.remove(1), 2);
  assert_eq!(v, [1, 3]);
}

#[test]
#[should_panic]
fn minivec_remove_panic() {
  let mut v = mini_vec![1, 2, 3];
  v.remove(v.len());
}

#[test]
fn minivec_remove_item() {
  let mut vec = mini_vec![1, 2, 3, 1];

  vec.remove_item(&1);

  assert_eq!(vec, mini_vec![2, 3, 1]);
}

#[test]
fn minivec_test_resize() {
  let mut vec = mini_vec!["hello"];
  vec.resize(3, "world");
  assert_eq!(vec, ["hello", "world", "world"]);

  let mut vec = mini_vec![1, 2, 3, 4];
  vec.resize(2, 0);
  assert_eq!(vec, [1, 2]);

  let mut vec = mini_vec![1, 2, 3, 4];
  vec.resize(vec.len(), -1);
  assert_eq!(vec, [1, 2, 3, 4]);
}

#[test]
fn minivec_resize_with() {
  let mut vec = mini_vec![1, 2, 3];
  vec.resize_with(5, Default::default);
  assert_eq!(vec, [1, 2, 3, 0, 0]);

  let mut vec = mini_vec![];
  let mut p = 1;
  vec.resize_with(4, || {
    p *= 2;
    p
  });

  assert_eq!(vec, [2, 4, 8, 16]);
}

#[test]
fn minivec_retain() {
  let mut vec = mini_vec![1, 2, 3, 4];
  vec.retain(|&x| x % 2 == 0);
  assert_eq!(vec, [2, 4]);

  let mut vec = mini_vec![1, 2, 3, 4, 5];
  let keep = [false, true, true, false, true];

  let mut i = 0;
  vec.retain(|_| {
    let should_keep = keep[i];
    i += 1;
    should_keep
  });

  assert_eq!(vec, [2, 3, 5]);
}

#[test]
fn minivec_shrink_to() {
  let mut vec = MiniVec::<i32>::with_capacity(10);
  vec.push(1);
  vec.push(2);
  vec.push(3);

  assert_eq!(vec.capacity(), 10);
  assert_eq!(vec.len(), 3);

  vec.shrink_to(4);
  assert_eq!(vec.capacity(), 4);
  assert_eq!(vec.len(), 3);

  vec.shrink_to(0);
  assert_eq!(vec.capacity(), 3);
}

#[test]
#[should_panic]
fn minivec_shrink_to_panic() {
  let mut vec = MiniVec::<String>::with_capacity(10);

  vec.push(String::from("1"));
  vec.push(String::from("2"));
  vec.push(String::from("3"));
  vec.push(String::from("4"));
  vec.push(String::from("5"));

  vec.shrink_to(10000);
}

#[test]
fn minivec_extend() {
  let mut v = mini_vec![1, 2, 3];
  let other = mini_vec![4, 5, 6];

  v.extend(other.iter());

  assert_eq!(v, mini_vec![1, 2, 3, 4, 5, 6]);

  let mut v = mini_vec![String::from("1"), String::from("2"), String::from("3")];
  let other = vec![String::from("4"), String::from("5"), String::from("6")];

  v.extend(other.into_iter());

  assert_eq!(
    v,
    mini_vec![
      String::from("1"),
      String::from("2"),
      String::from("3"),
      String::from("4"),
      String::from("5"),
      String::from("6")
    ]
  );
}

#[test]
fn minivec_from() {
  // From<&'a [T]>
  //
  let data = [1, 2, 3];
  let x: &[i32] = &data[..];
  let v = MiniVec::<i32>::from(x);

  assert_eq!(v, [1, 2, 3]);

  // From<&'a mut [T]>
  //
  let mut data = [1, 2, 3];
  let x: &mut [i32] = &mut data[..];
  let v = MiniVec::<i32>::from(x);

  assert_eq!(v, [1, 2, 3]);

  // From<&'a str>
  //
  let data: &str = "Hello, world!\n";
  let v = MiniVec::from(data);
  assert_eq!(v, Vec::from(data));

  // From<&'a MiniVec<T>> to Cow<'a, [T]>
  //
  let v = mini_vec![1, 2, 3];
  let cow_v = std::borrow::Cow::from(&v);
  assert_eq!(cow_v, &[1, 2, 3][..]);
}

#[test]
fn minivec_hash() {
  let v = mini_vec![1, 2, 3, 4, 5, 6];
  let w = vec![1, 2, 3, 4, 5, 6];

  let mut h = DefaultHasher::new();
  v.hash(&mut h);

  let hashed_mini_vec = h.finish();

  assert_eq!(hashed_mini_vec, 4819070452177268435);

  let mut h = DefaultHasher::new();
  w.hash(&mut h);

  assert_eq!(hashed_mini_vec, h.finish());
}

#[test]
fn minivec_index() {
  let mut v = mini_vec![1, 2, 3, 4, 5, 6];

  assert_eq!(*v.index(0), 1);
  assert_eq!(*v.index(1), 2);
  assert_eq!(*v.index(2), 3);
  assert_eq!(*v.index(3), 4);
  assert_eq!(*v.index(4), 5);
  assert_eq!(*v.index(5), 6);

  *v.index_mut(0) = 3;
  *v.index_mut(1) = 2 * 3;
  *v.index_mut(2) = 3 * 3;
  *v.index_mut(3) = 4 * 3;
  *v.index_mut(4) = 5 * 3;
  *v.index_mut(5) = 6 * 3;

  assert_eq!(*v.index(0), 3);
  assert_eq!(*v.index(1), 2 * 3);
  assert_eq!(*v.index(2), 3 * 3);
  assert_eq!(*v.index(3), 4 * 3);
  assert_eq!(*v.index(4), 5 * 3);
  assert_eq!(*v.index(5), 6 * 3);
}

#[test]
fn minivec_into_iter() {
  // test normal Iterator trait
  //
  let v = mini_vec![1, 2, 3, 4, 5];

  let iter = v.into_iter();
  assert_eq!(iter.size_hint(), (5, Some(5)));

  for (idx, x) in iter.enumerate() {
    assert_eq!(x, idx + 1);
  }

  let v = mini_vec![
    1.to_string(),
    2.to_string(),
    3.to_string(),
    4.to_string(),
    5.to_string()
  ];

  let iter = v.into_iter();
  for (idx, x) in iter.enumerate() {
    assert_eq!(x, (idx + 1).to_string());
  }

  // test as_slice() + as_mut_slice()
  //
  let v = mini_vec![1, 2, 3, 4, 5];

  let mut iter = v.into_iter();
  iter.next();
  iter.next();

  assert_eq!(iter.as_slice(), &[3, 4, 5]);

  let v = mini_vec![1, 2, 3, 4, 5];

  let mut iter = v.into_iter();
  iter.next();
  iter.next();

  assert_eq!(iter.as_mut_slice(), &[3, 4, 5]);

  // test AsRef
  //
  let v = mini_vec![1, 2, 3, 4, 5];

  let iter = v.into_iter();
  let s: &[i32] = iter.as_ref();
  assert_eq!(s, &[1, 2, 3, 4, 5]);

  // test clone()
  //
  let v = mini_vec![1, 2, 3, 4, 5];

  let mut iter = v.into_iter();

  #[allow(clippy::redundant_clone)]
  let w = iter.clone();

  assert_eq!(w.as_slice(), &[1, 2, 3, 4, 5]);

  iter.next();
  iter.next();

  #[allow(clippy::redundant_clone)]
  let w = iter.clone();

  assert_eq!(w.as_slice(), &[3, 4, 5]);

  // test DoubleEndedIterator
  //
  let v = mini_vec![1, 2, 3, 4, 5];

  let mut iter = v.into_iter();

  assert_eq!(iter.next().unwrap(), 1);
  assert_eq!(iter.next_back().unwrap(), 5);

  assert_eq!(iter.next().unwrap(), 2);
  assert_eq!(iter.next_back().unwrap(), 4);

  assert_eq!(iter.next().unwrap(), 3);
  assert_eq!(iter.next_back(), None);

  // test automatic impl of Drop to make sure it frees owning objects
  //
  let v = mini_vec![
    1.to_string(),
    2.to_string(),
    3.to_string(),
    4.to_string(),
    5.to_string()
  ];

  let mut iter = v.into_iter();

  iter.next();
  iter.next();

  // ExactSizeIterator test
  //
  let v = mini_vec![1, 2, 3, 4, 5];

  let mut iter = v.into_iter();

  assert_eq!(iter.len(), 5);

  iter.next();
  assert_eq!(iter.len(), 4);

  iter.next();
  assert_eq!(iter.len(), 3);

  iter.next();
  assert_eq!(iter.len(), 2);

  iter.next();
  assert_eq!(iter.len(), 1);

  iter.next();
  assert_eq!(iter.len(), 0);

  // FusedIterator test
  //
  let v = mini_vec![1];

  let mut iter = v.into_iter();
  assert_eq!(iter.next().unwrap(), 1);

  assert_eq!(iter.next(), None);
  assert_eq!(iter.next(), None);
  assert_eq!(iter.next(), None);
  assert_eq!(iter.next(), None);
  assert_eq!(iter.next(), None);

  // IntoIterator for &MiniVec<T>
  //
  let v = mini_vec![
    1.to_string(),
    2.to_string(),
    3.to_string(),
    4.to_string(),
    5.to_string()
  ];
  let mut slice_iter = (&v).into_iter();

  assert_eq!(slice_iter.next().unwrap(), &(1.to_string()));

  // IntoIterator for &mut MiniVec<T>
  //
  let mut v = mini_vec![
    1.to_string(),
    2.to_string(),
    3.to_string(),
    4.to_string(),
    5.to_string()
  ];

  let x: &mut MiniVec<_> = &mut v;

  let slice_iter = x.into_iter();

  for s in slice_iter {
    *s = "4321".to_string();
  }
}

#[test]
fn minivec_into_iter_as_slice() {
  let mut iter = MiniVec::<i32>::new().into_iter();
  let xs = iter.as_slice();
  assert!(xs.is_empty());

  let xs = iter.as_mut_slice();
  assert!(xs.is_empty());
}

#[test]
fn minivec_as_ref() {
  let v = mini_vec![1, 2, 3, 4];

  let xs: &MiniVec<i32> = <MiniVec<i32> as AsRef<MiniVec<i32>>>::as_ref(&v);
  assert_eq!(xs, &[1, 2, 3, 4]);

  let xs: &[_] = v.as_ref();
  assert_eq!(xs, &[1, 2, 3, 4]);
}

#[test]
fn minivec_borrow() {
  use std::borrow::{Borrow, BorrowMut};

  let mut v = mini_vec![1, 2, 3, 4];

  let xs: &[_] = v.borrow();
  assert_eq!(xs, &[1, 2, 3, 4]);

  let xs: &mut [_] = v.borrow_mut();
  assert_eq!(xs, &[1, 2, 3, 4]);
}

#[test]
fn minivec_ord() {
  let xs = mini_vec![1, 2, 3];
  let ys = mini_vec![4, 5, 6];

  assert_eq!(xs.cmp(&ys), std::cmp::Ordering::Less);
  assert_eq!(ys.cmp(&xs), std::cmp::Ordering::Greater);
  assert_eq!(ys.cmp(&ys), std::cmp::Ordering::Equal);
}

#[test]
fn minivec_partial_cmp() {
  let xs = mini_vec![1, 2, 3];
  let ys = mini_vec![4, 5, 6];

  assert_eq!(xs.partial_cmp(&ys), Some(std::cmp::Ordering::Less));
  assert_eq!(ys.partial_cmp(&xs), Some(std::cmp::Ordering::Greater));
  assert_eq!(ys.partial_cmp(&ys), Some(std::cmp::Ordering::Equal));
}

#[test]
fn minivec_swap_remove() {
  {
    let mut v: MiniVec<&str> = mini_vec!["foo", "bar", "baz", "qux"];

    assert_eq!(v.swap_remove(1), "bar");
    assert_eq!(v, ["foo", "qux", "baz"]);

    assert_eq!(v.swap_remove(0), "foo");
    assert_eq!(v, ["baz", "qux"]);

    assert_eq!(v.swap_remove(0), "baz");
    assert_eq!(v, ["qux"]);

    assert_eq!(v.swap_remove(0), "qux");
    assert!(v.is_empty());
  }

  {
    let mut a = 0;
    let mut b = 1;
    let mut vec = mini_vec![&mut a, &mut b];

    vec.swap_remove(1);
  }
}

#[test]
fn minivec_split_off() {
  let mut vec = mini_vec![1, 2, 3];
  let vec2 = vec.split_off(1);
  assert_eq!(vec, [1]);
  assert_eq!(vec2, [2, 3]);

  let mut vec = mini_vec![1, 2, 3];
  let vec2 = vec.split_off(3);
  assert_eq!(vec, [1, 2, 3]);
  assert_eq!(vec2, []);

  let mut vec = mini_vec![1, 2, 3];
  let vec2 = vec.split_off(0);
  assert_eq!(vec, []);
  assert_eq!(vec2, [1, 2, 3]);

  let mut vec = MiniVec::<i32>::new();
  let vec2 = vec.split_off(0);
  assert_eq!(vec, []);
  assert_eq!(vec2, []);
}

#[test]
fn minivec_splice_empty() {
  let mut v = MiniVec::<i32>::new();
  let replace = mini_vec![1, 2, 3];

  let u: MiniVec<_> = v.splice(.., replace.iter().cloned()).collect();

  assert!(u.is_empty());
  assert_eq!(v, &[1, 2, 3]);
}

#[test]
fn minivec_splice() {
  let mut v = mini_vec![1, 2, 3];
  let new = [7, 8];
  let u: MiniVec<_> = v.splice(..2, new.iter().cloned()).collect();
  assert_eq!(v, &[7, 8, 3]);
  assert_eq!(u, &[1, 2]);

  let mut v = mini_vec![1, 2, 3, 4, 5, 6];
  let new = [7, 8];
  let u: MiniVec<_> = v.splice(1..4, new.iter().cloned()).collect();
  assert_eq!(v, &[1, 7, 8, 5, 6]);
  assert_eq!(u, &[2, 3, 4]);

  let mut v = mini_vec![1, 2, 3];
  let new = [7, 8, 9, 10, 11];
  let u: MiniVec<_> = v.splice(..2, new.iter().cloned()).collect();
  assert_eq!(v, &[7, 8, 9, 10, 11, 3]);
  assert_eq!(u, &[1, 2]);

  let mut v = mini_vec![1, 2, 3];
  let new = [7, 8];
  let u: MiniVec<_> = v.splice(2..2, new.iter().cloned()).collect();
  assert_eq!(v, &[1, 2, 7, 8, 3]);
  assert_eq!(u, &[]);

  let mut v = mini_vec![1, 2, 3, 4, 5, 6];
  let new = [7, 8, 9, 10, 11];
  let mut iter = v.splice(..4, new.iter().cloned());
  let mut u = MiniVec::new();

  u.push(iter.next().unwrap());
  u.push(iter.next().unwrap());

  std::mem::drop(iter);

  assert_eq!(v, &[7, 8, 9, 10, 11, 5, 6]);
  assert_eq!(u, &[1, 2]);

  let mut v = mini_vec![1, 2, 3, 4, 5, 6];
  let new = [7, 8, 9, 10, 11];
  let iter = v.splice(..4, new.iter().cloned());
  std::mem::drop(iter);

  let mut v = mini_vec![1, 2, 3];
  let new = [7, 8, 9];
  let u: MiniVec<_> = v.splice(.., new.iter().cloned()).collect();
  assert_eq!(v, &[7, 8, 9]);
  assert_eq!(u, &[1, 2, 3]);
}

#[test]
fn minivec_splice_raii() {
  let mut v = mini_vec![1.to_string(), 2.to_string(), 3.to_string()];
  let new = [7.to_string(), 8.to_string()];
  let u: MiniVec<_> = v.splice(..2, new.iter().cloned()).collect();
  assert_eq!(v, &[7.to_string(), 8.to_string(), 3.to_string()]);
  assert_eq!(u, &[1.to_string(), 2.to_string()]);

  let mut v = mini_vec![
    1.to_string(),
    2.to_string(),
    3.to_string(),
    4.to_string(),
    5.to_string(),
    6.to_string()
  ];

  let new = [7.to_string(), 8.to_string()];
  let u: MiniVec<_> = v.splice(1..4, new.iter().cloned()).collect();

  assert_eq!(
    v,
    &[
      1.to_string(),
      7.to_string(),
      8.to_string(),
      5.to_string(),
      6.to_string()
    ]
  );

  assert_eq!(u, &[2.to_string(), 3.to_string(), 4.to_string()]);

  let mut v = mini_vec![1.to_string(), 2.to_string(), 3.to_string()];
  let new = [
    7.to_string(),
    8.to_string(),
    9.to_string(),
    10.to_string(),
    11.to_string(),
  ];

  let u: MiniVec<_> = v.splice(..2, new.iter().cloned()).collect();
  assert_eq!(
    v,
    &[
      7.to_string(),
      8.to_string(),
      9.to_string(),
      10.to_string(),
      11.to_string(),
      3.to_string()
    ]
  );

  assert_eq!(u, &[1.to_string(), 2.to_string()]);

  let mut v = mini_vec![1.to_string(), 2.to_string(), 3.to_string()];
  let new = [7.to_string(), 8.to_string()];
  let u: MiniVec<_> = v.splice(2..2, new.iter().cloned()).collect();

  assert_eq!(
    v,
    &[
      1.to_string(),
      2.to_string(),
      7.to_string(),
      8.to_string(),
      3.to_string()
    ]
  );

  assert!(u.is_empty());

  let mut v = mini_vec![
    1.to_string(),
    2.to_string(),
    3.to_string(),
    4.to_string(),
    5.to_string(),
    6.to_string()
  ];

  let new = [
    7.to_string(),
    8.to_string(),
    9.to_string(),
    10.to_string(),
    11.to_string(),
  ];

  let mut iter = v.splice(..4, new.iter().cloned());
  let mut u = MiniVec::new();

  u.push(iter.next().unwrap());
  u.push(iter.next().unwrap());

  std::mem::drop(iter);

  assert_eq!(
    v,
    &[
      7.to_string(),
      8.to_string(),
      9.to_string(),
      10.to_string(),
      11.to_string(),
      5.to_string(),
      6.to_string()
    ]
  );

  assert_eq!(u, &[1.to_string(), 2.to_string()]);
}

#[test]
fn minivec_truncate() {
  let mut vec = mini_vec![1, 2, 3, 4];
  vec.truncate(vec.len());

  assert_eq!(vec, [1, 2, 3, 4]);
}

#[test]
fn minivec_macro() {
  let vec: MiniVec<i32> = mini_vec!();
  assert_eq!(vec.as_ptr(), core::ptr::null());
  assert_eq!(vec.len(), 0);
  assert_eq!(vec.capacity(), 0);

  let vec: MiniVec<i32> = mini_vec![1337; 4096];
  assert_eq!(vec.len(), 4096);
  assert_eq!(vec[0], 1337);
}

#[test]
fn minivec_spare_capacity_mut() {
  let capacity = 128;
  let mut vec = MiniVec::<i32>::with_capacity(capacity);

  assert!(vec.is_empty());
  assert!(vec.capacity() >= capacity);

  let buf = vec.spare_capacity_mut();
  buf.iter_mut().enumerate().for_each(|(idx, byte)| {
    *byte = core::mem::MaybeUninit::<i32>::new(idx as i32);
  });

  let len = capacity;
  unsafe { vec.set_len(len) };

  let first = Some(0);
  let succ = |n: &i32| -> Option<i32> { Some(n + 1) };
  let iter = core::iter::successors(first, succ).take(capacity);
  let expected: MiniVec<i32> = iter.collect();

  assert_eq!(vec, expected);
}

#[allow(clippy::many_single_char_names, clippy::float_cmp, clippy::identity_op)]
#[cfg(target_feature = "avx")]
#[test]
fn minivec_with_alignment() {
  assert!(MiniVec::<i32>::with_alignment(24, 1).is_err());
  assert!(MiniVec::<i32>::with_alignment(24, 9).is_err());
  assert!(MiniVec::<i32>::with_alignment(24, core::mem::align_of::<*const ()>()).is_ok());

  let alignment = 32;
  let num_elems = 64;

  assert!(num_elems % alignment == 0);

  let mut vec = MiniVec::<f32>::with_alignment(num_elems, alignment).unwrap();

  let p = vec.as_ptr();
  assert!(p as usize % alignment == 0);

  let s = vec.spare_capacity_mut();
  assert_eq!(s.len(), num_elems);

  s.iter_mut().enumerate().for_each(|(idx, x)| {
    *x = core::mem::MaybeUninit::new(idx as f32);
  });

  unsafe {
    vec.set_len(num_elems);
  }

  let p = vec.as_mut_ptr();
  for idx in 0..(num_elems / 8) {
    unsafe {
      let x = core::arch::x86_64::_mm256_load_ps(p.add(idx * 8));
      let y = core::arch::x86_64::_mm256_set_ps(8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0);
      let z = core::arch::x86_64::_mm256_add_ps(x, y);
      core::arch::x86_64::_mm256_store_ps(p.add(idx * 8), z);
    }
  }

  for idx in 0..(num_elems / 8) {
    assert_eq!(vec[idx * 8 + 0], (idx * 8 + 0) as f32 + 1.0);
    assert_eq!(vec[idx * 8 + 1], (idx * 8 + 1) as f32 + 2.0);
    assert_eq!(vec[idx * 8 + 2], (idx * 8 + 2) as f32 + 3.0);
    assert_eq!(vec[idx * 8 + 3], (idx * 8 + 3) as f32 + 4.0);
    assert_eq!(vec[idx * 8 + 4], (idx * 8 + 4) as f32 + 5.0);
    assert_eq!(vec[idx * 8 + 5], (idx * 8 + 5) as f32 + 6.0);
    assert_eq!(vec[idx * 8 + 6], (idx * 8 + 6) as f32 + 7.0);
    assert_eq!(vec[idx * 8 + 7], (idx * 8 + 7) as f32 + 8.0);
  }
}

#[test]
fn minivec_into_raw_parts() {
  let vec = minivec::mini_vec![1, 2, 3, 4, 5];

  let (old_len, old_cap) = (vec.len(), vec.capacity());

  let (ptr, len, cap) = vec.into_raw_parts();

  assert_eq!(len, old_len);
  assert_eq!(cap, old_cap);

  let vec = unsafe { minivec::MiniVec::from_raw_parts(ptr, len, cap) };
  assert_eq!(vec, [1, 2, 3, 4, 5]);
}

#[test]
fn minivec_page_aligned() {
  let capacity = 1024;
  let alignment = 4096;
  let mut vec = minivec::MiniVec::<i32>::with_alignment(capacity, alignment).unwrap();
  assert_eq!(vec.as_mut_ptr() as usize % alignment, 0);
}

#[test]
fn minivec_split_at_spare_mut() {
  // empty case
  //
  let mut vec = MiniVec::<i32>::new();

  let (init, uninit) = vec.split_at_spare_mut();
  assert_eq!(init, []);
  assert_eq!(uninit.len(), 0);

  // Copy type
  //
  let capacity = 256;
  let len = capacity / 3;

  let mut vec = MiniVec::<i32>::with_capacity(capacity);

  for idx in 0..len {
    vec.push(idx as i32);
  }

  let (init, uninit) = vec.split_at_spare_mut();

  assert_eq!(init.len(), len);
  assert_eq!(uninit.len(), capacity - len);

  for (idx, v) in uninit.iter_mut().enumerate() {
    *v = core::mem::MaybeUninit::<i32>::new(idx as i32);
  }

  unsafe { vec.set_len(capacity) };

  for idx in 0..len {
    assert_eq!(vec[idx], idx as i32);
  }

  for idx in 0..(capacity - len) {
    assert_eq!(vec[idx + len], idx as i32);
  }

  // Clone/RAII type
  //
  let mut vec = MiniVec::<String>::with_capacity(4);
  vec.push(String::from("hello"));
  vec.push(String::from("world"));

  let (_, uninit) = vec.split_at_spare_mut();
  uninit[0] = core::mem::MaybeUninit::<String>::new(String::from("rawr"));
  uninit[1] = core::mem::MaybeUninit::<String>::new(String::from("RAWR"));

  unsafe { vec.set_len(4) };

  assert_eq!(vec[2], "rawr");
  assert_eq!(vec[3], "RAWR");
}

#[test]
fn minivec_extend_from_within() {
  let mut vec = minivec::MiniVec::<i32>::new();
  vec.extend_from_within(..);

  assert_eq!(vec, []);

  let mut vec = minivec::mini_vec![1, 2, 3, 4, 5];
  vec.extend_from_within(1..4);

  assert_eq!(vec, [1, 2, 3, 4, 5, 2, 3, 4]);

  let mut vec = minivec::MiniVec::<String>::new();
  vec.push(String::from("hello"));
  vec.push(String::from("world"));
  vec.push(String::from("hola"));
  vec.push(String::from("mundo"));

  vec.extend_from_within(0..=2);

  assert_eq!(
    vec,
    [
      String::from("hello"),
      String::from("world"),
      String::from("hola"),
      String::from("mundo"),
      String::from("hello"),
      String::from("world"),
      String::from("hola"),
    ]
  );

  struct Panicker {
    _a: usize,
  }

  static mut CLONES: i32 = 0;

  impl Clone for Panicker {
    fn clone(&self) -> Self {
      unsafe {
        CLONES += 1;

        if CLONES > 14 {
          panic!("oh no, charlie brown!");
        }
      }

      Self { _a: 0 }
    }
  }

  let mut vec = minivec::mini_vec![Panicker{_a: 0}; 10];

  std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
    vec.extend_from_within(..);
  }))
  .unwrap_err();

  assert_eq!(vec.len(), 14);
}

#[test]
#[should_panic]
fn minivec_test_extend_from_within_start_oob() {
  let mut vec = minivec::mini_vec![0i32; 10];
  vec.extend_from_within(13..);
}

#[test]
#[should_panic]
fn minivec_test_extend_from_within_end_oob() {
  let mut vec = minivec::mini_vec![0i32; 10];
  vec.extend_from_within(..24);
}

#[test]
#[should_panic]
fn minivec_test_extend_from_within_empty_oob() {
  let mut vec = minivec::MiniVec::<i32>::new();
  vec.extend_from_within(0..24);
}

#[test]
fn minivec_nonnull_option_space_optimization() {
  assert_eq!(
    core::mem::size_of::<Option<MiniVec<i32>>>(),
    core::mem::size_of::<MiniVec<i32>>()
  );

  assert_eq!(
    core::mem::size_of::<MiniVec<i32>>(),
    core::mem::size_of::<*mut u8>()
  );
}

#[test]
fn minivec_assume_minivec_init() {
  let mut buf = minivec::mini_vec![core::mem::MaybeUninit::<u8>::uninit(); 512];
  buf
    .iter_mut()
    .for_each(|v| *v = core::mem::MaybeUninit::new(137));

  unsafe { buf.set_len(512) };

  let bytes = unsafe { buf.assume_minivec_init() };
  assert_eq!(bytes[0], 137);
  assert_eq!(bytes[511], 137);
}

#[test]
fn minivec_try_reserve() {
  // the stdlib test cases cover things like overflow and maximum allocation size but we need at least _one_ test to
  // tell us that our capacity actually increased
  //
  let mut v = minivec::MiniVec::<i32>::new();
  assert_eq!(v.capacity(), 0);

  let result = v.try_reserve(1337);

  assert!(result.is_ok());
  assert!(v.capacity() > 0);
}
