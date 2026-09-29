use brk_types::{Cents, CentsSats, Sats, SupplyState};
use std::ops::AddAssign;

#[derive(Default, Clone, Copy)]
pub struct SpendDelta {
    pub supply: SupplyState,
    pub previous: CentsSats,
    pub profit: CentsSats,
    pub loss: CentsSats,
}
impl SpendDelta {
    pub fn new(value: Sats, previous: Cents, current: Cents) -> Self {
        let prev = CentsSats::from_price_sats(previous, value);
        let now = CentsSats::from_price_sats(current, value);
        Self {
            supply: SupplyState {
                value,
                utxo_count: 1,
            },
            previous: prev,
            profit: if now > prev {
                now - prev
            } else {
                CentsSats::ZERO
            },
            loss: if prev > now {
                prev - now
            } else {
                CentsSats::ZERO
            },
        }
    }
}
impl AddAssign<Self> for SpendDelta {
    fn add_assign(&mut self, rhs: Self) {
        self.supply += rhs.supply;
        self.previous += rhs.previous;
        self.profit += rhs.profit;
        self.loss += rhs.loss;
    }
}
