use bitcoin::Wtxid;
use brk_types::{MempoolEntryInfo, Sats, Timestamp, Txid, VSize, Weight};

use crate::key::TxKey;

/// Per-tx facts needed for projection. Chunk rates live on the snapshot, computed when its
/// graph is rebuilt.
#[derive(Debug, Clone)]
pub struct TxEntry {
    pub txid: Txid,
    /// The body's witness id: a template naming another under the same txid replaces the body.
    pub wtxid: Wtxid,
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
            wtxid: info.wtxid,
            fee: info.fee,
            vsize: info.vsize,
            weight: info.weight,
            size,
            first_seen: info.first_seen,
            rbf,
        }
    }

    #[inline]
    pub fn key(&self) -> TxKey {
        TxKey::from(&self.txid)
    }
}
