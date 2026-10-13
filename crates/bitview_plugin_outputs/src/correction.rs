use bitview_plugin_indexer::DUPLICATE_COINBASES;
use bitview_primitives::SupplyState;
use brk_types::{Height, Sats};

/// Outputs replaced by the two historical duplicate coinbase transactions.
pub fn overwritten_output(height: Height) -> Option<(Height, SupplyState)> {
    let dup = DUPLICATE_COINBASES
        .iter()
        .find(|dup| dup.height == height)?;
    Some((
        dup.overwritten_height,
        SupplyState {
            value: Sats::FIFTY_BTC,
            utxo_count: 1,
        },
    ))
}
