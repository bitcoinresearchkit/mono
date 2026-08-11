use brk_types::{Cents, CentsSats, CentsSquaredSats};

use crate::distribution::state::{RealizedOps, RealizedState};

#[derive(Default)]
pub struct RealizedAggregateState {
    pub cap_raw: CentsSats,
    pub capitalized_cap_raw: CentsSquaredSats,
    peak_regret: CentsSats,
    gross_pnl: Cents,
}

impl RealizedAggregateState {
    pub fn add(&mut self, state: &RealizedState) {
        self.cap_raw += state.cap_raw();
        self.capitalized_cap_raw += state.capitalized_cap_raw();
        self.peak_regret += CentsSats::new(state.peak_regret_raw());
        self.gross_pnl += state.profit() + state.loss();
    }

    pub fn peak_regret(&self) -> Cents {
        self.peak_regret.to_cents()
    }

    pub fn capitalized_price(&self) -> Cents {
        let cap = self.cap_raw.as_u128();
        self.capitalized_cap_raw
            .inner()
            .checked_div(cap)
            .map(|price| Cents::new(price as u64))
            .unwrap_or_default()
    }

    pub fn gross_pnl(&self) -> Cents {
        self.gross_pnl
    }
}
