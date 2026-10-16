use brk_types::{Cents, CentsSats, CentsSigned, Sats};

#[derive(Clone, Default)]
pub struct RealizedBlockData {
    pub(crate) cap_raw: CentsSats,
    pub(crate) supply: Sats,
    pub cap: Cents,
    pub profit: Cents,
    pub loss: Cents,
    pub(crate) net_pnl: CentsSigned,
    pub(crate) value_destroyed: Cents,
}

impl RealizedBlockData {
    /// Compute mean creation price only for consumers that publish it; undefined (NaN) without
    /// supply.
    pub fn price(&self) -> Cents {
        self.cap_raw.realized_price(self.supply)
    }
}
