/// Per-block activity counts, reset after every block.
#[derive(Debug, Default, Clone)]
pub struct BlockActivityCounts {
    pub reactivated: u32,
    pub sending: u32,
    pub receiving: u32,
    pub bidirectional: u32,
}

impl BlockActivityCounts {
    #[inline]
    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }

    #[inline(always)]
    pub(crate) fn active(&self) -> u32 {
        debug_assert!(self.bidirectional <= self.sending.min(self.receiving));
        self.sending + self.receiving - self.bidirectional
    }
}
