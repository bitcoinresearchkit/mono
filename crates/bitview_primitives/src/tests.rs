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

/// Five bytes hold values below 2^40 - 1 as they are, and all ones stand for `u64::MAX`, the
/// sentinel `TxInIndex::UNSPENT` is stored as.
#[test]
fn index40_round_trips_and_keeps_its_sentinel() {
    use crate::{Index40, TxInIndex};

    for value in [0, 1, 255, 1 << 32, (1 << 40) - 2] {
        assert_eq!(Index40::<u64>::new(value).get(), value);
    }
    let unspent = Index40::new(TxInIndex::UNSPENT);
    assert_eq!(unspent.get(), TxInIndex::UNSPENT);
    assert_eq!(Index40::<u64>::new(u64::MAX).get(), u64::MAX);
    #[cfg(feature = "storage")]
    assert_eq!(vecdb::Bytes::to_bytes(&unspent), [0xff; 5]);
}
