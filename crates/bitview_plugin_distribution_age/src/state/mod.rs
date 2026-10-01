mod cost_basis;
mod utxo;

use brk_types::{Sats, SupplyState};
use statedb::Amount;

pub use bitview_plugin_distribution_common::state::{
    CoreRealizedState, RealizedOps, SendPrecomputed, UnrealizedState, WithCapital, WithoutCapital,
};
pub use cost_basis::RealizedState;
pub use utxo::{UTXOStates, tick_tock_next_block};

#[inline]
pub(crate) fn supply(amount: Amount) -> SupplyState {
    SupplyState {
        value: Sats::new(amount.sats),
        utxo_count: amount.count,
    }
}
