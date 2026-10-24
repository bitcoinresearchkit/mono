use std::sync::Arc;

use bitview_plugin_indexer::SafeLengths;
use brk_mempool::PublishedTx;
use brk_types::{Address, Transaction, Txid};

use super::ResolvedAddrChainTxs;
use crate::{
    Error, Query, RepresentationId, Result,
    r#impl::{
        addr::{mempool::page_identity, parse_addr},
        mempool::anchored,
    },
};

/// One address page with frozen mempool bodies and a guarded confirmed selection.
pub struct ResolvedAddrTxs {
    guard: SafeLengths,
    mempool: Vec<PublishedTx>,
    chain: Option<ResolvedAddrChainTxs>,
}

impl ResolvedAddrTxs {
    /// The confirmed selection is fixed by its activity anchor; the captured mempool bodies by
    /// their digests. Revalidation loads and encodes no transaction body.
    pub fn identity(&self) -> RepresentationId {
        let anchor = self
            .chain
            .as_ref()
            .map(ResolvedAddrChainTxs::activity_anchor);
        page_identity(anchor, &self.mempool)
    }
}

impl Query {
    /// Select both parts against the same indexed chain, `limit` transactions in all: the
    /// mempool's first, the chain's filling the rest, as mempool.space pages. `after_txid`
    /// continues from a mempool or a confirmed transaction (a confirmed one skips the
    /// mempool); one that's neither is `UnknownAfterTxid`. Mempool bodies are immutable shared
    /// values; a prefix pin keeps selected chain rows stable.
    pub fn resolve_addr_txs(
        &self,
        addr: &Address,
        after_txid: Option<Txid>,
        limit: usize,
    ) -> Result<ResolvedAddrTxs> {
        let addr = parse_addr(addr)?;
        let (mempool, guard) = self.mempool_then_pin()?;
        let chain_addr = self.find_addr_bytes(&addr, &guard)?;
        let tip = guard.tip();
        let confirmed_cursor = match after_txid {
            None => false,
            Some(txid) => match self.resolve_tx_index_bounded(&txid, &guard) {
                Ok(_) => true,
                Err(Error::UnknownTxid) => false,
                Err(error) => return Err(error),
            },
        };
        let mempool = match (&mempool, confirmed_cursor) {
            (Some(mempool), false) => {
                if let Some(txid) = &after_txid
                    && !anchored(mempool.contains_txid(txid, &tip))?
                {
                    return Err(Error::UnknownAfterTxid);
                }
                anchored(mempool.addr_txs(&addr, after_txid.as_ref(), limit, &tip))?
            }
            (None, false) if after_txid.is_some() => return Err(Error::UnknownAfterTxid),
            _ => Vec::new(),
        };
        let chain_after = after_txid.filter(|_| confirmed_cursor);
        let chain_limit = limit - mempool.len();
        let chain = chain_addr
            .map(|(output_type, type_index)| {
                self.resolve_addr_chain_txs_for(
                    output_type,
                    type_index,
                    chain_after,
                    chain_limit,
                    &guard,
                )
            })
            .transpose()?;
        Ok(ResolvedAddrTxs {
            guard: guard.into_inner(),
            mempool,
            chain,
        })
    }

    pub fn addr_txs_resolved(&self, resolved: ResolvedAddrTxs) -> Result<Vec<Arc<Transaction>>> {
        let ResolvedAddrTxs {
            guard,
            mempool,
            chain,
        } = resolved;
        let mut txs: Vec<Arc<Transaction>> =
            mempool.into_iter().map(|published| published.tx).collect();
        if let Some(chain) = chain {
            txs.extend(
                self.addr_txs_chain_pinned(chain, &self.read_pin(guard))?
                    .into_iter()
                    .map(Arc::new),
            );
        }
        Ok(txs)
    }
}
