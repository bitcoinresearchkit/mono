use std::sync::Arc;

use bitview_plugin_indexer::SafeLengths;
use brk_types::{Addr, BlockHash, Transaction};

use super::ResolvedAddrChainTxs;
use crate::{Query, Result, r#impl::addr::parse_addr};

/// One address page with frozen mempool bodies and a guarded confirmed selection.
pub struct ResolvedAddrTxs {
    guard: SafeLengths,
    mempool: Vec<Arc<Transaction>>,
    chain: Option<ResolvedAddrChainTxs>,
}

impl ResolvedAddrTxs {
    /// Latest relevant block for the captured confirmed page.
    pub fn chain_anchor(&self) -> Option<BlockHash> {
        self.chain
            .as_ref()
            .map(ResolvedAddrChainTxs::activity_anchor)
    }

    pub fn mempool_transactions(&self) -> &[Arc<Transaction>] {
        &self.mempool
    }
}

impl Query {
    /// Select both parts against the same indexed chain. Mempool bodies are
    /// immutable shared values; a prefix pin keeps selected chain rows stable.
    pub fn resolve_addr_txs(
        &self,
        addr: &Addr,
        mempool_limit: usize,
        chain_floor: usize,
        total_target: usize,
    ) -> Result<ResolvedAddrTxs> {
        let addr = parse_addr(addr)?;
        let guard = self.pin_safe_lengths()?;
        let chain_addr = self.find_addr_bytes(&addr)?;
        let tip = self.tip_blockhash_at(&guard)?;
        let mempool = self
            .mempool()
            .map(|mempool| mempool.addr_txs(&addr, mempool_limit, &tip))
            .transpose()?
            .unwrap_or_default();
        if chain_addr.is_none() && mempool.is_empty() {
            return Err(self.missing_addr());
        }
        let chain_limit = total_target.saturating_sub(mempool.len()).max(chain_floor);
        let chain = chain_addr
            .map(|(output_type, type_index)| {
                self.resolve_addr_chain_txs_for(output_type, type_index, None, chain_limit, &guard)
            })
            .transpose()?;
        Ok(ResolvedAddrTxs {
            guard,
            mempool,
            chain,
        })
    }

    pub fn addr_txs_resolved(&self, resolved: ResolvedAddrTxs) -> Result<Vec<Arc<Transaction>>> {
        let ResolvedAddrTxs {
            guard,
            mut mempool,
            chain,
        } = resolved;
        if let Some(chain) = chain {
            mempool.extend(
                self.addr_txs_chain_pinned(chain, &guard)?
                    .into_iter()
                    .map(Arc::new),
            );
        }
        Ok(mempool)
    }
}
