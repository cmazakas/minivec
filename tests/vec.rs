extern crate minivec;

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
}
