use brk_types::{Age, Cents, CentsSats, CentsSquaredSats, Sats, SupplyState};

pub struct SendPrecomputed {
    pub(crate) sats: Sats,
    pub(crate) prev_price: Cents,
    pub(crate) age: Age,
    pub(crate) current_ps: CentsSats,
    pub(crate) prev_ps: CentsSats,
    pub(crate) ath_ps: CentsSats,
    pub(crate) prev_capitalized_cap: CentsSquaredSats,
}

impl SendPrecomputed {
    pub fn new(
        supply: &SupplyState,
        current_price: Cents,
        prev_price: Cents,
        ath: Cents,
        age: Age,
    ) -> Option<Self> {
        if supply.utxo_count == 0 || supply.value == Sats::ZERO {
            return None;
        }

        let sats = supply.value;
        let current_ps = CentsSats::from_price_sats(current_price, sats);
        let prev_ps = CentsSats::from_price_sats(prev_price, sats);
        let ath_ps = if ath == current_price {
            current_ps
        } else {
            CentsSats::from_price_sats(ath, sats)
        };

        Some(Self {
            sats,
            prev_price,
            age,
            current_ps,
            prev_ps,
            ath_ps,
            prev_capitalized_cap: prev_ps.to_capitalized_cap(prev_price),
        })
    }
}
