use std::{str::FromStr, sync::Arc};

use brk_error::{Error, Result};
use brk_types::{Addr, AddrBytes, OutputType, TypeIndex};

use super::{ResolvedAddrChainTxs, ResolvedAddrTxs};
use crate::{
    Query, RepresentationId,
    r#impl::{
        CachedJson,
        addr::{mempool::AddrMempoolTxsPagePreflight, resolve},
    },
};

#[derive(Clone, Copy)]
pub(super) struct AddrTxsLimits {
    pub(super) mempool_limit: usize,
    pub(super) chain_floor: usize,
    pub(super) total_target: usize,
}

impl AddrTxsLimits {
    fn chain_limit(self, mempool_count: usize) -> usize {
        self.total_target
            .saturating_sub(mempool_count)
            .max(self.chain_floor)
    }
}

pub enum AddrTxsPreflight {
    Cached(Arc<[u8]>, RepresentationId),
    Chain(ResolvedAddrChainTxs),
    Resolved(Box<ResolvedAddrTxs>),
}

impl AddrTxsPreflight {
    fn cached((bytes, identity): (Arc<[u8]>, RepresentationId)) -> Self {
        Self::Cached(bytes, identity)
    }
}

impl Query {
    /// Resolve the exact sources of a combined address page before body loading.
    pub fn addr_txs_json_preflight(
        &self,
        addr: &Addr,
        mempool_limit: usize,
        chain_floor: usize,
        total_target: usize,
    ) -> Result<AddrTxsPreflight> {
        let limits = AddrTxsLimits {
            mempool_limit,
            chain_floor,
            total_target,
        };
        let addr = AddrBytes::from_str(addr)?;
        let chain_tip = self.tip_blockhash();
        let chain_addr = resolve::find_addr_bytes(self, &addr)?;

        let Some(mempool) = self.mempool() else {
            return self.addr_txs_chain_only_preflight(chain_addr, limits.chain_limit(0));
        };
        let mempool =
            self.addr_mempool_txs_json_preflight_for(mempool, addr.clone(), limits.mempool_limit);
        let Some(mempool_source) = mempool.source() else {
            return self.addr_txs_chain_only_preflight(chain_addr, limits.chain_limit(0));
        };

        let chain =
            self.resolve_optional_addr_chain_txs(chain_addr, limits.chain_limit(mempool.count()))?;
        if let Some(chain_hash) = chain.as_ref().and_then(ResolvedAddrChainTxs::block_hash)
            && let Some(cached) =
                self.0
                    .addr_txs_cache
                    .current(&addr, limits, mempool_source, chain_hash)
        {
            return Ok(AddrTxsPreflight::cached(cached));
        }
        if chain
            .as_ref()
            .map(ResolvedAddrChainTxs::is_empty)
            .unwrap_or(true)
            && let AddrMempoolTxsPagePreflight::Cached(json) = mempool
        {
            return Ok(AddrTxsPreflight::cached(json.into_value()));
        }

        Ok(AddrTxsPreflight::Resolved(Box::new(ResolvedAddrTxs::new(
            addr, mempool, chain_addr, chain, chain_tip, limits,
        ))))
    }

    /// Build a cold mixed page once and cache only a stable exact representation.
    pub fn addr_txs_json_resolved(
        &self,
        resolved: ResolvedAddrTxs,
    ) -> Result<(Arc<[u8]>, RepresentationId)> {
        let (addr, mempool, mut chain_addr, mut chain, chain_tip, limits) = resolved.into_parts();
        let expected_count = mempool.count();
        let mempool = match mempool {
            AddrMempoolTxsPagePreflight::Cached(json) => json,
            AddrMempoolTxsPagePreflight::Resolved(resolved) => {
                self.addr_mempool_txs_json_resolved_page(resolved)?
            }
        };

        let build_tip = self.tip_blockhash();
        if build_tip != chain_tip || mempool.count() != expected_count {
            chain_addr = resolve::find_addr_bytes(self, &addr)?;
            chain = self
                .resolve_optional_addr_chain_txs(chain_addr, limits.chain_limit(mempool.count()))?;
        }
        if mempool.count() == 0 && chain_addr.is_none() {
            return Err(Error::UnknownAddr);
        }

        let chain_anchor = chain.as_ref().and_then(ResolvedAddrChainTxs::anchor);
        let chain_txs = chain
            .map(|resolved| self.addr_txs_chain_resolved(resolved))
            .transpose()?
            .unwrap_or_default();
        if chain_txs.is_empty() {
            return Ok(mempool.into_value());
        }

        let chain_json = serde_json::to_vec(&chain_txs).unwrap();
        if mempool.count() == 0 {
            return Ok(CachedJson::from_vec(chain_json).value());
        }

        let json = CachedJson::from_vec(join_json_arrays(mempool.bytes(), &chain_json));
        let Some(mempool_source) = mempool.source() else {
            return Ok(json.value());
        };
        let Some((chain_height, chain_hash)) = chain_anchor else {
            return Ok(json.value());
        };
        let mempool_is_current = self
            .mempool()
            .and_then(|mempool| mempool.addr_txs_source(&addr, limits.mempool_limit))
            == Some(mempool_source);
        let chain_is_current = self.tip_blockhash() == build_tip
            && self
                .validate_block_at_height(&chain_hash, chain_height)
                .is_ok();
        if !mempool_is_current || !chain_is_current {
            return Ok(json.value());
        }

        Ok(self
            .0
            .addr_txs_cache
            .insert(addr, limits, mempool_source, chain_hash, json))
    }

    fn addr_txs_chain_only_preflight(
        &self,
        chain_addr: Option<(OutputType, TypeIndex)>,
        limit: usize,
    ) -> Result<AddrTxsPreflight> {
        let Some((output_type, type_index)) = chain_addr else {
            return Err(Error::UnknownAddr);
        };
        self.resolve_addr_chain_txs_for(output_type, type_index, None, limit)
            .map(AddrTxsPreflight::Chain)
    }

    fn resolve_optional_addr_chain_txs(
        &self,
        chain_addr: Option<(OutputType, TypeIndex)>,
        limit: usize,
    ) -> Result<Option<ResolvedAddrChainTxs>> {
        chain_addr
            .map(|(output_type, type_index)| {
                self.resolve_addr_chain_txs_for(output_type, type_index, None, limit)
            })
            .transpose()
    }
}

fn join_json_arrays(first: &[u8], second: &[u8]) -> Vec<u8> {
    debug_assert!(first.starts_with(b"[") && first.ends_with(b"]"));
    debug_assert!(second.starts_with(b"[") && second.ends_with(b"]"));
    let mut joined = Vec::with_capacity(first.len() + second.len());
    joined.extend_from_slice(&first[..first.len() - 1]);
    joined.push(b',');
    joined.extend_from_slice(&second[1..]);
    joined
}

#[cfg(test)]
mod tests {
    use super::join_json_arrays;

    #[test]
    fn joins_non_empty_json_arrays_without_reserializing() {
        assert_eq!(
            join_json_arrays(br#"[{"a":1}]"#, br#"[{"b":2}]"#),
            br#"[{"a":1},{"b":2}]"#
        );
    }
}
