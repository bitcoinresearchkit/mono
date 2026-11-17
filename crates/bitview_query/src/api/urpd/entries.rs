use bitcoin::Amount;
use bitview_primitives::{CentsCompact, Date};
use bitview_types::{Urpd, UrpdAggregation, UrpdBuckets, UrpdCohort, UrpdWeight};
use brk_types::{Bitcoin, BlockHash, Cents, CentsSats, CentsSigned, Dollars, Height, Sats};

use crate::{Result, api::json};

/// One bucket in cents and sats.
struct Bucket {
    price: u64,
    supply: u64,
    realized_cap: u64,
    unrealized_pnl: i64,
}

/// What a URPD response echoes about the distribution [`Self::build`] buckets.
pub(super) struct UrpdHeader {
    pub(super) cohort: UrpdCohort,
    pub(super) height: Height,
    pub(super) hash: BlockHash,
    pub(super) date: Date,
    pub(super) weight: UrpdWeight,
    pub(super) aggregation: UrpdAggregation,
    pub(super) close: Cents,
}

impl UrpdHeader {
    /// Aggregates the distribution's entries into buckets in one pass and writes them as the
    /// JSON of `Urpd`: they arrive unique and sorted by price from its projection, so nothing is
    /// rehashed or re-sorted.
    pub fn build(self, entries: impl IntoIterator<Item = (CentsCompact, Sats)>) -> Result<Vec<u8>> {
        let close = self.close;
        // The URPD history rejects unpriced blocks, the close's included.
        debug_assert!(!close.is_nan(), "URPD close is priced");
        // Every bucket's market value, at most all money's, must fit the signed cents range.
        debug_assert!(
            u128::from(close.inner()) * u128::from(Amount::MAX_MONEY.to_sat()) / Sats::ONE_BTC_U128
                <= i64::MAX as u128,
            "URPD market value exceeds signed price range"
        );
        let mut buckets = Vec::new();
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
            buckets.push(Bucket {
                price: price.inner(),
                supply: u64::from(supply),
                realized_cap: capital.to_cents().inner(),
                unrealized_pnl: i64::from(CentsSigned::from(unrealized_pnl)),
            });
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
        let mut body = Vec::with_capacity(512 + 48 * buckets.len());
        json::open(&mut body, self.height, &self.hash, &self.date)?;
        json::field(&mut body, "cohort", &self.cohort)?;
        json::field(&mut body, "weight", &self.weight)?;
        json::field(&mut body, "aggregation", &self.aggregation)?;
        json::key(&mut body, "close");
        json::append(&mut body, close.inner(), json::decimal::<2>);
        json::key(&mut body, "supply");
        json::append(&mut body, total, json::decimal::<8>);
        body.extend_from_slice(b",\"buckets\":{\"price\":");
        json::append_array(&mut body, &buckets, |out, b| json::decimal::<2>(out, 0, b.price));
        json::key(&mut body, "supply");
        json::append_array(&mut body, &buckets, |out, b| json::decimal::<8>(out, 0, b.supply));
        json::key(&mut body, "realized_cap");
        json::append_array(&mut body, &buckets, |out, b| {
            json::decimal::<2>(out, 0, b.realized_cap)
        });
        json::key(&mut body, "unrealized_pnl");
        json::append_array(&mut body, &buckets, |out, b| {
            json::signed_decimal::<2>(out, 0, b.unrealized_pnl)
        });
        body.extend_from_slice(b"}}");
        debug_assert!(
            body == serde_json::to_vec(&Urpd {
                height: self.height,
                hash: self.hash,
                date: self.date,
                cohort: self.cohort,
                weight: self.weight,
                aggregation: self.aggregation,
                close: Dollars::from(close),
                supply: Bitcoin::from(Sats::new(total)),
                buckets: UrpdBuckets {
                    price: buckets.iter().map(|b| Dollars::from(Cents::new(b.price))).collect(),
                    supply: buckets.iter().map(|b| Bitcoin::from(Sats::new(b.supply))).collect(),
                    realized_cap: (buckets.iter())
                        .map(|b| Dollars::from(Cents::new(b.realized_cap)))
                        .collect(),
                    unrealized_pnl: (buckets.iter())
                        .map(|b| Dollars::from(CentsSigned::new(b.unrealized_pnl)))
                        .collect(),
                },
            })?,
            "the URPD body is its schema's JSON"
        );
        Ok(body)
    }
}
