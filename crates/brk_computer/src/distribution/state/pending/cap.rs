use brk_types::CentsSats;

#[derive(Clone, Debug, Default)]
pub(crate) struct PendingCapDelta {
    pub inc: CentsSats,
    pub dec: CentsSats,
}

impl PendingCapDelta {
    pub(crate) fn is_zero(&self) -> bool {
        self.inc == CentsSats::ZERO && self.dec == CentsSats::ZERO
    }
}
