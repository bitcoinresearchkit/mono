use bitcoin::hex::DisplayHex;
use brk_types::Txid;

use super::{TransactionSource, body::ResolvedTxBody};
use crate::{Query, RepresentationId, Result};

/// Raw transaction data resolved to one exact in-memory or indexed source.
pub struct ResolvedRawTransaction {
    source: ResolvedTxBody,
}

impl ResolvedRawTransaction {
    pub fn identity(&self) -> RepresentationId {
        self.source.identity()
    }
}

impl Query {
    /// Resolve raw transaction data once before an async response handoff.
    pub fn resolve_raw_transaction(&self, txid: &Txid) -> Result<ResolvedRawTransaction> {
        let source = match self
            .resolve_transaction_source(txid)
            .map_err(|error| self.transaction_error(error))?
        {
            TransactionSource::Memory(transaction) => {
                ResolvedTxBody::memory(transaction.encode_bytes())
            }
            TransactionSource::Chain(transaction) => ResolvedTxBody::Chain(transaction),
        };
        Ok(ResolvedRawTransaction { source })
    }

    /// Read raw bytes without repeating the transaction-prefix lookup.
    pub fn transaction_raw_resolved(&self, transaction: ResolvedRawTransaction) -> Result<Vec<u8>> {
        match transaction.source {
            ResolvedTxBody::Memory { bytes, .. } => Ok(bytes),
            ResolvedTxBody::Chain(transaction) => {
                let read = self.read_indexer()?;
                let (_, index, _) = read
                    .revalidate_confirmed_tx(transaction)
                    .map_err(|error| self.transaction_error(error))?;
                self.transaction_raw_by_index(index, read.pin())
            }
        }
    }

    /// Hex-encode raw bytes without repeating transaction resolution.
    pub fn transaction_hex_resolved(&self, transaction: ResolvedRawTransaction) -> Result<String> {
        self.transaction_raw_resolved(transaction)
            .map(|bytes| bytes.to_lower_hex_string())
    }
}
