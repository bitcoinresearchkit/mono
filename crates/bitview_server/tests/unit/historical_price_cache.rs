use brk_error::Error;
use brk_types::{Dollars, ExchangeRates, HistoricalPrice, HistoricalPriceEntry, Timestamp};

use super::*;

fn value(price: f64) -> HistoricalPrice {
    HistoricalPrice {
        prices: vec![HistoricalPriceEntry {
            time: Timestamp::new(1231008000),
            usd: Dollars::from(price),
        }],
        exchange_rates: ExchangeRates {},
    }
}

#[test]
fn shares_bytes_and_rebuilds_on_publication_even_when_the_tip_is_unchanged() {
    let cache = HistoricalPriceCache::default();
    let (first, identity) = cache.get_or_try_init(1, || Ok(value(1.0))).unwrap();
    let (same, _) = cache.get_or_try_init(1, || panic!("cache miss")).unwrap();
    assert_eq!(first.as_ptr(), same.as_ptr());
    assert_eq!(first.as_ref(), to_vec(&value(1.0)).unwrap());
    assert!(
        matches!(identity, RepresentationId::Content(hash) if hash == RepresentationId::content_hash(&first))
    );
    assert!(
        cache
            .get_or_try_init(2, || Err(Error::StateUpdating))
            .is_err()
    );
    let (updated, _) = cache.get_or_try_init(2, || Ok(value(2.0))).unwrap();
    assert_ne!(first, updated);
    assert_eq!(updated.as_ref(), to_vec(&value(2.0)).unwrap());
}
