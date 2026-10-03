use brk_types::Txid;
use serde_json::to_vec;

use super::{TransactionSource, body::ResolvedTxBody};
use crate::{Query, RepresentationId, Result};

/// Transaction JSON resolved to one exact in-memory or indexed source.
pub struct ResolvedTransaction {
    source: ResolvedTxBody,
}

impl ResolvedTransaction {
    pub fn identity(&self) -> RepresentationId {
        self.source.identity()
    }
}

impl Query {
    /// Resolve transaction JSON once before an async response handoff.
    pub fn resolve_transaction(&self, txid: &Txid) -> Result<ResolvedTransaction> {
        let source = match self
            .resolve_transaction_source(txid)
            .map_err(|error| self.transaction_error(error))?
        {
            TransactionSource::Memory(transaction) => {
                ResolvedTxBody::memory(to_vec(transaction.as_ref()).unwrap())
            }
            TransactionSource::Chain(transaction) => ResolvedTxBody::Chain(transaction),
        };
        Ok(ResolvedTransaction { source })
    }

    /// Build JSON bytes without repeating the transaction-prefix lookup.
    pub fn transaction_json_resolved(&self, transaction: ResolvedTransaction) -> Result<Vec<u8>> {
        match transaction.source {
            ResolvedTxBody::Memory { bytes, .. } => Ok(bytes),
            ResolvedTxBody::Chain(transaction) => {
                let read = self.read_indexer()?;
                let (_, index, _) = read
                    .revalidate_confirmed_tx(transaction)
                    .map_err(|error| self.transaction_error(error))?;
                let value = self.transaction_by_index(index, read.pin())?;
                drop(read);
                let bytes = to_vec(&value).unwrap();
                Ok(bytes)
            }
        }
    }
}
