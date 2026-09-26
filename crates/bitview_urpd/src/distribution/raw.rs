use std::collections::BTreeMap;

use bitcoin::Amount;
use brk_error::{Error, Result};
use brk_types::{CentsCompact, Sats};

/// Mutable price distribution used by live UTXO state.
#[derive(Debug, Clone, Default)]
pub struct UrpdRaw {
    pub map: BTreeMap<CentsCompact, Sats>,
}

impl UrpdRaw {
    /// Resource ceilings, not truncation thresholds. These leave headroom above
    /// the producer's five-significant-digit dollar buckets.
    pub const MAX_ENTRIES: usize = 2_000_000;
    pub const MAX_ENCODED_BYTES: usize = 64 * 1024 * 1024;
}

pub(crate) fn checked_supply(mut values: impl Iterator<Item = u64>) -> Result<Sats> {
    values
        .try_fold(0_u64, |sum, value| {
            sum.checked_add(value)
                .filter(|sum| *sum <= Amount::MAX_MONEY.to_sat())
                .ok_or_else(|| {
                    Error::Deserialization("UrpdRaw: supply exceeds Bitcoin's maximum".into())
                })
        })
        .map(Sats::from)
}
