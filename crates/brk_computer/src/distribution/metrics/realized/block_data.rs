use brk_types::{Cents, CentsSats, CentsSigned, Sats};

use super::RealizedTotals;

#[derive(Clone, Default)]
pub(crate) struct RealizedBlockData {
    pub cap_raw: CentsSats,
    pub supply: Sats,
    pub cap: Cents,
    pub price: Cents,
    pub profit: Cents,
    pub loss: Cents,
    pub net_pnl: CentsSigned,
    pub value_destroyed: Cents,
}

impl RealizedBlockData {
    pub(crate) fn totals(&self) -> RealizedTotals {
        RealizedTotals {
            cap_raw: self.cap_raw,
            supply: self.supply,
        }
    }
}
