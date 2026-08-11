use brk_types::CentsSats;

#[derive(Clone, Debug, Default)]
pub struct PendingCapDelta {
    pub inc: CentsSats,
    pub dec: CentsSats,
}

impl PendingCapDelta {
    pub fn is_zero(&self) -> bool {
        self.inc == CentsSats::ZERO && self.dec == CentsSats::ZERO
    }
}
