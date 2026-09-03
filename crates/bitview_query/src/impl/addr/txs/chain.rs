use brk_error::Result;
use brk_types::{Addr, BlockHash, Height, OutputType, Transaction, TxIndex, Txid, TypeIndex};

use crate::Query;

/// A confirmed address transaction page resolved against one best-chain view.
#[derive(Debug)]
pub struct ResolvedAddrChainTxs {
    txindices: Vec<TxIndex>,
    anchor: Option<(Height, BlockHash)>,
    activity_anchor: BlockHash,
}

impl ResolvedAddrChainTxs {
    /// The newest block represented by this page, when the page is non-empty.
    #[inline]
    pub fn block_hash(&self) -> Option<BlockHash> {
        self.anchor.map(|(_, hash)| hash)
    }

    /// Latest relevant block, or the resolved tip while the page is empty.
    #[inline]
    pub const fn activity_anchor(&self) -> BlockHash {
        self.activity_anchor
    }

    pub(super) fn is_empty(&self) -> bool {
        self.txindices.is_empty()
    }

    pub(super) fn anchor(&self) -> Option<(Height, BlockHash)> {
        self.anchor
    }
}

impl Query {
    pub fn addr_txs_chain(
        &self,
        addr: &Addr,
        after_txid: Option<Txid>,
        limit: usize,
    ) -> Result<Vec<Transaction>> {
        let txindices = self.addr_txindices(addr, after_txid, limit)?;
        self.transactions_by_indices(&txindices)
    }

    /// Resolve an address page once before body loading.
    pub fn resolve_addr_chain_txs(
        &self,
        addr: &Addr,
        after_txid: Option<Txid>,
        limit: usize,
    ) -> Result<ResolvedAddrChainTxs> {
        let (output_type, type_index) = super::super::resolve::resolve_addr(self, addr)?;
        self.resolve_addr_chain_txs_for(output_type, type_index, after_txid, limit)
    }

    pub(super) fn resolve_addr_chain_txs_for(
        &self,
        output_type: OutputType,
        type_index: TypeIndex,
        after_txid: Option<Txid>,
        limit: usize,
    ) -> Result<ResolvedAddrChainTxs> {
        let txindices = self.addr_txindices_for(output_type, type_index, after_txid, limit)?;
        let anchor = txindices
            .first()
            .map(|txindex| -> Result<_> {
                let height = super::super::super::tx::confirmed_status_height(self, *txindex)?;
                let hash = self.block_hash_by_height(height)?;
                Ok((height, hash))
            })
            .transpose()?;
        let activity_anchor = anchor
            .map(|(_, hash)| hash)
            .unwrap_or_else(|| self.tip_blockhash());

        Ok(ResolvedAddrChainTxs {
            txindices,
            anchor,
            activity_anchor,
        })
    }

    /// Load a previously resolved page after confirming its chain anchor survived.
    pub fn addr_txs_chain_resolved(
        &self,
        resolved: ResolvedAddrChainTxs,
    ) -> Result<Vec<Transaction>> {
        if let Some((height, hash)) = resolved.anchor {
            self.validate_block_at_height(&hash, height)?;
        }
        self.transactions_by_indices(&resolved.txindices)
    }

    pub fn addr_txids(
        &self,
        addr: Addr,
        after_txid: Option<Txid>,
        limit: usize,
    ) -> Result<Vec<Txid>> {
        let txindices = self.addr_txindices(&addr, after_txid, limit)?;
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
    ) -> Result<Vec<TxIndex>> {
        let (output_type, type_index) = super::super::resolve::resolve_addr(self, addr)?;
        self.addr_txindices_for(output_type, type_index, after_txid, limit)
    }

    fn addr_txindices_for(
        &self,
        output_type: OutputType,
        type_index: TypeIndex,
        after_txid: Option<Txid>,
        limit: usize,
    ) -> Result<Vec<TxIndex>> {
        let stores = self.indexer().stores();
        let tx_index_len = self.safe_lengths().tx_index;

        if let Some(after_txid) = after_txid {
            let (after_tx_index, _) = self.resolve_tx(&after_txid)?;
            Ok(stores
                .addr_tx_indexes_before(output_type, type_index, after_tx_index)?
                .rev()
                .filter(|tx_index| *tx_index < tx_index_len)
                .take(limit)
                .collect())
        } else {
            Ok(stores
                .addr_tx_indexes(output_type, type_index)?
                .rev()
                .filter(|tx_index| *tx_index < tx_index_len)
                .take(limit)
                .collect())
        }
    }
}
