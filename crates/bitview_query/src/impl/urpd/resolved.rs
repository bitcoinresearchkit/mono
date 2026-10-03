use bitcoin::Amount;
use bitview_types::{Urpd, UrpdBucket};
use brk_error::{Error, Result};
use brk_types::{Bitcoin, Cents, CentsCompact, CentsSats, CentsSigned, Dollars, Sats};

use super::ResolvedUrpd;

impl ResolvedUrpd {
    pub fn entries(&self) -> &[(CentsCompact, Sats)] {
        &self.entries
    }

    /// Aggregates the validated, sorted entries into response buckets.
    pub fn build(self) -> Result<Urpd> {
        self.validate()?;
        let close = self.close;
        let mut buckets = Vec::new();
        let mut total = Sats::ZERO;
        let mut current: Option<(Cents, Sats, CentsSats)> = None;
        let finish = |(price, supply, capital): (Cents, Sats, CentsSats)| {
            let realized_cap = capital.to_cents();
            let market_cap = CentsSats::from_price_sats(close, supply).to_cents();
            UrpdBucket {
                price_floor: Dollars::from(price),
                supply: Bitcoin::from(supply),
                realized_cap: Dollars::from(realized_cap),
                unrealized_pnl: Dollars::from(
                    CentsSigned::from(market_cap.inner()) - CentsSigned::from(realized_cap.inner()),
                ),
            }
        };
        for &(price, supply) in &self.entries {
            let price = Cents::from(price);
            let floor = self.aggregation.bucket_floor(price);
            let capital = CentsSats::from_price_sats(price, supply);
            total += supply;
            if let Some((last, sats, cap)) = &mut current
                && *last == floor
            {
                *sats += supply;
                *cap += capital;
            } else if let Some(previous) = current.replace((floor, supply, capital)) {
                buckets.push(finish(previous));
            }
        }
        if let Some(last) = current {
            buckets.push(finish(last));
        }
        Ok(Urpd {
            cohort: self.cohort,
            date: self.date,
            height: self.height,
            weight: self.weight,
            aggregation: self.aggregation,
            close: Dollars::from(close),
            total_supply: Bitcoin::from(total),
            buckets,
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.close.is_nan() {
            return Err(Error::Internal("Invalid URPD close price"));
        }
        if !self.entries.windows(2).all(|pair| pair[0].0 < pair[1].0) {
            return Err(Error::Internal("Unsorted URPD prices"));
        }
        let supply = self
            .entries
            .iter()
            .try_fold(0_u64, |total, (price, sats)| {
                if price.is_nan() || sats.is_max() {
                    return Err(Error::Internal("Invalid URPD entry"));
                }
                total
                    .checked_add(u64::from(*sats))
                    .filter(|sum| *sum <= Amount::MAX_MONEY.to_sat())
                    .ok_or(Error::Internal("URPD supply overflow"))
            })?;
        if u128::from(self.close.inner()) * u128::from(supply) / Sats::ONE_BTC_U128
            > i64::MAX as u128
        {
            return Err(Error::Internal(
                "URPD market value exceeds signed price range",
            ));
        }
        Ok(())
    }
}
