use bitview_plugin_indexer::SafeLengths;
use bitview_primitives::TypeIndex;
use brk_types::{Address, BlockHash, Height, OutputType, Transaction, TxIndex, Txid};

use crate::{Error, Query, ReadPin, Result, api::addr::parse_addr};

/// A confirmed address transaction page resolved against one best-chain view.
#[derive(Debug)]
pub struct ResolvedAddrChainTxs {
    pub(super) txindices: Vec<TxIndex>,
    pub(super) script: Option<(OutputType, TypeIndex)>,
    anchor_height: Option<Height>,
    activity_anchor: BlockHash,
}

impl ResolvedAddrChainTxs {
    pub(super) fn empty(pin: &SafeLengths) -> Self {
        Self {
            txindices: Vec::new(),
            script: None,
            anchor_height: None,
            activity_anchor: pin.tip(),
        }
    }

    /// Both body readers revalidate the same selection before reading its columns.
    pub(super) fn revalidate(&self, query: &Query, pin: &ReadPin) -> Result<()> {
        if let Some(height) = self.anchor_height {
            query.validate_block_at_height(&self.activity_anchor, height, pin)?;
        }
        Ok(())
    }

    /// Latest relevant block, or the resolved tip while the page is empty.
    #[inline]
    pub const fn activity_anchor(&self) -> BlockHash {
        self.activity_anchor
    }
}

impl Query {
    /// Resolve an address page once before body loading. A never-confirmed address has an
    /// empty page, anchored at the tip.
    pub fn resolve_addr_chain_txs(
        &self,
        addr: &Address,
        after_txid: Option<Txid>,
        limit: usize,
    ) -> Result<ResolvedAddrChainTxs> {
        let bytes = parse_addr(addr)?;
        let pin = self.pin_safe_lengths()?;
        match self.find_addr_bytes(&bytes, &pin)? {
            Some((output_type, type_index)) => {
                self.resolve_addr_chain_txs_for(output_type, type_index, after_txid, limit, &pin)
            }
            None => Ok(ResolvedAddrChainTxs::empty(&pin)),
        }
    }

    /// Load a previously resolved page after confirming its chain anchor survived.
    pub fn addr_txs_chain_resolved(
        &self,
        resolved: ResolvedAddrChainTxs,
    ) -> Result<Vec<Transaction>> {
        self.addr_txs_chain_pinned(resolved, &self.pin_safe_lengths()?)
    }

    /// The confirmed transactions of an address `pin` holds, oldest first: those before the
    /// `before` cursor, or all of them. A cursor outside the address's history (unknown, or
    /// another address's transaction) has none before it, as Esplora pages.
    pub(crate) fn addr_tx_history(
        &self,
        output_type: OutputType,
        type_index: TypeIndex,
        before: Option<&Txid>,
        pin: &SafeLengths,
    ) -> Result<impl DoubleEndedIterator<Item = brk_error::Result<TxIndex>> + '_> {
        let before = self.addr_chain_cursor(before, pin)?;
        self.addr_tx_history_before(output_type, type_index, before, pin)
    }

    /// Preserve history's empty-page contract for an unknown cursor. Summary validates its
    /// cursor first and passes the already resolved index instead.
    fn addr_chain_cursor(&self, before: Option<&Txid>, pin: &SafeLengths) -> Result<Option<TxIndex>> {
        before.map(|txid| match self.resolve_tx_index_bounded(txid, pin) {
            Ok(index) => Ok(index),
            Err(Error::UnknownTxid) => Ok(TxIndex::ZERO),
            Err(error) => Err(error),
        }).transpose()
    }

    fn addr_tx_history_before(
        &self,
        output_type: OutputType,
        type_index: TypeIndex,
        before: Option<TxIndex>,
        pin: &SafeLengths,
    ) -> Result<impl DoubleEndedIterator<Item = brk_error::Result<TxIndex>> + '_> {
        debug_assert!(type_index < pin.lengths().to_type_index(output_type));
        let before = match before {
            Some(index) if index != TxIndex::ZERO
                && self.indexer().stores().addr_has_tx(output_type, type_index, index)? => index,
            Some(_) => TxIndex::ZERO,
            None => pin.lengths().tx_index,
        };
        Ok(self.indexer().stores().addr_tx_indexes_before(output_type, type_index, before)?)
    }

    pub(crate) fn resolve_addr_chain_txs_for(
        &self,
        output_type: OutputType,
        type_index: TypeIndex,
        after_txid: Option<Txid>,
        limit: usize,
        pin: &ReadPin,
    ) -> Result<ResolvedAddrChainTxs> {
        let before = self.addr_chain_cursor(after_txid.as_ref(), pin)?;
        self.resolve_addr_chain_txs_before(output_type, type_index, before, limit, pin)
    }

    pub(super) fn resolve_addr_chain_txs_before(
        &self,
        output_type: OutputType,
        type_index: TypeIndex,
        before: Option<TxIndex>,
        limit: usize,
        pin: &ReadPin,
    ) -> Result<ResolvedAddrChainTxs> {
        let txindices = self
            .addr_tx_history_before(output_type, type_index, before, pin)?
            .rev()
            .take(limit)
            .collect::<brk_error::Result<Vec<_>>>()?;
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
            None => pin.tip(),
        };

        Ok(ResolvedAddrChainTxs {
            txindices,
            script: Some((output_type, type_index)),
            anchor_height: anchor.map(|(height, _)| height),
            activity_anchor,
        })
    }

    pub(crate) fn addr_txs_chain_pinned(
        &self,
        resolved: ResolvedAddrChainTxs,
        guard: &ReadPin,
    ) -> Result<Vec<Transaction>> {
        resolved.revalidate(self, guard)?;
        self.transactions_at_indices(&resolved.txindices, guard)
    }
}
