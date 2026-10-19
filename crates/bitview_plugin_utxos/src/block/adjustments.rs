use bitview_plugin_indexer::overwritten_output;
use brk_types::{Cents, Height, OutputType};

use crate::state::{ReceivedOutputs, UTXOStates};

/// The genesis output never enters the set.
pub fn normalize_supply(height: Height, received: &mut ReceivedOutputs) {
    if height.is_zero() {
        *received = ReceivedOutputs::default();
    }
}

/// A BIP30 duplicate coinbase overwrites an earlier coinbase output: it leaves the set without
/// being spent.
pub fn remove_overwritten(height: Height, states: &mut UTXOStates, prices: &[Cents]) {
    if let Some((original, lost)) = overwritten_output(height) {
        states.lose(OutputType::P2PK65, &lost, prices[usize::from(original)]);
    }
}
