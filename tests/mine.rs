extern crate minivec;

use minivec::mini_vec;
use minivec::MiniVec;

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
fn minivec_push() {
  let mut v: MiniVec<i32> = MiniVec::new();

  assert_eq!(v.len(), v.capacity());

  v.push(1);
  v.push(2);
  v.push(3);

  assert_eq!(v.len(), 3);
  assert!(v.capacity() >= v.len());

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
