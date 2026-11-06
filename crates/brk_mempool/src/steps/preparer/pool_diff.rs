use rustc_hash::FxHashSet;

use super::{TxAddition, TxRemoval};
use crate::key::TxKey;

pub struct PoolDiff {
    /// What Core lists: pending, whether loaded yet or not.
    pub listed: FxHashSet<TxKey>,
    pub added: Vec<TxAddition>,
    /// Pooled txs that turned listed: they re-assert their outpoints.
    pub promoted: Vec<TxKey>,
    pub removed: Vec<(TxKey, TxRemoval)>,
}
