use bitview_plugin_indexer::SafeLengths;
use bitview_types::{RbfResponse, RbfTx, ReplacementNode};
use brk_mempool::{RbfForTx, RbfNode};
use brk_types::{CheckedSub, FeeRate, Timestamp, Txid, VSize};
use vecdb::ReadableVec;

use super::serialize_json;
use crate::{
    Error, OptionData, Query, RepresentationId, Result, r#impl::tx::confirmed::IndexerRead,
};

/// An exact owned RBF tree resolved before an async response handoff.
pub struct ResolvedRbf {
    source: RbfForTx,
    pin: SafeLengths,
}

impl Query {
    /// Resolve the exact owned replacement tree from one mempool publication.
    pub fn resolve_rbf(&self, txid: &Txid) -> Result<ResolvedRbf> {
        let (mempool, pin) = self.mempool_then_pin()?;
        let (mempool, tip) = Self::lookup_mempool(mempool).ok_or(Error::MempoolNotAvailable)?;
        Ok(ResolvedRbf {
            source: mempool.rbf_for_tx(txid, &tip)?,
            pin: pin.into_inner(),
        })
    }

    /// Enrich an already resolved tree without repeating its mempool lookup.
    fn tx_rbf_resolved(&self, rbf: ResolvedRbf) -> Result<RbfResponse> {
        if rbf.source.is_empty() {
            return Ok(RbfResponse::EMPTY);
        }
        // Enrich at the publication the tree was resolved against.
        let read = self.read_indexer_at(self.read_pin(rbf.pin));
        let RbfForTx { root, replaces } = rbf.source;
        let replacements = root
            .map(|node| read.enrich_rbf_tree(node, false))
            .transpose()?
            .flatten();
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
        let (mempool, pin) = self.mempool_then_pin()?;
        let read = self.read_indexer_at(pin);
        let (mempool, tip) = Self::lookup_mempool(mempool).ok_or(Error::MempoolNotAvailable)?;
        let trees = mempool.recent_rbf_trees(full_rbf_only, &tip)?;
        Ok(trees
            .into_iter()
            .map(|node| read.enrich_rbf_tree(node, full_rbf_only))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
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
}

impl IndexerRead<'_> {
    /// A tree from its root: each node enriched, and the root marked `mined` when any of its
    /// transactions is. None when an unseen root resolves to no tree (`with_chain_facts`), or to
    /// one `full_rbf_only` excludes.
    fn enrich_rbf_tree(&self, root: RbfNode, full_rbf_only: bool) -> Result<Option<ReplacementNode>> {
        fn any_mined(node: &ReplacementNode) -> bool {
            node.tx.mined.is_some() || node.replaces.iter().any(any_mined)
        }
        let Some(root) = self
            .with_chain_facts(root)?
            .filter(|root| !full_rbf_only || root.full_rbf)
        else {
            return Ok(None);
        };
        let mut root = self.enrich_rbf_node(root, None)?;
        root.mined = any_mined(&root).then_some(true);
        Ok(Some(root))
    }

    /// An unseen root (a conflict confirmed without the pool seeing it) with its facts from the
    /// chain (its time stays when the replacement was seen, as mempool.space's). One the chain
    /// doesn't hold either (a reorg took it) is never served bare: the one version it replaced
    /// roots the tree when that one has a replacement history of its own, else there's none.
    fn with_chain_facts(&self, mut root: RbfNode) -> Result<Option<RbfNode>> {
        if !root.unseen {
            return Ok(Some(root));
        }
        let index = match self.resolve_confirmed_position(&root.txid) {
            Ok((index, _)) => index,
            Err(Error::UnknownTxid) => {
                return Ok(match root.replaces.len() {
                    1 => root.replaces.pop().filter(|only| !only.replaces.is_empty()),
                    _ => None,
                });
            }
            Err(error) => return Err(error),
        };
        let tx = self.query().transaction_by_index(index, self.pin())?;
        root.fee = tx.fee;
        root.vsize = tx.total_sigop_cost.adjust_vsize(VSize::from(tx.weight));
        root.value = tx.output.iter().map(|output| output.value).sum();
        root.rbf = tx.input.iter().any(|input| input.sequence < 0xffff_fffe);
        Ok(Some(root))
    }

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
        let replaces: Vec<ReplacementNode> = node
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
                full_rbf: (!replaces.is_empty()).then_some(node.replaced_non_signaling),
                mined,
            },
            time: first_seen,
            full_rbf: node.full_rbf,
            interval,
            mined: None,
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
                    .tx
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
