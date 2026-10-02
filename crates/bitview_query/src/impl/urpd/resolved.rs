use bitcoin::Amount;
use bitview_urpd::build_response;
use brk_error::{Error, Result};
use brk_types::{CentsCompact, Sats, Urpd};

use super::ResolvedUrpd;

impl ResolvedUrpd {
    pub fn entries(&self) -> &[(CentsCompact, Sats)] {
        &self.entries
    }

    pub fn build(self) -> Result<Urpd> {
        self.validate()?;
        Ok(build_response(
            self.cohort,
            self.height,
            self.date,
            self.weight,
            self.close,
            self.entries.into_vec(),
            self.aggregation,
        ))
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
