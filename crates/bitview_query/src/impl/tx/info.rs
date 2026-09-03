use brk_error::Result;
use brk_types::{Transaction, Txid};

use crate::{Query, RepresentationId, representation_id::content_hash};

use super::{ResolvedConfirmedTx, resolved::TransactionSource};

/// Transaction JSON resolved to one exact in-memory or indexed source.
pub struct ResolvedTransaction {
    source: TransactionInfoSource,
}

enum TransactionInfoSource {
    Memory { bytes: Vec<u8>, hash: u64 },
    Chain(ResolvedConfirmedTx),
}

impl ResolvedTransaction {
    fn memory(transaction: Transaction) -> Self {
        let bytes = serde_json::to_vec(&transaction).unwrap();
        let hash = content_hash(&bytes);
        Self {
            source: TransactionInfoSource::Memory { bytes, hash },
        }
    }

    pub fn identity(&self) -> RepresentationId {
        match &self.source {
            TransactionInfoSource::Memory { hash, .. } => RepresentationId::Content(*hash),
            TransactionInfoSource::Chain(transaction) => transaction.identity(),
        }
    }
}

impl Query {
    /// Resolve transaction JSON once before an async response handoff.
    pub fn resolve_transaction(&self, txid: &Txid) -> Result<ResolvedTransaction> {
        Ok(match self.resolve_transaction_source(txid)? {
            TransactionSource::Memory(transaction) => ResolvedTransaction::memory(transaction),
            TransactionSource::Chain(transaction) => ResolvedTransaction {
                source: TransactionInfoSource::Chain(transaction),
            },
        })
    }

    /// Build JSON bytes without repeating the transaction-prefix lookup.
    pub fn transaction_json_resolved(&self, transaction: ResolvedTransaction) -> Result<Vec<u8>> {
        match transaction.source {
            TransactionInfoSource::Memory { bytes, .. } => Ok(bytes),
            TransactionInfoSource::Chain(transaction) => {
                let (_, index, _) = self.revalidate_confirmed_tx(transaction)?;
                let value = self.transaction_by_index(index)?;
                let bytes = serde_json::to_vec(&value).unwrap();
                self.revalidate_confirmed_tx(transaction)?;
                Ok(bytes)
            }
        }
    }
}
