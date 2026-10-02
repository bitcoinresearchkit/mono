#![cfg(all(feature = "derive", feature = "pco"))]

use vecdb::{Bytes, Pco};

#[derive(Debug, Clone, Copy, PartialEq, Pco)]
#[repr(align(16))]
struct Aligned(u64);

#[test]
fn alignment_prevents_native_and_transparent_layouts() {
    const { assert!(!Aligned::IS_NATIVE_LAYOUT) };
    const { assert!(!Aligned::IS_TRANSPARENT) };
    assert_eq!(Aligned::from_number(42).unwrap(), Aligned(42));
    assert_eq!(Aligned(42).to_bytes(), 42_u64.to_bytes());
}
