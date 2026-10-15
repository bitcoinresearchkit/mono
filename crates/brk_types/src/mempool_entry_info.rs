use crate::{Sats, Timestamp, Txid, VSize, Weight};

/// Mempool entry info from Bitcoin Core's `getmempoolentry`.
#[derive(Debug, Clone)]
pub struct MempoolEntryInfo {
    pub txid: Txid,
    pub vsize: VSize,
    pub weight: Weight,
    pub fee: Sats,
    pub first_seen: Timestamp,
}
