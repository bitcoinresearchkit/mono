use brk_types::{Sats, SatsSigned};

#[derive(Clone, Copy, Debug, Default)]
pub struct PendingDelta(SatsSigned);

impl PendingDelta {
    #[inline(always)]
    pub fn increment(&mut self, sats: Sats) {
        self.0 = SatsSigned::new(self.0.inner().wrapping_add_unsigned(sats.into()));
    }

    #[inline(always)]
    pub fn decrement(&mut self, sats: Sats) {
        self.0 = SatsSigned::new(self.0.inner().wrapping_sub_unsigned(sats.into()));
    }

    #[inline(always)]
    pub fn inner(self) -> i64 {
        self.0.inner()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gross_churn_may_cross_signed_range() {
        let mut delta = PendingDelta::default();
        let sats = Sats::new(i64::MAX as u64 + 1);

        delta.increment(sats);
        delta.decrement(sats);

        assert_eq!(delta.inner(), 0);
    }

    #[test]
    fn retains_small_net_after_crossing_signed_range() {
        let mut delta = PendingDelta::default();
        let gross = Sats::new(i64::MAX as u64 + 1);

        delta.increment(gross);
        delta.decrement(gross - Sats::new(1));

        assert_eq!(delta.inner(), 1);
    }
}
