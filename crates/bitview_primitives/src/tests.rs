use std::str::FromStr;

use crate::PoolSlug;

/// A slug newer than this build reads as `Unknown` through serde, never through `FromStr`.
#[test]
fn unknown_pool_slugs() {
    let read = |json: &str| serde_json::from_str::<PoolSlug>(json).unwrap();
    assert_eq!(read("\"foundryusa\""), PoolSlug::FoundryUsa);
    assert_eq!(read("\"a_pool_from_the_future\""), PoolSlug::Unknown);
    assert_eq!(
        serde_json::to_string(&PoolSlug::Unknown).unwrap(),
        "\"unknown\""
    );
    assert!(PoolSlug::from_str("a_pool_from_the_future").is_err());
}

/// A price above the compact range saturates at its top, never wraps to a small one.
#[test]
fn cents_compact_saturates() {
    use brk_types::Cents;

    use crate::CentsCompact;

    let top = CentsCompact::from(Cents::from(u64::from(u32::MAX) - 1));
    assert_eq!(
        CentsCompact::from(Cents::from(u64::from(u32::MAX) + 1)),
        top
    );
    assert_eq!(CentsCompact::from(Cents::from(10_000_000_000_u64)), top);
    assert!(!top.is_nan());
    assert_eq!(CentsCompact::from(Cents::from(12_345_u64)).inner(), 12_345);
}
