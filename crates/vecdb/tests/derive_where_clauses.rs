#![cfg(all(feature = "derive", feature = "pco"))]

use vecdb::{Bytes, Pco};

#[derive(Debug, Clone, Copy, PartialEq, Bytes)]
struct BytesWhere<T>(T)
where
    T: Copy;

#[derive(Debug, Clone, Copy, PartialEq, Bytes)]
struct BytesTrailing<T>(T)
where
    T: Copy;

#[derive(Debug, Clone, Copy, PartialEq, Pco)]
struct PcoWhere<T>(T)
where
    T: Copy;

#[derive(Debug, Clone, Copy, PartialEq, Pco)]
struct PcoTrailing<T>(T)
where
    T: Copy;

#[derive(Debug, Clone, Copy, PartialEq, Pco)]
#[repr(align(16))]
struct Aligned(u64);

#[test]
fn existing_bounds_with_or_without_trailing_commas_roundtrip() {
    let bytes = 42_u64.to_bytes();
    assert_eq!(BytesWhere::from_bytes(&bytes).unwrap(), BytesWhere(42_u64));
    assert_eq!(
        BytesTrailing::from_bytes(&bytes).unwrap(),
        BytesTrailing(42_u64)
    );
    assert_eq!(PcoWhere::from_bytes(&bytes).unwrap(), PcoWhere(42_u64));
    assert_eq!(
        PcoTrailing::from_bytes(&bytes).unwrap(),
        PcoTrailing(42_u64)
    );
    assert_eq!(PcoWhere::<u64>::from_number(42).unwrap().to_number(), 42);
    assert_eq!(PcoTrailing::<u64>::from_number(42).unwrap().to_number(), 42);
}

#[test]
fn layout_guards_preserve_nontransparent_wrappers() {
    const { assert!(!Aligned::IS_NATIVE_LAYOUT) };
    const { assert!(!Aligned::IS_TRANSPARENT) };
    assert_eq!(Aligned::from_number(42).unwrap(), Aligned(42));
    assert_eq!(Aligned(42).to_bytes(), 42_u64.to_bytes());
}
