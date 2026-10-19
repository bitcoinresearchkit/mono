use bitcoin::Amount;
use bitview_primitives::{CentsCompact, Date};
use bitview_types::{Cohort, Urpd, UrpdAggregation, UrpdBuckets, UrpdWeight};
use brk_types::{Bitcoin, BlockHash, Cents, CentsSats, CentsSigned, Dollars, Height, Sats};

/// What a URPD response echoes about the distribution [`Self::build`] buckets.
pub(super) struct UrpdHeader {
    pub(super) cohort: Cohort,
    pub(super) height: Height,
    pub(super) hash: BlockHash,
    pub(super) date: Date,
    pub(super) weight: UrpdWeight,
    pub(super) aggregation: UrpdAggregation,
    pub(super) close: Cents,
}

impl UrpdHeader {
    /// Aggregates the distribution's entries into response buckets in one pass: they arrive
    /// unique and sorted by price from its projection, so nothing is rehashed or re-sorted.
    pub fn build(self, entries: impl IntoIterator<Item = (CentsCompact, Sats)>) -> Urpd {
        let close = self.close;
        // `OriginUrpd::new` rejects unpriced blocks, the close's included.
        debug_assert!(!close.is_nan(), "URPD close is priced");
        // Every bucket's market value, at most all money's, must fit the signed cents range.
        debug_assert!(
            u128::from(close.inner()) * u128::from(Amount::MAX_MONEY.to_sat()) / Sats::ONE_BTC_U128
                <= i64::MAX as u128,
            "URPD market value exceeds signed price range"
        );
        let mut buckets = UrpdBuckets::default();
        let mut total = 0_u64;
        let mut current: Option<(Cents, Sats, CentsSats)> = None;
        let mut finish = |(price, supply, capital): (Cents, Sats, CentsSats)| {
            let market = CentsSats::from_price_sats(close, supply);
            // The exact difference, rounded once.
            let unrealized_pnl = if market >= capital {
                i128::from((market - capital).to_cents().inner())
            } else {
                -i128::from((capital - market).to_cents().inner())
            };
            buckets.price.push(Dollars::from(price));
            buckets.supply.push(Bitcoin::from(supply));
            buckets.realized_cap.push(Dollars::from(capital.to_cents()));
            buckets.unrealized_pnl.push(Dollars::from(CentsSigned::from(unrealized_pnl)));
        };
        for (price, supply) in entries {
            // Weights are within [0, 1]: a weighted mass never exceeds the raw one.
            debug_assert!(!supply.is_max(), "URPD entry below the sats sentinel");
            total += u64::from(supply);
            debug_assert!(
                total <= Amount::MAX_MONEY.to_sat(),
                "URPD supply within all money"
            );
            let price = Cents::from(price);
            let floor = self.aggregation.bucket_floor(price);
            debug_assert!(
                current.is_none_or(|(last, ..)| last <= floor),
                "URPD prices arrive sorted"
            );
            let capital = CentsSats::from_price_sats(price, supply);
            if let Some((last, sats, cap)) = &mut current
                && *last == floor
            {
                *sats += supply;
                *cap += capital;
            } else if let Some(previous) = current.replace((floor, supply, capital)) {
                finish(previous);
            }
        }
        if let Some(last) = current {
            finish(last);
        }
        Urpd {
            height: self.height,
            hash: self.hash,
            date: self.date,
            cohort: self.cohort,
            weight: self.weight,
            aggregation: self.aggregation,
            close: Dollars::from(close),
            supply: Bitcoin::from(Sats::new(total)),
            buckets,
        }
    }
}
