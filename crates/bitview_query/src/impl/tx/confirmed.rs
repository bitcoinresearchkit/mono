use brk_error::{Error, Result};
use brk_types::{BlockHash, Height, TxIndex, Txid};
use vecdb::ReadableVec;

use crate::{Query, RepresentationId};

/// An exact confirmed transaction identified by its published chain position.
///
/// Private fields prevent callers from constructing mismatched transaction,
/// height, and block-hash combinations. Query methods revalidate the token
/// after an async handoff.
#[derive(Debug, Clone, Copy)]
pub struct ResolvedConfirmedTx {
    txid: Txid,
    index: TxIndex,
    height: Height,
    block_hash: BlockHash,
}

impl ResolvedConfirmedTx {
    /// The block hash that identifies the containing best-chain block.
    #[inline]
    pub const fn block_hash(self) -> BlockHash {
        self.block_hash
    }

    #[inline]
    pub const fn identity(self) -> RepresentationId {
        RepresentationId::Block {
            hash: self.block_hash,
            height: self.height,
        }
    }

    #[inline]
    pub fn is_deeply_confirmed(self, current_height: Height) -> bool {
        self.height.is_deeply_confirmed(current_height)
    }
}

impl Query {
    /// Resolve an exact transaction confirmed in the published best chain.
    pub fn resolve_confirmed_tx(&self, txid: &Txid) -> Result<ResolvedConfirmedTx> {
        let (index, height) = self.resolve_confirmed_position(txid)?;
        let block_hash = self.confirmed_block_hash(height)?;

        Ok(ResolvedConfirmedTx {
            txid: *txid,
            index,
            height,
            block_hash,
        })
    }

    /// Preserve the existing `(TxIndex, Height)` API without resolving a block
    /// hash that its callers may not need.
    pub(super) fn resolve_confirmed_position(&self, txid: &Txid) -> Result<(TxIndex, Height)> {
        let index = super::resolve_tx_index(self, txid)?;
        let height = self.validate_confirmed_position(txid, index)?;
        Ok((index, height))
    }

    /// Revalidate a token without repeating its txid-prefix store lookup.
    pub(crate) fn revalidate_confirmed_tx(
        &self,
        tx: ResolvedConfirmedTx,
    ) -> Result<(Txid, TxIndex, Height)> {
        // The block hash commits to the transaction list and its ordering.
        // Since the token was exactly verified when constructed, confirming
        // that the same block remains at the same height is sufficient.
        if self.confirmed_block_hash(tx.height)? != tx.block_hash {
            return Err(Error::UnknownTxid);
        }
        Ok((tx.txid, tx.index, tx.height))
    }

    /// Validate between safe-bound snapshots. Rollback lowers the published
    /// bounds before mutating transaction vectors or confirmed-height data.
    fn validate_confirmed_position(&self, txid: &Txid, index: TxIndex) -> Result<Height> {
        let safe = self.safe_lengths();
        if index >= safe.tx_index
            || self.indexer().vecs().transactions.txid.collect_one(index) != Some(*txid)
        {
            return Err(Error::UnknownTxid);
        }

        let height = super::confirmed_status_height(self, index)?;
        let safe = self.safe_lengths();
        if index >= safe.tx_index || height >= safe.height {
            return Err(Error::UnknownTxid);
        }

        Ok(height)
    }

    /// Read a confirmed block hash between safe-bound snapshots.
    fn confirmed_block_hash(&self, height: Height) -> Result<BlockHash> {
        if height >= self.safe_lengths().height {
            return Err(Error::UnknownTxid);
        }
        let hash = self
            .indexer()
            .vecs()
            .blocks
            .blockhash
            .get(height)
            .ok_or(Error::UnknownTxid)?;
        if height >= self.safe_lengths().height {
            return Err(Error::UnknownTxid);
        }
        Ok(hash)
    }
}
