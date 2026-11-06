use brk_types::{BlockHash, FeeRate, Txid};

pub struct MempoolState {
    pub live_txids: Vec<Txid>,
    pub min_fee: FeeRate,
    /// Chain tip's hash from the block template.
    pub tip_hash: BlockHash,
    /// The node operator's `prioritisetransaction` fee deltas (sats) of transactions in the
    /// mempool, in txid byte order: Core's block builder ranks by base fee plus delta. Empty on a
    /// node without `getprioritisedtransactions` (before Core 27).
    pub fee_deltas: Box<[(Txid, i64)]>,
}
