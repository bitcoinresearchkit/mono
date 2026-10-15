use std::sync::Arc;

use brk_types::{Timestamp, Transaction, VSize};

use crate::stores::{TxTombstone, tx_store::TxRecord};

/// A live or recently vanished transaction as one publication holds it.
pub struct PublishedTx {
    pub tx: Arc<Transaction>,
    /// Core's entry vsize: sigop-adjusted, what fee rates divide by.
    pub vsize: VSize,
    pub first_seen: Timestamp,
    /// Hash of the body's public JSON: changes whenever the body does, so it identifies the
    /// representations built from it.
    pub digest: u64,
}

impl PublishedTx {
    pub(crate) fn live(record: &TxRecord) -> Self {
        Self {
            tx: record.tx.clone(),
            vsize: record.entry.vsize,
            first_seen: record.entry.first_seen,
            digest: record.digest,
        }
    }

    pub(crate) fn vanished(tomb: &TxTombstone) -> Self {
        Self {
            tx: tomb.tx.clone(),
            vsize: tomb.entry.vsize,
            first_seen: tomb.entry.first_seen,
            digest: tomb.digest,
        }
    }
}
