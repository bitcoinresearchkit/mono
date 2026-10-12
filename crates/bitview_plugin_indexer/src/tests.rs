use bitview_primitives::{TxOutIndex, TypeIndex};
use brk_types::{OutputType, Sats, TxIndex};

use crate::stores::Utxo;

/// The packed entry keeps every field, the 5-byte output index at its bound and small values;
/// a large value is only marked.
#[test]
fn utxo_entry_round_trips() {
    let small = Sats::from(u64::from(u32::MAX >> 1));
    let large = Sats::from(1_u64 << 31);
    for (value, stored) in [
        (small, Some(small)),
        (Sats::ZERO, Some(Sats::ZERO)),
        (large, None),
    ] {
        let utxo = Utxo::new(
            TxIndex::from(u32::MAX - 1),
            TxOutIndex::from((1_u64 << 40) - 1),
            OutputType::P2TR,
            TypeIndex::from(u32::MAX - 2),
            value,
        );
        assert_eq!(utxo.tx_index(), TxIndex::from(u32::MAX - 1));
        assert_eq!(utxo.txout_index(), TxOutIndex::from((1_u64 << 40) - 1));
        assert_eq!(utxo.output_type(), OutputType::P2TR);
        assert_eq!(utxo.type_index(), TypeIndex::from(u32::MAX - 2));
        assert_eq!(utxo.value(), stored);
        assert_eq!(Utxo::is_large(value), stored.is_none());
    }
}
