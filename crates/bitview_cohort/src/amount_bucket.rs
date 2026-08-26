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
        let value = u64::from(value);
        Self(if value == 0 {
            0
        } else {
            (value.ilog10() + 1).min(14) as u8
        })
    }
}

/// Checks whether two amounts belong to different buckets.
#[inline(always)]
pub fn amounts_in_different_buckets(a: Sats, b: Sats) -> bool {
    AmountBucket::from(a) != AmountBucket::from(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal_boundaries_select_the_expected_bucket() {
        assert_eq!(AmountBucket::from(Sats::ZERO).index(), 0);
        for exponent in 0..14 {
            let boundary = 10_u64.pow(exponent);
            assert_eq!(
                AmountBucket::from(Sats::from(boundary)).index(),
                (exponent + 1).min(14) as u8,
            );
            if boundary > 1 {
                assert_eq!(
                    AmountBucket::from(Sats::from(boundary - 1)).index(),
                    exponent as u8,
                );
            }
        }
        assert_eq!(AmountBucket::from(Sats::MAX).index(), 14);
    }
}
