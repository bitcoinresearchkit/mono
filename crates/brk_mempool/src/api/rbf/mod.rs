//! RBF trees captured from one publication for later confirmed-chain enrichment.

use brk_error::{Error, Result};
use brk_types::{BlockHash, FeeRate, Sats, Transaction, Txid};
use rapidhash::RapidHashSet;

use crate::{
    ReadOnlyState,
    snapshot::Snapshot,
    state::{Pool, TxEntry},
    stores::{ReadOnlyTxStore, TxGraveyard},
};

pub mod for_tx;
pub mod node;

pub use for_tx::RbfForTx;
pub use node::RbfNode;

const MAX_RBF_WORK: usize = 4096;
/// The trees a recent replacements listing holds: mempool.space's count.
const RECENT_TREES: usize = 25;
// Also bounds later enrichment, JSON nesting and recursive destruction: two levels a version plus
// the response's own, 127 at 63 versions, within serde_json's default limit (128) every Rust
// client parses with.
const MAX_RBF_DEPTH: usize = 63;

/// A history walk that ran past one of its bounds.
enum Exceeded {
    Work,
    Depth,
}

/// A history past the bounds is a request over a limit, not a failure: a 400 `limit_exceeded`.
impl From<Exceeded> for Error {
    fn from(exceeded: Exceeded) -> Self {
        Error::LimitExceeded(match exceeded {
            Exceeded::Work => "RBF history exceeds its size limit (4,096 transactions)",
            Exceeded::Depth => "RBF history exceeds its depth limit (63 versions)",
        })
    }
}

impl ReadOnlyState {
    /// Walk forward through `Replaced { by }` to the terminal replacer
    /// and return its full predecessor tree (none without a replacement),
    /// plus the requested tx's direct predecessors from the acquired publication.
    /// Rejects (400 `limit_exceeded`) histories exceeding 4096 traversal steps or 63 nested nodes;
    /// never returns a silently truncated tree.
    pub fn rbf_for_tx(&self, txid: &Txid, tip: &BlockHash) -> Result<RbfForTx> {
        // Trees read entries and outputs, never prevouts: no input needs resolving.
        let state = self.pool()?.at(tip)?;
        let mut remaining = MAX_RBF_WORK;
        let root_txid = state
            .graveyard
            .replacement_root_of(*txid, &mut remaining, |txid| state.txs.contains(txid))
            .ok_or(Exceeded::Work)?;
        let replaces: Vec<_> = state
            .graveyard
            .predecessors_of(txid)
            .take(remaining + 1)
            .map(|(predecessor, _)| *predecessor)
            .collect();
        remaining = remaining
            .checked_sub(replaces.len())
            .ok_or(Exceeded::Work)?;
        // A transaction that neither replaced nor was replaced has no tree (mempool.space: null).
        let mut root =
            Self::build_rbf_node(&root_txid, &state.txs, &state.graveyard, &mut remaining, 0)?
                .filter(|root| !root.replaces.is_empty());
        if let Some(root) = root.as_mut() {
            Self::apply_snapshot_rates(root, &state.graph);
        }
        Ok(RbfForTx { root, replaces })
    }

    /// Recent terminal-replacer trees, most-recent first, deduplicated by root, 25 at most
    /// (mempool.space's `/replacements`). `full_rbf_only` drops trees with no non-signaling
    /// predecessor. Walked once per publication, by the first request.
    pub fn recent_rbf_trees(&self, full_rbf_only: bool, tip: &BlockHash) -> Result<Vec<RbfNode>> {
        let state = self.pool()?.at(tip)?;
        Ok(state.recent_trees[usize::from(full_rbf_only)]
            .get_or_init(|| Self::walk_recent_trees(state, full_rbf_only))
            .clone())
    }

