use super::{TxAddition, TxRemoval};
use crate::key::TxKey;

pub struct PoolDiff {
    pub live_len: usize,
    pub added: Vec<TxAddition>,
    /// Pooled txs that turned listed: they re-assert their outpoints.
    pub promoted: Vec<TxKey>,
    pub removed: Vec<(TxKey, TxRemoval)>,
}
