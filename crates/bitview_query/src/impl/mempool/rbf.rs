use bitview_types::{RbfResponse, RbfTx, ReplacementNode};
use brk_mempool::{RbfForTx, RbfNode};
use brk_types::{BlockHash, CheckedSub, FeeRate, Timestamp, Txid};
use vecdb::ReadableVec;

use super::serialize_json;
use crate::{
    Error, OptionData, Query, RepresentationId, Result, r#impl::tx::confirmed::IndexerRead,
};

const RECENT_REPLACEMENTS_LIMIT: usize = 25;

/// An exact owned RBF tree resolved before an async response handoff.
pub struct ResolvedRbf {
    source: RbfForTx,
    tip: BlockHash,
}

impl ResolvedRbf {
    /// Present when the final empty response was completely resolved in preflight.
    pub fn identity(&self) -> Option<RepresentationId> {
        self.source
            .is_empty()
            .then(|| serialize_json(&RbfResponse::EMPTY).1)
    }
}

impl Query {
    /// Resolve the exact owned replacement tree from one mempool publication.
    pub fn resolve_rbf(&self, txid: &Txid) -> Result<ResolvedRbf> {
        let pin = self.pin_safe_lengths()?;
        let tip = self.tip_blockhash_at(&pin)?;
        Ok(ResolvedRbf {
            source: self.require_mempool()?.rbf_for_tx(txid, &tip)?,
            tip,
        })
    }

    /// Enrich an already resolved tree without repeating its mempool lookup.
    fn tx_rbf_resolved(&self, rbf: ResolvedRbf) -> Result<RbfResponse> {
        if rbf.source.is_empty() {
            return Ok(RbfResponse::EMPTY);
        }
        let _publication = self.read_publication()?;
        let read = self.read_indexer()?;
        if self.tip_blockhash() != rbf.tip {
            return Err(Error::StateUpdating);
        }
        let RbfForTx { root, replaces } = rbf.source;
        let replacements = root
            .map(|node| read.enrich_rbf_node(node, None))
            .transpose()?;
        let replaces = (!replaces.is_empty()).then_some(replaces);
        Ok(RbfResponse {
            replacements,
            replaces,
        })
    }

    /// Serialize an already resolved tree with its exact content identity.
    pub fn tx_rbf_json_resolved(&self, rbf: ResolvedRbf) -> Result<(Vec<u8>, RepresentationId)> {
        let response = self.tx_rbf_resolved(rbf)?;
        Ok(serialize_json(&response))
    }

    /// Recent RBF replacements. Matches mempool.space's
    /// `GET /api/v1/replacements` and `GET /api/v1/fullrbf/replacements`.
    /// Most-recent first, capped at 25. `full_rbf_only` keeps only
    /// trees with at least one non-signaling predecessor.
    fn recent_replacements(&self, full_rbf_only: bool) -> Result<Vec<ReplacementNode>> {
        let _publication = self.read_publication()?;
        let read = self.read_indexer()?;
        let trees = self.require_mempool()?.recent_rbf_trees(
            full_rbf_only,
            RECENT_REPLACEMENTS_LIMIT,
            &self.tip_blockhash(),
        )?;
        trees
            .into_iter()
            .map(|node| read.enrich_rbf_node(node, None))
            .collect()
    }

    /// Serialize recent replacements with their exact content identity.
    pub fn recent_replacements_json(
        &self,
        full_rbf_only: bool,
    ) -> Result<(Vec<u8>, RepresentationId)> {
        let replacements = self.recent_replacements(full_rbf_only)?;
        Ok(serialize_json(&replacements))
    }
}

impl IndexerRead<'_> {
    /// Layer `mined` and effective fee rate onto an owned RBF tree.
    fn enrich_rbf_node(
        &self,
        node: RbfNode,
        successor_time: Option<Timestamp>,
    ) -> Result<ReplacementNode> {
        let interval = successor_time
            .and_then(|time| time.checked_sub(node.first_seen))
            .map(|duration| *duration);
        let (mined, rate) = self.rbf_status_and_rate(&node)?;
        let first_seen = node.first_seen;
        let replaces = node
            .replaces
            .into_iter()
            .map(|child| self.enrich_rbf_node(child, Some(first_seen)))
            .collect::<Result<_>>()?;
        Ok(ReplacementNode {
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
        })
    }

    /// Resolve confirmation and its effective rate with one exact txid lookup.
    fn rbf_status_and_rate(&self, node: &RbfNode) -> Result<(Option<bool>, FeeRate)> {
        let confirmed_rate = match self.resolve_confirmed_position(&node.txid) {
            Ok((index, _)) => Some(
                self.query()
                    .plugins()
                    .transactions
                    .fees
                    .effective_fee_rate
                    .tx_index
                    .collect_one(index)
                    .data()?,
            ),
            Err(Error::UnknownTxid) => None,
            Err(error) => return Err(error),
        };
        let mined = confirmed_rate.is_some().then_some(true);
        let rate = confirmed_rate.unwrap_or(node.rate);
        Ok((mined, rate))
    }
}
