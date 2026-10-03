use brk_types::{Cents, CentsSats};

use crate::{CentsSquaredSats, SupplyState};

/// Snapshot of cost basis related state.
#[derive(Clone, Debug)]
pub struct CostBasisSnapshot {
    pub realized_price: Cents,
    pub supply_state: SupplyState,
    pub price_sats: CentsSats,
    pub capitalized_cap_raw: CentsSquaredSats,
}

impl CostBasisSnapshot {
    #[inline]
    pub fn from_utxo(price: Cents, supply: &SupplyState) -> Self {
        let price_sats = CentsSats::from_price_sats(price, supply.value);
        Self {
            realized_price: price,
            supply_state: *supply,
            price_sats,
            capitalized_cap_raw: CentsSquaredSats::from_price_cents_sats(price, price_sats),
        }
    }
}
