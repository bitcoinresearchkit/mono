use super::DetailedSpends;
use crate::state::{RealizedOps, Transacted, UTXOStates};
use brk_types::{Cents, OutputType, Sats};
#[test]
fn scalar_spend_totals_preserve_profit_loss_and_zero_value_counts() {
    let mut states = UTXOStates::new();
    let mut spends = DetailedSpends::default();
    for (value, ty, previous) in [
        (Sats::ONE_BTC, OutputType::P2PKH, Cents::new(100)),
        (Sats::ONE_BTC, OutputType::P2PKH, Cents::new(200)),
        (Sats::ZERO, OutputType::P2PKH, Cents::new(150)),
    ] {
        let mut received = Transacted::default();
        received.iterate(value, ty);
        states.receive_details(&received, previous);
        spends.add(value, ty, previous, Cents::new(150));
    }
    spends.apply(&mut states);
    let state = states.type_.get(OutputType::P2PKH);
    assert_eq!(
        (state.supply.value, state.supply.utxo_count),
        (Sats::ZERO, 0)
    );
    assert_eq!(state.spent_utxo_count, 3);
    assert_eq!(state.sent, Sats::new(200_000_000));
    assert_eq!(state.realized.cap(), Cents::ZERO);
    assert_eq!(state.realized.profit(), Cents::new(50));
    assert_eq!(state.realized.loss(), Cents::new(50));
}
