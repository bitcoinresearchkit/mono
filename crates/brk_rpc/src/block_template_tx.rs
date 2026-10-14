use bitcoin::{Transaction, consensus::encode};
use brk_error::Result;
use brk_types::{Sats, Txid, Weight};

/// One transaction from `getblocktemplate`: Core's stats plus the still
/// encoded body, so block 0 can be projected without a follow-up
/// `getmempoolentry`/`getrawtransaction` per tx, and a poll decodes only the
/// bodies its caller doesn't already hold.
#[derive(Debug, Clone)]
pub struct BlockTemplateTx {
    pub txid: Txid,
    pub fee: Sats,
    pub weight: Weight,
    /// Parent txids also in this template, resolved from Core's
    /// wire-level one-based indices.
    pub depends: Vec<Txid>,
    /// Consensus-encoded body as Core's hex.
    pub(crate) data: String,
}

impl BlockTemplateTx {
    pub fn decode(&self) -> Result<Transaction> {
        Ok(encode::deserialize_hex(&self.data)?)
    }
}
