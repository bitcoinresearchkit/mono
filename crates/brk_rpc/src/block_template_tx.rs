use bitcoin::Transaction;
use brk_types::{Sats, Txid, Weight};

/// One transaction from `getblocktemplate`. Carries the full decoded
/// body and stats so block 0 can be projected without a follow-up
/// `getmempoolentry`/`getrawtransaction` per tx.
#[derive(Debug, Clone)]
pub struct BlockTemplateTx {
    pub txid: Txid,
    pub fee: Sats,
    pub weight: Weight,
    /// Parent txids also in this template, resolved from Core's
    /// wire-level one-based indices.
    pub depends: Vec<Txid>,
    pub tx: Transaction,
}
