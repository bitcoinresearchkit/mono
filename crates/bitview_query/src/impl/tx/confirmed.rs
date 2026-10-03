use brk_types::{BlockHash, Height, TxIndex, Txid};

use crate::{Query, RepresentationId, Result};

mod indexer_read;

pub(crate) use indexer_read::IndexerRead;

/// An exact confirmed transaction identified by its published chain position.
///
/// Private fields prevent callers from constructing mismatched transaction,
/// height, and block-hash combinations. Query methods revalidate the token
/// after an async handoff.
///
/// Revalidation alone is not exposed on `Query`: the internal read view must
/// retain rollback protection through the subsequent dependent reads.
///
#[derive(Debug, Clone, Copy)]
pub struct ResolvedConfirmedTx {
    txid: Txid,
    index: TxIndex,
    height: Height,
    block_hash: BlockHash,
}

impl ResolvedConfirmedTx {
    #[inline]
    pub const fn identity(self) -> RepresentationId {
        RepresentationId::Block(self.block_hash)
    }
}

impl Query {
    /// Resolve an exact transaction confirmed in the published best chain.
    ///
    /// The public entry point acquires its own read view. The old caller-guarded
    /// entry point is deliberately unavailable.
    ///
    pub fn resolve_confirmed_tx(&self, txid: &Txid) -> Result<ResolvedConfirmedTx> {
        self.read_indexer()?
            .resolve_confirmed_tx(txid)
            .map_err(|error| self.transaction_error(error))
    }
}
