use rustc_hash::FxHashSet;

use super::{TxAddition, TxRemoval};
use crate::key::TxKey;

pub struct PoolDiff {
    /// What Core lists: pending, whether loaded yet or not.
    pub listed: FxHashSet<TxKey>,
    pub added: Vec<TxAddition>,
    pub removed: Vec<(TxKey, TxRemoval)>,
}
