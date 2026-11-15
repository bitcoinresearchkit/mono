use brk_types::{BlockHash, FeeRate, Txid};

pub struct MempoolState {
    pub live_txids: Vec<Txid>,
    pub min_fee: FeeRate,
    /// The chain tip's hash.
    pub tip_hash: BlockHash,
    /// The node finished loading its saved mempool before the listing was taken: until then a
    /// transaction missing from it may only be unread yet.
    pub loaded: bool,
}

/// One observation for the projector.
pub struct MempoolObservation {
    pub state: MempoolState,
    /// The node's signed `modified_fee` (sats) of each prioritised transaction in the mempool,
    /// in txid byte order: base fee plus the operator's `prioritisetransaction` delta. Empty on a
    /// node without `getprioritisedtransactions` (before Core 27); the previous cycle's after a
    /// failed read.
    pub modified_fees: Box<[(Txid, i64)]>,
}
