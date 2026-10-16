use std::sync::Arc;

use bitcoin::hex::DisplayHex;
use brk_types::{Transaction, Txid};
use serde_json::to_vec;

use super::body::ResolvedTxBody;
use crate::{Query, RepresentationId, Result};

/// A transaction resolved to one exact in-memory or indexed source, encoded as JSON, raw bytes
/// or hex once a response needs it.
pub struct ResolvedTransaction {
    source: ResolvedTxBody<Arc<Transaction>>,
}

impl ResolvedTransaction {
    pub fn identity(&self) -> RepresentationId {
        self.source.identity()
    }
}

impl Query {
    /// Resolve a transaction once before an async response handoff.
    pub fn resolve_transaction(&self, txid: &Txid) -> Result<ResolvedTransaction> {
        let source = self.resolve_tx_body(txid, |mempool, tip| {
            Ok(mempool
                .transaction(txid, tip)?
                .map(|transaction| (transaction.tx, transaction.digest)))
        })?;
        Ok(ResolvedTransaction { source })
    }

    /// Build JSON bytes without repeating the transaction-prefix lookup.
    pub fn transaction_json_resolved(&self, transaction: ResolvedTransaction) -> Result<Vec<u8>> {
        match transaction.source {
            ResolvedTxBody::Memory { value: tx, .. } => Ok(to_vec(tx.as_ref()).unwrap()),
            ResolvedTxBody::Chain(transaction) => {
                let read = self.read_indexer()?;
                let (_, index, _) = read.revalidate_confirmed_tx(transaction)?;
                let value = self.transaction_by_index(index, read.pin())?;
                drop(read);
                let bytes = to_vec(&value).unwrap();
                Ok(bytes)
            }
        }
    }

    /// Read raw bytes without repeating the transaction-prefix lookup.
    pub fn transaction_raw_resolved(&self, transaction: ResolvedTransaction) -> Result<Vec<u8>> {
        match transaction.source {
            ResolvedTxBody::Memory { value: tx, .. } => Ok(tx.encode_bytes()),
            ResolvedTxBody::Chain(transaction) => {
                let read = self.read_indexer()?;
                let (_, index, _) = read.revalidate_confirmed_tx(transaction)?;
                self.transaction_raw_by_index(index, read.pin())
            }
        }
    }

    /// Hex-encode raw bytes without repeating transaction resolution.
    pub fn transaction_hex_resolved(&self, transaction: ResolvedTransaction) -> Result<String> {
        self.transaction_raw_resolved(transaction)
            .map(|bytes| bytes.to_lower_hex_string())
    }
}
