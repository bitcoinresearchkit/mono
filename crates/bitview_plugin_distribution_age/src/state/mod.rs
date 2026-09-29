mod utxo;
pub use bitview_plugin_distribution_common::state::*;
mod cost_basis;
mod pending;
use brk_types::{Sats, SupplyState};
pub use cost_basis::{
    CoreRealizedState, CostBasisData, RealizedState, UnrealizedState, WithCapital, WithoutCapital,
};
pub use pending::PendingDelta;
use statedb::Amount;

#[inline]
pub(crate) fn supply(amount: Amount) -> SupplyState {
    SupplyState {
        value: Sats::new(amount.sats),
        utxo_count: amount.count,
    }
}
pub use utxo::{PercentileResult, UTXOStates, tick_tock_next_block};
