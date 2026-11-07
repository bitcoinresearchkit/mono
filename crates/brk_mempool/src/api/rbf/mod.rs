//! RBF trees captured from one publication for later confirmed-chain enrichment.

use std::time::{Instant, SystemTime, UNIX_EPOCH};

use brk_error::{Error, Result};
use brk_types::{BlockHash, FeeRate, Sats, Timestamp, Transaction, Txid, VSize};
use rapidhash::{RapidHashMap, RapidHashSet};

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
        let root_txid = Self::tree_root(state, txid, &mut remaining, &mut RapidHashMap::default())
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
        // Every version a walk passed, with its tree's root: each chain is walked once, however
        // many of its versions the listing meets.
        let mut roots: RapidHashMap<Txid, Txid> = RapidHashMap::default();
        let mut total = MAX_RBF_WORK.saturating_mul(limit + 1);
        let mut trees = Vec::new();
        for candidate in state.graveyard.replacements_recent_first() {
            let Some(left) = total.checked_sub(1) else {
                break;
            };
            total = left;
            // From the replaced transaction to its tree's root: a replacer the pool never saw (a
            // confirmed conflict) roots the tree of everything it replaced.
            let Some((replaced, _)) = candidate else {
                continue;
            };
            let mut remaining = MAX_RBF_WORK.min(total);
            let built = Self::tree_root(state, replaced, &mut remaining, &mut roots)
                .filter(|root| seen.insert(*root))
                .map(|root| {
                    Self::build_rbf_node(&root, &state.txs, &state.graveyard, &mut remaining, 0)
                });
            total -= MAX_RBF_WORK.min(total) - remaining;
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
            // A root neither pending nor buried replaced what lists it (a confirmed conflict).
            return if depth == 0 {
                Self::unseen_root(txid, txs, graveyard, remaining)
            } else {
                Ok(None)
            };
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
            unseen: false,
            rbf: entry.rbf,
            replaced_non_signaling,
            full_rbf,
            replaces,
        }))
    }

    /// The root of `txid`'s tree: the terminal replacer, or, when the last version the pool
    /// knows was replaced by a transaction neither pending nor buried (a conflict a block
    /// confirmed unseen), that transaction (mempool.space's mined replacement).
    /// `txid`'s tree's root: its terminal replacer, or the replacer the pool never saw (a
    /// confirmed conflict) that the last buried version names. `roots` remembers every version
    /// walked, with the root it reached: a walk meeting one stops there.
    fn tree_root(
        state: &Pool,
        txid: &Txid,
        remaining: &mut usize,
        roots: &mut RapidHashMap<Txid, Txid>,
    ) -> Option<Txid> {
        let live = |txid: &Txid| state.txs.contains(txid);
        let mut path = Vec::new();
        let mut at = *txid;
        let root = loop {
            if let Some(&root) = roots.get(&at) {
                break root;
            }
            *remaining = remaining.checked_sub(1)?;
            path.push(at);
            match state.graveyard.next_version(&at, live) {
                Some(by) => at = by,
                None => {
                    break match state.graveyard.get(&at).and_then(|tomb| tomb.replaced_by()) {
                        Some(by) if !live(by) && state.graveyard.get(by).is_none() => *by,
                        _ => at,
                    };
                }
            }
        };
        roots.extend(path.into_iter().map(|version| (version, root)));
        Some(root)
    }

    /// An unseen confirmed replacer: what it replaced, its own facts left for the chain.
    fn unseen_root(
        txid: &Txid,
        txs: &ReadOnlyTxStore,
        graveyard: &TxGraveyard,
        remaining: &mut usize,
    ) -> std::result::Result<Option<RbfNode>, Exceeded> {
        let mut replaces = Vec::new();
        // When the replacement was seen (mempool.space's tree time): the latest of what it
        // replaced, told it by its block.
        let mut seen: Option<Instant> = None;
        for (predecessor, tomb) in graveyard.predecessors_of(txid) {
            seen = seen.max(Some(tomb.removed_at));
            if let Some(node) = Self::build_rbf_node(predecessor, txs, graveyard, remaining, 1)? {
                replaces.push(node);
            }
        }
        let Some(seen) = seen.filter(|_| !replaces.is_empty()) else {
            return Ok(None);
        };
        let seen = SystemTime::now()
            .checked_sub(seen.elapsed())
            .and_then(|at| at.duration_since(UNIX_EPOCH).ok())
            .map_or_else(Timestamp::now, |since| {
                Timestamp::from(since.as_secs() as usize)
            });
        let replaced_non_signaling = replaces.iter().any(|c| !c.rbf);
        Ok(Some(RbfNode {
            txid: *txid,
            fee: Sats::ZERO,
            vsize: VSize::default(),
            value: Sats::ZERO,
            first_seen: seen,
            rate: FeeRate::default(),
            in_mempool: false,
            unseen: true,
            rbf: false,
            replaced_non_signaling,
            full_rbf: replaced_non_signaling || replaces.iter().any(|c| c.full_rbf),
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
