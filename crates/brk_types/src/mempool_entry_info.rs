use bitcoin::Wtxid;

use crate::{Sats, Timestamp, Txid, VSize, Weight};

/// Mempool entry info from Bitcoin Core's `getmempoolentry`.
#[derive(Debug, Clone)]
pub struct MempoolEntryInfo {
    pub txid: Txid,
    /// The witness variant the entry describes: a body is paired with it only when they match.
    pub wtxid: Wtxid,
    pub vsize: VSize,
    pub weight: Weight,
    pub fee: Sats,
    pub first_seen: Timestamp,
}
