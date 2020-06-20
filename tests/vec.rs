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
