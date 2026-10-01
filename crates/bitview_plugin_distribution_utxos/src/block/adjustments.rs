use super::detailed_spends::DetailedSpends;
use crate::state::Transacted;
use bitview_plugin_outputs::overwritten_output;
use brk_types::{Cents, Height, OutputType, Sats};
pub fn normalize_supply(
    height: Height,
    received: &mut Transacted,
    detailed: &mut DetailedSpends,
    prices: &[Cents],
) {
    if height.is_zero() {
        *received = Transacted::default();
    }
    if let Some((original, _)) = overwritten_output(height) {
        detailed.add(
            Sats::FIFTY_BTC,
            OutputType::P2PK65,
            prices[usize::from(original)],
            prices[usize::from(height)],
        );
    }
}
