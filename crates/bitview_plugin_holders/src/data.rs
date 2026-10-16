use bitview_primitives::{CentsSquaredSats, CoinDays, Count};
use brk_types::{Cents, CentsSats, CentsSigned, Sats};
use derive_more::AddAssign;

/// Additive accounting inputs from disjoint age bands at one height.
#[derive(Clone, Copy, Default, AddAssign)]
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
    pub cdd: CoinDays,
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
    pub capital_profit: Cents,
    pub capital_loss: Cents,
    pub peak_regret_raw: CentsSats,
}
