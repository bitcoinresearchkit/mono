use crate::state::{Transacted, UTXOStates};
use bitview_plugin_outputs::overwritten_output;
use brk_types::{Cents, Height, OutputType};

/// The genesis output never enters the set.
pub fn normalize_supply(height: Height, received: &mut Transacted) {
    if height.is_zero() {
        *received = Transacted::default();
    }
}

/// A BIP30 duplicate coinbase overwrites an earlier coinbase output: it leaves the set without
/// being spent.
pub fn remove_overwritten(height: Height, states: &mut UTXOStates, prices: &[Cents]) {
    if let Some((original, lost)) = overwritten_output(height) {
        states.lose(OutputType::P2PK65, &lost, prices[usize::from(original)]);
    }
}
