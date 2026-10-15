use std::sync::Arc;

use bitview_plugin_indexer::SafeLengths;
use brk_mempool::PublishedTx;
use brk_types::{Address, Transaction};

use super::ResolvedAddrChainTxs;
use crate::{
    Query, RepresentationId, Result,
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
    /// Select both parts against the same indexed chain. Mempool bodies are
    /// immutable shared values; a prefix pin keeps selected chain rows stable.
    pub fn resolve_addr_txs(
        &self,
        addr: &Address,
        mempool_limit: usize,
        chain_floor: usize,
        total_target: usize,
    ) -> Result<ResolvedAddrTxs> {
        let addr = parse_addr(addr)?;
        let (mempool, guard) = self.mempool_then_pin()?;
        let chain_addr = self.find_addr_bytes(&addr, &guard)?;
        let tip = guard.tip();
        let mempool = mempool
            .map(|mempool| anchored(mempool.addr_txs(&addr, mempool_limit, &tip)))
            .transpose()?
            .unwrap_or_default();
        let chain_limit = total_target.saturating_sub(mempool.len()).max(chain_floor);
        let chain = chain_addr
            .map(|(output_type, type_index)| {
                self.resolve_addr_chain_txs_for(output_type, type_index, None, chain_limit, &guard)
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
        let mut mempool: Vec<Arc<Transaction>> =
            mempool.into_iter().map(|published| published.tx).collect();
        if let Some(chain) = chain {
            mempool.extend(
                self.addr_txs_chain_pinned(chain, &self.read_pin(guard))?
                    .into_iter()
                    .map(Arc::new),
            );
        }
        Ok(mempool)
    }
}
