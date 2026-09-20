use bitview_plugin_indexer::SafeLengths;
use brk_error::Result;
use brk_types::{Addr, BlockHash, Height, OutputType, Transaction, TxIndex, Txid, TypeIndex};

use crate::Query;

/// A confirmed address transaction page resolved against one best-chain view.
#[derive(Debug)]
pub struct ResolvedAddrChainTxs {
    txindices: Vec<TxIndex>,
    anchor_height: Option<Height>,
    activity_anchor: BlockHash,
}

impl ResolvedAddrChainTxs {
    /// Latest relevant block, or the resolved tip while the page is empty.
    #[inline]
    pub const fn activity_anchor(&self) -> BlockHash {
        self.activity_anchor
    }
}

impl Query {
    /// Resolve an address page once before body loading.
    pub fn resolve_addr_chain_txs(
        &self,
        addr: &Addr,
        after_txid: Option<Txid>,
        limit: usize,
    ) -> Result<ResolvedAddrChainTxs> {
        let pin = self.pin_safe_lengths()?;
        let (output_type, type_index) = self.resolve_addr(addr)?;
        self.resolve_addr_chain_txs_for(output_type, type_index, after_txid, limit, &pin)
    }

    /// Load a previously resolved page after confirming its chain anchor survived.
    pub fn addr_txs_chain_resolved(
        &self,
        resolved: ResolvedAddrChainTxs,
    ) -> Result<Vec<Transaction>> {
        self.addr_txs_chain_at(resolved)
    }

    pub fn addr_txids(
        &self,
        addr: Addr,
        after_txid: Option<Txid>,
        limit: usize,
    ) -> Result<Vec<Txid>> {
        let pin = self.pin_safe_lengths()?;
        let txindices = self.addr_txindices(&addr, after_txid, limit, &pin)?;
        let txid_reader = self.indexer().vecs().transactions.txid.reader();
        Ok(txindices
            .into_iter()
            .map(|tx_index| txid_reader.get(tx_index))
            .collect())
    }

    fn addr_txindices(
        &self,
        addr: &Addr,
        after_txid: Option<Txid>,
        limit: usize,
        pin: &SafeLengths,
    ) -> Result<Vec<TxIndex>> {
        let (output_type, type_index) = self.resolve_addr(addr)?;
        self.addr_txindices_for(output_type, type_index, after_txid, limit, pin)
    }

    fn addr_txindices_for(
        &self,
        output_type: OutputType,
        type_index: TypeIndex,
        after_txid: Option<Txid>,
        limit: usize,
        pin: &SafeLengths,
    ) -> Result<Vec<TxIndex>> {
        let stores = self.indexer().stores();
        let safe = pin.lengths();
        if type_index >= safe.to_type_index(output_type) {
            return Err(self.missing_addr());
        }
        let tx_index_len = safe.tx_index;

        let before = after_txid
            .as_ref()
            .map(|txid| self.resolve_tx_index_bounded(txid))
            .transpose()?
            .unwrap_or(tx_index_len)
            .min(tx_index_len);
        Ok(stores
            .addr_tx_indexes_before(output_type, type_index, before)?
            .rev()
            .take(limit)
            .collect())
    }

    pub(crate) fn resolve_addr_chain_txs_for(
        &self,
        output_type: OutputType,
        type_index: TypeIndex,
        after_txid: Option<Txid>,
        limit: usize,
        pin: &SafeLengths,
    ) -> Result<ResolvedAddrChainTxs> {
        let txindices = self.addr_txindices_for(output_type, type_index, after_txid, limit, pin)?;
        let anchor = txindices
            .first()
            .map(|txindex| -> Result<_> {
                let height = self.confirmed_status_height_bounded(*txindex, pin.lengths())?;
                let hash = self.block_hash_by_height(height, pin)?;
                Ok((height, hash))
            })
            .transpose()?;
        let activity_anchor = match anchor {
            Some((_, hash)) => hash,
            None => self.tip_blockhash_at(pin)?,
        };

        Ok(ResolvedAddrChainTxs {
            txindices,
            anchor_height: anchor.map(|(height, _)| height),
            activity_anchor,
        })
    }

    /// Revalidate the selected prefix under rollback protection before reading.
    pub fn addr_txs_chain_at(&self, resolved: ResolvedAddrChainTxs) -> Result<Vec<Transaction>> {
        let guard = self.pin_safe_lengths()?;
        self.addr_txs_chain_pinned(resolved, &guard)
    }

    pub(crate) fn addr_txs_chain_pinned(
        &self,
        resolved: ResolvedAddrChainTxs,
        guard: &SafeLengths,
    ) -> Result<Vec<Transaction>> {
        if let Some(height) = resolved.anchor_height {
            self.validate_block_at_height(&resolved.activity_anchor, height, guard)?;
        }
        self.transactions_at_indices(&resolved.txindices, guard)
    }
}
