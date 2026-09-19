use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    Bitcoin, Cents, CentsSats, CentsSigned, Cohort, Date, Dollars, Sats, UrpdAggregation,
    UrpdBucket, UrpdRaw, UrpdWeight,
};

/// UTXO Realized Price Distribution for a cohort on a specific date.
///
/// Supply is grouped by the close price at which each UTXO was last moved.
/// Each bucket exposes three values: supply in BTC, realized cap contribution
/// in USD (sum of `realized_price * supply` over the coins in the bucket), and
/// unrealized P&L in USD (`close * supply - realized_cap`, can be negative).
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Urpd {
    pub cohort: Cohort,
    pub date: Date,
    /// Weighting applied to the source supply.
    pub weight: UrpdWeight,
    /// Aggregation strategy applied to the buckets.
    pub aggregation: UrpdAggregation,
    /// Close price on `date`, in USD. Anchor for `unrealized_pnl`.
    pub close: Dollars,
    /// Sum of `supply` across all buckets, in BTC.
    pub total_supply: Bitcoin,
    pub buckets: Vec<UrpdBucket>,
}

#[derive(Default, Clone, Copy)]
struct BucketAccum {
    supply: Sats,
    realized_cap: CentsSats,
}

impl Urpd {
    /// Build from the raw on-disk distribution plus context.
    pub fn build(
        cohort: Cohort,
        date: Date,
        weight: UrpdWeight,
        close_cents: Cents,
        raw: &UrpdRaw,
        aggregation: UrpdAggregation,
    ) -> Self {
        let sorted = sorted_buckets(raw, aggregation);

        let close = Dollars::from(close_cents);
        let total_supply: Sats = raw.map.values().copied().sum();

        let buckets = sorted
            .into_iter()
            .map(|(price_floor_cents, slot)| {
                let realized_cap_cents = slot.realized_cap.to_cents();
                let close_mc_cents =
                    CentsSats::from_price_sats(close_cents, slot.supply).to_cents();
                let pnl = CentsSigned::from(close_mc_cents.inner())
                    - CentsSigned::from(realized_cap_cents.inner());
                UrpdBucket {
                    price_floor: Dollars::from(price_floor_cents),
                    supply: Bitcoin::from(slot.supply),
                    realized_cap: Dollars::from(realized_cap_cents),
                    unrealized_pnl: Dollars::from(pnl),
                }
            })
            .collect();

        Self {
            cohort,
            date,
            weight,
            aggregation,
            close,
            total_supply: Bitcoin::from(total_supply),
            buckets,
        }
    }
}

fn sorted_buckets(raw: &UrpdRaw, aggregation: UrpdAggregation) -> Vec<(Cents, BucketAccum)> {
    // Raw prices are already unique and ordered by the source BTreeMap.
    if aggregation == UrpdAggregation::Raw {
        return raw
            .map
            .iter()
            .map(|(&price, &supply)| {
                let price = Cents::from(price);
                (
                    price,
                    BucketAccum {
                        supply,
                        realized_cap: CentsSats::from_price_sats(price, supply),
                    },
                )
            })
            .collect();
    }
    let mut agg: FxHashMap<Cents, BucketAccum> =
        FxHashMap::with_capacity_and_hasher(raw.map.len(), Default::default());
    for (&price_cents, &sats) in &raw.map {
        let price = Cents::from(price_cents);
        let key = aggregation.bucket_floor(price);
        let slot = agg.entry(key).or_default();
        slot.supply += sats;
        slot.realized_cap += CentsSats::from_price_sats(price, sats);
    }
    let mut sorted: Vec<_> = agg.into_iter().collect();
    sorted.sort_unstable_by_key(|&(price, _)| price);
    sorted
}
