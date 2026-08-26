use brk_types::{BlockHash, FeeRate, Height, Txid};

/// Mempool snapshot data that survives one fetch cycle.
pub struct MempoolState {
    pub live_txids: Vec<Txid>,
    pub min_fee: FeeRate,
    /// Chain tip's hash from the block template.
    pub tip_hash: BlockHash,
    /// Chain tip's height from the block template.
    pub tip_height: Height,
}
