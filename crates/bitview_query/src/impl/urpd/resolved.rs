use bitview_urpd::{EncodedAgeRangeUrpds, UrpdRaw, build_response, weighted_entries};
use brk_error::{Error, Result};
use brk_types::{CentsCompact, Sats, Urpd};

#[allow(clippy::large_enum_variant)] // One captured request input; keep aggregate sections inline.
pub enum UrpdInput {
    Raw(Vec<u8>),
    Aggregate(EncodedAgeRangeUrpds),
}

impl UrpdInput {
    pub(super) fn decode_entries(&self) -> Result<Vec<(CentsCompact, Sats)>> {
        match self {
            Self::Raw(bytes) => UrpdRaw::deserialize_entries(bytes),
            Self::Aggregate(input) => input.decode_entries(),
        }
    }
}

use super::ResolvedUrpd;

impl ResolvedUrpd {
    /// Visit encoded sections in decoding order without allocating an iterator wrapper.
    pub fn for_each_section(&self, mut visit: impl FnMut(&[u8])) {
        match &self.input {
            UrpdInput::Raw(bytes) => visit(bytes),
            UrpdInput::Aggregate(input) => input.sections().for_each(visit),
        }
    }

    pub fn build(self) -> Result<Urpd> {
        let entries = self.validated_entries()?;
        drop(self.input);
        Ok(build_response(
            self.cohort,
            self.date,
            self.weight,
            self.close,
            entries,
            self.aggregation,
        ))
    }

    /// Check captured inputs without constructing response buckets or JSON.
    pub fn validate(self) -> Result<()> {
        self.validated_entries().map(|_| ())
    }

    fn validated_entries(&self) -> Result<Vec<(CentsCompact, Sats)>> {
        self.validate_metadata()?;
        let entries: Vec<_> = weighted_entries(self.input.decode_entries()?, self.scalar).collect();
        // Validated source supply and a scalar in [0, 1] bound the weighted sum.
        self.validate_market_value(entries.iter().map(|(_, sats)| u64::from(*sats)).sum())?;
        Ok(entries)
    }

    fn validate_metadata(&self) -> Result<()> {
        if !self.scalar.is_finite() || !(0.0..=1.0).contains(&self.scalar) || self.close.is_nan() {
            return Err(Error::Internal("Invalid URPD weight or close price"));
        }
        Ok(())
    }

    fn validate_market_value(&self, supply: u64) -> Result<()> {
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

#[cfg(test)]
#[path = "../../../tests/unit/impl/urpd/resolved.rs"]
mod tests;
