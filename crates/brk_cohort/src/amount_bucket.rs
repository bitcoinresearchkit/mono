use brk_types::Sats;

/// Bucket index for amount ranges. Use for cheap comparisons and direct lookups.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AmountBucket(u8);

impl AmountBucket {
    /// Returns both buckets when they differ.
    #[inline(always)]
    pub fn transition_to(self, other: Self) -> Option<(Self, Self)> {
        (self != other).then_some((self, other))
    }

    #[inline(always)]
    pub fn index(self) -> u8 {
        self.0
    }
}

impl From<Sats> for AmountBucket {
    #[inline(always)]
    fn from(value: Sats) -> Self {
        Self(match value {
            v if v < Sats::_1 => 0,
            v if v < Sats::_10 => 1,
            v if v < Sats::_100 => 2,
            v if v < Sats::_1K => 3,
            v if v < Sats::_10K => 4,
            v if v < Sats::_100K => 5,
            v if v < Sats::_1M => 6,
            v if v < Sats::_10M => 7,
            v if v < Sats::_1BTC => 8,
            v if v < Sats::_10BTC => 9,
            v if v < Sats::_100BTC => 10,
            v if v < Sats::_1K_BTC => 11,
            v if v < Sats::_10K_BTC => 12,
            v if v < Sats::_100K_BTC => 13,
            _ => 14,
        })
    }
}

/// Checks whether two amounts belong to different buckets.
#[inline(always)]
pub fn amounts_in_different_buckets(a: Sats, b: Sats) -> bool {
    AmountBucket::from(a) != AmountBucket::from(b)
}
