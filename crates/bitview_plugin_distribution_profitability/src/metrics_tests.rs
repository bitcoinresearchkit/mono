use bitview_plugin_distribution_common::state::cost_basis::PriceTotals;

use super::*;

#[test]
fn pnl_retains_polarity_and_zero_supply_nupl() {
    let spot = Cents::new(200);
    assert_eq!(
        Metrics::pnl(spot, Cents::new(100), Sats::ONE_BTC, true),
        Cents::new(100)
    );
    assert_eq!(
        Metrics::pnl(spot, Cents::new(300), Sats::ONE_BTC, false),
        Cents::new(100)
    );
    assert_eq!(
        Metrics::nupl(spot, Cents::new(100), Sats::ONE_BTC),
        PartsPerMillionSigned32::from(0.5)
    );
    assert_eq!(
        Metrics::nupl(spot, Cents::new(300), Sats::ONE_BTC),
        PartsPerMillionSigned32::from(-0.5)
    );
    assert_eq!(
        Metrics::nupl(spot, Cents::ZERO, Sats::ZERO),
        PartsPerMillionSigned32::ZERO
    );
    assert_eq!(
        Metrics::nupl(Cents::ZERO, Cents::new(100), Sats::ONE_BTC),
        PartsPerMillionSigned32::ZERO
    );
}

#[test]
fn every_filter_rounds_its_own_exact_cap() {
    let bucket = Bucket::between(
        PriceTotals::default(),
        PriceTotals {
            sats: [10, 4, 2, 8],
            cap: [150_000_000, 70_000_000, 10_000_000, 100_000_000],
        },
    );
    assert_eq!(bucket.supply.all, bucket.supply.sth + bucket.supply.lth);
    assert_eq!(bucket.cap.all, Cents::new(2));
    assert_eq!(bucket.cap.sth, Cents::new(1));
    assert_eq!(bucket.cap.lth, Cents::new(1));
    assert_eq!(bucket.cap.under_4m, Cents::ZERO);
    assert_eq!(bucket.cap.over_4m, Cents::new(1));
    assert_eq!(bucket.cap.under_6m, Cents::new(1));
    assert_eq!(bucket.cap.over_6m, Cents::new(1));
}
