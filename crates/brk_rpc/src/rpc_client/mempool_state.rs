use brk_types::{BlockHash, FeeRate, Txid};

pub struct MempoolState {
    pub live_txids: Vec<Txid>,
    pub min_fee: FeeRate,
    /// Chain tip's hash from the block template.
    pub tip_hash: BlockHash,
    /// The node operator's `prioritisetransaction` fee deltas (sats) of transactions in the
    /// mempool, in txid byte order. These are the legacy deltas; the projector uses the node's
    /// authoritative signed `modified_fee` amounts in `MempoolObservation`. Empty on a node
    /// without `getprioritisedtransactions` (before Core 27), or a failed metadata read.
    pub fee_deltas: Box<[(Txid, i64)]>,
    /// The node finished loading its saved mempool before the listing was taken: until then a
    /// transaction missing from it may only be unread yet.
    pub loaded: bool,
}

/// One observation for the projector, with ranking metadata separate from the legacy delta API.
pub struct MempoolObservation {
    pub state: MempoolState,
    pub block_template: Option<Vec<crate::BlockTemplateTx>>,
    pub modified_fees: Box<[(Txid, i64)]>,
}
