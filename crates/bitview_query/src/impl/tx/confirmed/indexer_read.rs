use bitview_plugin_indexer::SafeLengths;
use brk_types::{BlockHash, Height, TxIndex, Txid};
use vecdb::ReadableVec;

use super::ResolvedConfirmedTx;
use crate::{Error, Query, Result};

/// One logical read of this query's published indexer state.
///
/// Pins the published prefix against rollback without blocking ordinary appends.
/// Dependent reads must reuse this pin instead of acquiring another one.
pub(crate) struct IndexerRead<'a> {
    query: &'a Query,
    pin: SafeLengths,
}

impl Query {
    pub(crate) fn read_indexer(&self) -> Result<IndexerRead<'_>> {
        Ok(IndexerRead {
            query: self,
            pin: self.pin_safe_lengths()?,
        })
    }
}

impl IndexerRead<'_> {
    pub(crate) fn pin(&self) -> &SafeLengths {
        &self.pin
    }

    pub(crate) fn query(&self) -> &Query {
        self.query
    }

    /// Validate the exact transaction against this view's pinned bounds.
    fn validate_confirmed_position(&self, txid: &Txid, index: TxIndex) -> Result<Height> {
        let query = self.query;
        let safe = self.pin.lengths();
        if index >= safe.tx_index
            || query.indexer().vecs().transactions.txid.collect_one(index) != Some(*txid)
        {
            return Err(Error::UnknownTxid);
        }

        let height = query.confirmed_status_height_bounded(index, safe)?;
        if height >= safe.height {
            return Err(Error::UnknownTxid);
        }

        Ok(height)
    }

    /// Read a confirmed block hash inside the pinned prefix.
    fn confirmed_block_hash(&self, height: Height) -> Result<BlockHash> {
        let query = self.query;
        if height >= self.pin.lengths().height {
            return Err(Error::UnknownTxid);
        }
        let hash = query
            .indexer()
            .vecs()
            .blocks
            .blockhash
            .collect_one(height)
            .ok_or(Error::UnknownTxid)?;
        Ok(hash)
    }

    pub(crate) fn resolve_confirmed_tx(&self, txid: &Txid) -> Result<ResolvedConfirmedTx> {
        let (index, height) = self.resolve_confirmed_position(txid)?;
        let block_hash = self.confirmed_block_hash(height)?;
        Ok(ResolvedConfirmedTx {
            txid: *txid,
            index,
            height,
            block_hash,
        })
    }

    /// Resolve the position without reading a block hash the caller may not need.
    pub(crate) fn resolve_confirmed_position(&self, txid: &Txid) -> Result<(TxIndex, Height)> {
        let index = self.query.resolve_tx_index(txid)?;
        let height = self.validate_confirmed_position(txid, index)?;
        Ok((index, height))
    }

    /// Revalidate an async handoff without repeating its txid-prefix store lookup.
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
}
