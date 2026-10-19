use bitview_primitives::{TxOutIndex, TypeIndex};
use brk_error::{Error, Result};
use brk_store::Store;
use brk_types::{OutputType, Sats, StoreValue, TxIndex, TxidPrefix};
use vecdb::{Bytes, OverflowVecValue};

/// Key of an unspent output: the creating transaction's txid prefix, then the output's vout (a vout
/// past 65,535 mixes into the prefix, see [`UtxoKey::new`]).
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct UtxoKey([u8; 10]);

impl UtxoKey {
    /// A vout past 65,535 (only a non-standard transaction a miner builds on purpose) mixes its
    /// high bits into the prefix, so its key differs from the outputs sharing its low bits; every
    /// other key is the prefix and the vout.
    #[inline]
    pub(crate) fn new(txid_prefix: TxidPrefix, vout: u32) -> Self {
        let mut prefix = txid_prefix.to_store_bytes();
        if vout > u32::from(u16::MAX) {
            let high = u64::from(vout >> 16).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            prefix = (u64::from_be_bytes(prefix) ^ high).to_be_bytes();
        }
        let mut bytes = [0; 10];
        bytes[..8].copy_from_slice(&prefix);
        bytes[8..].copy_from_slice(&(vout as u16).to_be_bytes());
        Self(bytes)
    }
}

brk_types::impl_store_value!(UtxoKey, [u8; 10]);

/// A large value's marker in the entry: its value is in the overflow store.
const LARGE: u32 = u32::MAX;

/// What resolving an input needs about the output it spends, packed (18 bytes, no padding: the pending map holds
/// millions of them between saves). The output index takes 5 bytes (below 2^40, as `Index40`); the value is
/// `Sats`' compact form, as in `outputs.value`, and a large one (2^31 sats or more) is kept in the overflow store.
#[derive(Debug, Clone, Copy)]
pub struct Utxo([u8; 18]);

impl Utxo {
    #[inline]
    fn new(
        tx_index: TxIndex,
        txout_index: TxOutIndex,
        output_type: OutputType,
        type_index: TypeIndex,
        value: Sats,
    ) -> Self {
        let txout_index = u64::from(txout_index);
        debug_assert!(txout_index < 1 << 40, "output index above 2^40");
        let mut bytes = [0; 18];
        bytes[..4].copy_from_slice(&u32::from(tx_index).to_be_bytes());
        bytes[4..9].copy_from_slice(&txout_index.to_be_bytes()[3..]);
        bytes[9] = output_type as u8;
        bytes[10..14].copy_from_slice(&u32::from(type_index).to_be_bytes());
        bytes[14..].copy_from_slice(&value.to_compact().unwrap_or(LARGE).to_be_bytes());
        Self(bytes)
    }

    /// Whether `value` is kept in the overflow store instead of the entry.
    #[inline]
    fn is_large(value: Sats) -> bool {
        value.to_compact().is_none()
    }

    #[inline]
    pub fn tx_index(&self) -> TxIndex {
        TxIndex::from(u32::from_be_bytes(self.0[..4].try_into().unwrap()))
    }

    #[inline]
    pub fn txout_index(&self) -> TxOutIndex {
        let mut bytes = [0; 8];
        bytes[3..].copy_from_slice(&self.0[4..9]);
        TxOutIndex::from(u64::from_be_bytes(bytes))
    }

    #[inline]
    pub fn output_type(&self) -> OutputType {
        OutputType::from_bytes(&self.0[9..10]).expect("stored output type")
    }

    #[inline]
    pub fn type_index(&self) -> TypeIndex {
        TypeIndex::from(u32::from_be_bytes(self.0[10..14].try_into().unwrap()))
    }

    /// The output's value, or `None` for a large one, which is in the overflow store.
    #[inline]
    pub fn value(&self) -> Option<Sats> {
        let compact = u32::from_be_bytes(self.0[14..].try_into().unwrap());
        (compact != LARGE).then(|| Sats::from_compact(compact))
    }
}

brk_types::impl_store_value!(Utxo, [u8; 18]);

/// Every output not spent yet but OP_RETURN ones: an entry each with what resolving an input needs,
/// in one lookup, and for a large value its overflow record, which inserts and removals keep
/// beside the entry.
#[derive(Clone)]
pub struct Utxos {
    pub(super) entries: Store<UtxoKey, Utxo>,
    /// The values of large unspent outputs, which their entry only marks.
    pub(super) large_values: Store<UtxoKey, Sats>,
}

impl Utxos {
    /// The unspent output under `key`, as last ingested: it can be above the published safe lengths, and the key
    /// holds only an 8-byte txid prefix, so readers outside the indexer check both. Spent outputs miss.
    #[inline]
    pub fn get(&self, key: &UtxoKey) -> Result<Option<Utxo>> {
        self.entries.get(key)
    }

    /// The value of `utxo`, the unspent output under `key`: in the entry, or for a large one in the overflow store.
    #[inline]
    pub fn value(&self, key: &UtxoKey, utxo: &Utxo) -> Result<Sats> {
        match utxo.value() {
            Some(value) => Ok(value),
            None => self
                .large_values
                .get(key)?
                .ok_or(Error::Internal("large unspent output without its value")),
        }
    }

    #[inline]
    pub(crate) fn insert(
        &mut self,
        key: UtxoKey,
        tx_index: TxIndex,
        txout_index: TxOutIndex,
        output_type: OutputType,
        type_index: TypeIndex,
        value: Sats,
    ) {
        self.entries.insert(
            key,
            Utxo::new(tx_index, txout_index, output_type, type_index, value),
        );
        if Utxo::is_large(value) {
            self.large_values.insert(key, value);
        }
    }

    #[inline]
    pub(crate) fn remove(&mut self, key: UtxoKey, value: Sats) {
        self.entries.remove(key);
        if Utxo::is_large(value) {
            self.large_values.remove(key);
        }
    }
}
