use std::ops::AddAssign;

use bitview_primitives::{CentsSquaredSats, Count, StoredF64};
use brk_types::{Cents, CentsSats, CentsSigned, Sats};

/// Additive accounting inputs from disjoint age bands at one height.
#[derive(Clone, Copy, Default)]
pub(crate) struct Data {
    pub supply: Sats,
    pub count: Count,
    pub spent_count: Count,
    pub volume_sats: Sats,
    pub volume_cents: Cents,
    pub volume_profit_sats: Sats,
    pub volume_profit_cents: Cents,
    pub volume_loss_sats: Sats,
    pub volume_loss_cents: Cents,
    pub adjusted_volume: Cents,
    pub adjusted_value_destroyed: Cents,
    pub cdd: StoredF64,
    pub cap: Cents,
    pub cap_raw: CentsSats,
    pub capitalized_cap_raw: CentsSquaredSats,
    pub profit: Cents,
    pub loss: Cents,
    pub net_pnl: CentsSigned,
    pub value_destroyed: Cents,
    pub unrealized_profit: Cents,
    pub unrealized_loss: Cents,
    pub supply_profit: Sats,
    pub supply_loss: Sats,
    pub capitalized_profit: CentsSquaredSats,
    pub capitalized_loss: CentsSquaredSats,
    pub peak_regret_raw: CentsSats,
}

impl AddAssign for Data {
    fn add_assign(&mut self, rhs: Self) {
        self.supply += rhs.supply;
        self.count += rhs.count;
        self.spent_count += rhs.spent_count;
        self.volume_sats += rhs.volume_sats;
        self.volume_cents += rhs.volume_cents;
        self.volume_profit_sats += rhs.volume_profit_sats;
        self.volume_profit_cents += rhs.volume_profit_cents;
        self.volume_loss_sats += rhs.volume_loss_sats;
        self.volume_loss_cents += rhs.volume_loss_cents;
        self.adjusted_volume += rhs.adjusted_volume;
        self.adjusted_value_destroyed += rhs.adjusted_value_destroyed;
        self.cdd += rhs.cdd;
        self.cap += rhs.cap;
        self.cap_raw += rhs.cap_raw;
        self.capitalized_cap_raw += rhs.capitalized_cap_raw;
        self.profit += rhs.profit;
        self.loss += rhs.loss;
        self.net_pnl += rhs.net_pnl;
        self.value_destroyed += rhs.value_destroyed;
        self.unrealized_profit += rhs.unrealized_profit;
        self.unrealized_loss += rhs.unrealized_loss;
        self.supply_profit += rhs.supply_profit;
        self.supply_loss += rhs.supply_loss;
        self.capitalized_profit += rhs.capitalized_profit;
        self.capitalized_loss += rhs.capitalized_loss;
        self.peak_regret_raw += rhs.peak_regret_raw;
    }
}
