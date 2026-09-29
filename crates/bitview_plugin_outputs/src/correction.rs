use brk_types::{Height, Sats, SupplyState};

/// Outputs replaced by the two historical duplicate coinbase transactions.
pub fn overwritten_output(height: Height) -> Option<(Height, SupplyState)> {
    let origin = match u32::from(height) {
        91842 => 91812,
        91880 => 91722,
        _ => return None,
    };
    Some((
        Height::new(origin),
        SupplyState {
            value: Sats::FIFTY_BTC,
            utxo_count: 1,
        },
    ))
}