    /// Each tree has `rbf_for_tx`'s budget: one larger or nested deeper is skipped, never
    /// truncated, and the listing goes on. The whole walk is bounded too (`RECENT_TREES + 1`
    /// trees' budgets, charged for discarded candidates and stale graveyard entries): once that
    /// runs out the listing ends with the complete trees found so far.
    fn walk_recent_trees(state: &Pool, full_rbf_only: bool) -> Vec<RbfNode> {
        let limit = RECENT_TREES;
        let mut seen: RapidHashSet<Txid> = RapidHashSet::default();
        let mut total = MAX_RBF_WORK.saturating_mul(limit + 1);
        let mut trees = Vec::new();
        for candidate in state.graveyard.replacements_recent_first() {
            let Some(left) = total.checked_sub(1) else {
                break;
            };
            total = left;
            // From the replaced transaction: a replacer the pool never saw (a confirmed conflict)
            // leaves the walk rooted at the last member it knows.
            let Some((replaced, _)) = candidate else {
                continue;
            };
            let mut remaining = MAX_RBF_WORK.min(total);
            let built = state
                .graveyard
                .replacement_root_of(*replaced, &mut remaining, |txid| state.txs.contains(txid))
                .filter(|root| seen.insert(*root))
                .map(|root| {
                    Self::build_rbf_node(&root, &state.txs, &state.graveyard, &mut remaining, 0)
                });
            total -= MAX_RBF_WORK.min(total) - remaining;
            // A root that replaced nothing it knows (its replacer confirmed unseen) is no tree, as in
            // `rbf_for_tx`.
            if let Some(Ok(Some(node))) = built
                && !node.replaces.is_empty()
                && (!full_rbf_only || node.full_rbf)
            {
                trees.push(node);
                if trees.len() == limit {
                    break;
                }
            }
        }
        // Resolve rates only for returned trees, after filtering and admission.
        for root in &mut trees {
            Self::apply_snapshot_rates(root, &state.graph);
        }
        trees
    }

    fn build_rbf_node(
        txid: &Txid,
        txs: &ReadOnlyTxStore,
        graveyard: &TxGraveyard,
        remaining: &mut usize,
        depth: usize,
    ) -> std::result::Result<Option<RbfNode>, Exceeded> {
        if depth >= MAX_RBF_DEPTH {
            return Err(Exceeded::Depth);
        }
        *remaining = remaining.checked_sub(1).ok_or(Exceeded::Work)?;
        let Some((tx, entry, rate, in_mempool)) = Self::resolve_rbf_node(txid, txs, graveyard)
        else {
            return Ok(None);
        };

        let mut replaces = Vec::new();
        for (predecessor, _) in graveyard.predecessors_of(txid) {
            if let Some(node) =
                Self::build_rbf_node(predecessor, txs, graveyard, remaining, depth + 1)?
            {
                replaces.push(node);
            }
        }

        let replaced_non_signaling = replaces.iter().any(|c| !c.rbf);
        // mempool.space's tree flag: a replaced leaf's own non-signaling, a replacer's subtree's.
        let full_rbf = if replaces.is_empty() {
            !entry.rbf
        } else {
            replaced_non_signaling || replaces.iter().any(|c| c.full_rbf)
        };
        let value: Sats = tx.output.iter().map(|o| o.value).sum();

        Ok(Some(RbfNode {
            txid: *txid,
            fee: entry.fee,
            vsize: entry.vsize,
            value,
            first_seen: entry.first_seen,
            rate,
            in_mempool,
            rbf: entry.rbf,
            replaced_non_signaling,
            full_rbf,
            replaces,
        }))
    }

    fn resolve_rbf_node<'a>(
        txid: &Txid,
        txs: &'a ReadOnlyTxStore,
        graveyard: &'a TxGraveyard,
    ) -> Option<(&'a Transaction, &'a TxEntry, FeeRate, bool)> {
        if let Some(record) = txs.record(txid) {
            return Some((&record.tx, &record.entry, record.own_rate(), true));
        }
        graveyard.get(txid).map(|tombstone| {
            (
                tombstone.record.tx.as_ref(),
                &tombstone.record.entry,
                tombstone.chunk_rate,
                false,
            )
        })
    }

    fn apply_snapshot_rates(node: &mut RbfNode, snapshot: &Snapshot) {
        if node.in_mempool
            && let Some(rate) = snapshot.chunk_rate_for(&node.txid)
        {
            node.rate = rate;
        }
        for predecessor in &mut node.replaces {
            Self::apply_snapshot_rates(predecessor, snapshot);
        }
    }
}
