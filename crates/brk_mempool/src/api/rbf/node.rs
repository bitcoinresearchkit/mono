use brk_types::{FeeRate, Sats, Timestamp, Txid, VSize};

#[derive(Debug, Clone)]
pub struct RbfNode {
    pub txid: Txid,
    pub fee: Sats,
    pub vsize: VSize,
    pub value: Sats,
    pub first_seen: Timestamp,
    pub rate: FeeRate,
    pub in_mempool: bool,
    /// BIP-125 signaling: at least one input has sequence < 0xffffffff-1.
    pub rbf: bool,
    /// `true` iff any predecessor in this subtree was non-signaling.
    pub full_rbf: bool,
    pub replaces: Vec<RbfNode>,
}
