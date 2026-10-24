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
