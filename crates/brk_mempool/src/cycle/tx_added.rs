use brk_types::{FeeRate, Sats, Timestamp, Txid, VSize};

#[derive(Debug, Clone, Copy)]
pub struct TxAdded {
    pub txid: Txid,
    pub fee: Sats,
    pub vsize: VSize,
    pub fee_rate: FeeRate,
    pub first_seen: Timestamp,
}
