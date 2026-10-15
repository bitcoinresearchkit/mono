use brk_types::{Cents, Sats};

#[derive(Debug, Default, Clone)]
pub(crate) struct UnrealizedState {
    pub(crate) supply_in_profit: Sats,
    pub(crate) supply_in_loss: Sats,
    pub(crate) unrealized_profit: Cents,
    pub(crate) unrealized_loss: Cents,
    /// Creation-date value of the supply in profit, rounded once from the exact sum.
    pub(crate) capital_in_profit: Cents,
    /// Creation-date value of the supply in loss, rounded once from the exact sum.
    pub(crate) capital_in_loss: Cents,
}
