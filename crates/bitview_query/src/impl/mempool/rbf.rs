use brk_error::Result;
use brk_mempool::{RbfForTx, RbfNode};
use brk_types::{CheckedSub, FeeRate, RbfResponse, RbfTx, ReplacementNode, Timestamp, Txid};
use vecdb::ReadableVec;

use crate::{Query, RepresentationId};

use super::serialize_json;

const RECENT_REPLACEMENTS_LIMIT: usize = 25;

/// An exact owned RBF tree resolved before an async response handoff.
pub struct ResolvedRbf {
    source: RbfForTx,
    identity: Option<RepresentationId>,
}

impl ResolvedRbf {
    fn new(source: RbfForTx) -> Self {
        let identity = source
            .is_empty()
            .then(|| serialize_json(&RbfResponse::EMPTY).1);
        Self { source, identity }
    }

    /// Present when the final empty response was completely resolved in preflight.
    pub fn identity(&self) -> Option<RepresentationId> {
        self.identity
    }
}

impl Query {
    /// Resolve the exact owned replacement tree once under the mempool lock.
    pub fn resolve_rbf(&self, txid: &Txid) -> Result<ResolvedRbf> {
        Ok(ResolvedRbf::new(self.require_mempool()?.rbf_for_tx(txid)))
    }

    /// RBF history for a tx. Matches mempool.space's
    /// `GET /api/v1/tx/:txid/rbf`.
    pub fn tx_rbf(&self, txid: &Txid) -> Result<RbfResponse> {
        Ok(self.tx_rbf_resolved(self.resolve_rbf(txid)?))
    }

    /// Enrich an already resolved tree without repeating its mempool lookup.
    pub fn tx_rbf_resolved(&self, rbf: ResolvedRbf) -> RbfResponse {
        let RbfForTx { root, replaces } = rbf.source;
        let replacements = root.map(|node| self.enrich_rbf_node(node, None));
        let replaces = (!replaces.is_empty()).then_some(replaces);
        RbfResponse {
            replacements,
            replaces,
        }
    }

    /// Serialize an already resolved tree with its exact content identity.
    pub fn tx_rbf_json_resolved(&self, rbf: ResolvedRbf) -> Result<(Vec<u8>, RepresentationId)> {
        let response = self.tx_rbf_resolved(rbf);
        Ok(serialize_json(&response))
    }

    /// Recent RBF replacements. Matches mempool.space's
    /// `GET /api/v1/replacements` and `GET /api/v1/fullrbf/replacements`.
    /// Most-recent first, capped at 25. `full_rbf_only` keeps only
    /// trees with at least one non-signaling predecessor.
    pub fn recent_replacements(&self, full_rbf_only: bool) -> Result<Vec<ReplacementNode>> {
        Ok(self
            .require_mempool()?
            .recent_rbf_trees(full_rbf_only, RECENT_REPLACEMENTS_LIMIT)
            .into_iter()
            .map(|node| self.enrich_rbf_node(node, None))
            .collect())
    }

    /// Serialize recent replacements with their exact content identity.
    pub fn recent_replacements_json(
        &self,
        full_rbf_only: bool,
    ) -> Result<(Vec<u8>, RepresentationId)> {
        let replacements = self.recent_replacements(full_rbf_only)?;
        Ok(serialize_json(&replacements))
    }

    /// Layer `mined` and effective fee rate onto an owned RBF tree.
    fn enrich_rbf_node(&self, node: RbfNode, successor_time: Option<Timestamp>) -> ReplacementNode {
        let interval = successor_time
            .and_then(|time| time.checked_sub(node.first_seen))
            .map(|duration| *duration);
        let (mined, rate) = self.rbf_status_and_rate(&node);
        let first_seen = node.first_seen;
        let replaces = node
            .replaces
            .into_iter()
            .map(|child| self.enrich_rbf_node(child, Some(first_seen)))
            .collect();
        ReplacementNode {
            tx: RbfTx {
                txid: node.txid,
                fee: node.fee,
                vsize: node.vsize,
                value: node.value,
                rate,
                time: first_seen,
                rbf: node.rbf,
                full_rbf: Some(node.full_rbf),
            },
            time: first_seen,
            full_rbf: node.full_rbf,
            interval,
            mined,
            replaces,
        }
    }

    /// Resolve confirmation and its effective rate with one exact txid lookup.
    fn rbf_status_and_rate(&self, node: &RbfNode) -> (Option<bool>, FeeRate) {
        let confirmed_rate = self
            .resolve_confirmed_tx(&node.txid)
            .ok()
            .and_then(|transaction| {
                let (_, index, _) = self.revalidate_confirmed_tx(transaction).ok()?;
                let rate = self
                    .plugins()
                    .transactions
                    .fees
                    .effective_fee_rate
                    .tx_index
                    .collect_one(index);
                self.revalidate_confirmed_tx(transaction).ok()?;
                Some(rate)
            });
        let mined = confirmed_rate.is_some().then_some(true);
        let rate = if node.in_mempool {
            node.rate
        } else {
            confirmed_rate.flatten().unwrap_or(node.rate)
        };
        (mined, rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::representation_id::content_hash;

    #[test]
    fn empty_rbf_is_identified_by_its_exact_json() {
        let resolved = ResolvedRbf::new(RbfForTx::default());
        let bytes = serde_json::to_vec(&RbfResponse::EMPTY).unwrap();
        assert!(matches!(
            resolved.identity(),
            Some(RepresentationId::Content(hash)) if hash == content_hash(&bytes)
        ));
    }
}
