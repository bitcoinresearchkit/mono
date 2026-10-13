use bitview_primitives::{TxOutIndex, TypeIndex};
use brk_types::{OutPoint, OutputType, Sats, StoreValue, TxIndex, TxidPrefix};

use crate::stores::{Utxo, UtxoKey};

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

/// A vout up to 65,535 keys as the prefix and the vout; a larger one (only a non-standard
/// transaction) gets a key of its own instead of its low bits' key, and its outpoint keeps it whole.
#[test]
fn utxo_keys_keep_vouts_past_u16_apart() {
    let prefix = TxidPrefix::from_store_bytes([7; 8]);
    let key = UtxoKey::new(prefix, 0x1234).to_store_bytes();
    assert_eq!(key[..8], prefix.to_store_bytes());
    assert_eq!(key[8..], [0x12, 0x34]);
    for low in [0, 1, u32::from(u16::MAX)] {
        let wide = UtxoKey::new(prefix, (1 << 16) + low).to_store_bytes();
        assert_ne!(wide, UtxoKey::new(prefix, low).to_store_bytes());
        assert_ne!(wide, UtxoKey::new(prefix, (2 << 16) + low).to_store_bytes());
        assert_eq!(wide[8..], (low as u16).to_be_bytes());
    }
    let outpoint = OutPoint::with_vout_index(TxIndex::from(5_u32), 70_000);
    assert_eq!(
        (outpoint.tx_index(), outpoint.vout_index()),
        (TxIndex::from(5_u32), 70_000)
    );
}
