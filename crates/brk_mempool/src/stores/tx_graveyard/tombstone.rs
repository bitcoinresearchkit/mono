use std::{sync::Arc, time::Instant};

use brk_types::{FeeRate, Transaction, Txid};

use crate::{TxRemoval, state::TxEntry};

/// A buried mempool tx, retained for reappearance detection and
/// post-mine analytics. `chunk_rate` is the linearized chunk feerate at
/// burial time - same value `effective_fee_rate` reported while
/// the tx was alive, so an evicted RBF predecessor reports the
/// package-effective rate, not a misleading isolated `fee/vsize`.
#[derive(Clone)]
pub struct TxTombstone {
    pub tx: Arc<Transaction>,
    pub entry: TxEntry,
    /// The body's [`crate::stores::tx_store::TxRecord::digest`].
    pub digest: u64,
    pub chunk_rate: FeeRate,
    pub removal: TxRemoval,
    pub removed_at: Instant,
}

impl TxTombstone {
    pub fn replaced_by(&self) -> Option<&Txid> {
        match &self.removal {
            TxRemoval::Replaced { by } => Some(by),
            TxRemoval::Vanished => None,
        }
    }
}
