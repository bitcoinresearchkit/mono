use brk_types::{BlockHash, Txid, Vout};

/// A block's transactions and the outpoints their inputs spend: what the mempool needs to tell a
/// transaction the block confirmed from one a confirmed conflict displaced.
pub struct BlockSpends {
    pub prev: BlockHash,
    pub txids: Vec<Txid>,
    /// Each non-coinbase input's outpoint and the transaction spending it.
    pub spends: Vec<((Txid, Vout), Txid)>,
}
