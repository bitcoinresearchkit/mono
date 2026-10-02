use brk_types::{Sats, SatsSigned};

#[derive(Clone, Copy, Debug, Default)]
pub struct PendingDelta(SatsSigned);

impl PendingDelta {
    #[inline(always)]
    pub(crate) fn increment(&mut self, sats: Sats) {
        self.0 = SatsSigned::new(self.0.inner().wrapping_add_unsigned(sats.into()));
    }

    #[inline(always)]
    pub(crate) fn decrement(&mut self, sats: Sats) {
        self.0 = SatsSigned::new(self.0.inner().wrapping_sub_unsigned(sats.into()));
    }

    #[inline(always)]
    pub(crate) fn inner(self) -> i64 {
        self.0.inner()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gross_churn_may_cross_signed_range() {
        for net in [0, 1] {
            let mut delta = PendingDelta::default();
            let gross = Sats::new(i64::MAX as u64 + 1);

            delta.increment(gross);
            delta.decrement(gross - Sats::new(net));

            assert_eq!(delta.inner(), net as i64);
        }
    }
}
