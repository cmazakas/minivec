extern crate minivec;

use minivec::MiniVec;

// Shamelessly copy-paste most of the test structure from the official Vec
// https://github.com/rust-lang/rust/blob/1e6e082039a52c03a2ca93c0483e86b3c7f67af4/src/liballoc/tests/vec.rs
// all credit for helpers should go to the Rust team for the things I stole like DropCounter
//
// For licensing information, see the accompanying license files. The only real modifications done
// to the original source are largely limited to renaming `Vec` to `MiniVec`
// Rust is dual-licensed under Apache and MIT but this is just a test file so ideally, users need
// not even worry about it
//

struct DropCounter<'a> {
    count: &'a mut u32,
}

impl Drop for DropCounter<'_> {
    fn drop(&mut self) {
        *self.count += 1;
    }
}

#[test]
fn size_test() {
    assert_eq!(
        std::mem::size_of::<MiniVec<i32>>(),
        std::mem::size_of::<*const i32>()
    );
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
fn deref_test() {
    let mut v: MiniVec<i32> = MiniVec::new();
    v.push(1);
    v.push(2);
    v.push(3);

    assert_eq!(v[0], 1);
    assert_eq!(v[1], 2);
    assert_eq!(v[2], 3);
}
