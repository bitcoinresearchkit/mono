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
