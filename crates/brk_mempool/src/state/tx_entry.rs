use brk_types::{FeeRate, MempoolEntryInfo, Sats, Timestamp, Txid, VSize, Weight};

use crate::key::TxKey;

/// Per-tx facts needed for projection. Chunk rates live on the snapshot, computed when its
/// graph is rebuilt.
#[derive(Debug, Clone)]
pub struct TxEntry {
    pub txid: Txid,
    pub fee: Sats,
    pub vsize: VSize,
    pub weight: Weight,
    /// Serialized tx size in bytes (witness + non-witness).
    pub size: u64,
    pub first_seen: Timestamp,
    /// BIP-125 explicit signaling: any input has sequence < 0xfffffffe.
    pub rbf: bool,
}

impl TxEntry {
    pub fn new(info: &MempoolEntryInfo, size: u64, rbf: bool) -> Self {
        Self {
            txid: info.txid,
            fee: info.fee,
            vsize: info.vsize,
            weight: info.weight,
            size,
            first_seen: info.first_seen,
            rbf,
        }
    }

    #[inline]
    pub fn fee_rate(&self) -> FeeRate {
        FeeRate::from((self.fee, self.vsize))
    }

    #[inline]
    pub fn key(&self) -> TxKey {
        TxKey::from(&self.txid)
    }
}
