use brk_types::{FeeRate, Sats, Timestamp, Txid, VSize};

#[derive(Debug, Clone)]
pub struct RbfNode {
    pub txid: Txid,
    pub fee: Sats,
    pub vsize: VSize,
    pub value: Sats,
    pub first_seen: Timestamp,
    pub rate: FeeRate,
    pub(crate) in_mempool: bool,
    /// Neither pending nor buried: a conflict a block confirmed without the pool seeing it (the
    /// mined replacement of what it replaced). Its txid, what it replaced and when that was seen
    /// (`first_seen`) are known here; its own facts (fee, size, value, rate, signaling) are the
    /// chain's to give.
    pub unseen: bool,
    /// BIP-125 signaling: at least one input has sequence < 0xffffffff-1.
    pub rbf: bool,
    /// `true` iff a transaction it directly replaced was non-signaling.
    pub replaced_non_signaling: bool,
    /// mempool.space's tree `fullRbf`: on a replacer, whether any predecessor in its subtree was
    /// non-signaling; on a replaced leaf, whether it didn't signal itself.
    pub full_rbf: bool,
    pub replaces: Vec<RbfNode>,
}
