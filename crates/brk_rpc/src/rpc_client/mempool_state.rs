use brk_types::{BlockHash, FeeRate, Txid};

pub struct MempoolState {
    pub live_txids: Vec<Txid>,
    pub min_fee: FeeRate,
    /// Chain tip's hash from the block template.
    pub tip_hash: BlockHash,
}
