use brk_types::{Sats, Timestamp};

/// Age of a UTXO or address balance.
#[derive(Debug, Clone, Copy)]
pub struct Age {
    /// Age in hours (primary internal unit for cohort boundaries)
    hours: usize,
    /// Fractional age in days, for satoshi-days destroyed.
    days: f64,
}

impl Age {
    #[inline]
    pub fn new(current_timestamp: Timestamp, prev_timestamp: Timestamp) -> Self {
        Self {
            hours: current_timestamp.hours_since(prev_timestamp),
            days: current_timestamp.days_since(prev_timestamp),
        }
    }

    /// Hours old (for cohort bucket lookup via HOURS_* boundaries)
    #[inline]
    pub fn hours(&self) -> usize {
        self.hours
    }

    /// Supply in satoshis times fractional age in days, rounded down.
    #[inline]
    pub fn satdays_destroyed(&self, supply: Sats) -> Sats {
        Sats::from((u64::from(supply) as f64 * self.days).floor() as u64)
    }
}
