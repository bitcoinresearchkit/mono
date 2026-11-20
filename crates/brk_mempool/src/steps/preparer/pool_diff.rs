use brk_types::Txid;

use super::{TxAddition, TxRemoval};

pub struct PoolDiff {
    pub added: Vec<TxAddition>,
    pub removed: Vec<(Txid, TxRemoval)>,
}
