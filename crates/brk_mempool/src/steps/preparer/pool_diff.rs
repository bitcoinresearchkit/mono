use super::{TxAddition, TxRemoval};
use crate::key::TxKey;

pub struct PoolDiff {
    pub added: Vec<TxAddition>,
    pub removed: Vec<(TxKey, TxRemoval)>,
}
