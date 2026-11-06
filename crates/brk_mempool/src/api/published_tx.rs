use std::sync::Arc;

use brk_types::{Sats, Timestamp, Transaction, VSize};

use crate::stores::tx_store::TxRecord;

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
    pub(crate) fn from_record(record: &TxRecord) -> Self {
        Self {
            tx: record.tx.clone(),
            vsize: record.entry.vsize,
            first_seen: record.entry.first_seen,
            digest: record.digest,
        }
    }
}

/// A live transaction's entry as Core reports it.
pub struct PublishedEntry {
    pub fee: Sats,
    /// Sigop-adjusted, what fee rates divide by.
    pub vsize: VSize,
    pub first_seen: Timestamp,
}
