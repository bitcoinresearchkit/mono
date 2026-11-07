//! RBF trees captured from one publication for later confirmed-chain enrichment.

use brk_error::Result;
use brk_types::{BlockHash, FeeRate, Sats, Transaction, Txid, VSize};
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

/// The transactions a tree holds at most: past it, it is served cut (`truncated`).
const MAX_RBF_WORK: usize = 4096;
/// The trees a recent replacements listing holds: mempool.space's count.
const RECENT_TREES: usize = 25;
/// The versions a tree holds at most, newest first: past it, it is served cut (`truncated`). It
/// bounds enrichment, JSON nesting and recursive destruction too: two levels a version plus the
/// response's own, 127 at 63 versions, within serde_json's default limit (128) every Rust client
/// parses with. mempool.space serves trees whole, however deep.
const MAX_RBF_DEPTH: usize = 63;

impl ReadOnlyState {
    /// Walk forward through `Replaced { by }` to the terminal replacer
    /// and return its predecessor tree (none without a replacement),
    /// plus the requested tx's direct predecessors from the acquired publication.
    /// A history past 4,096 transactions or 63 versions comes cut, its root marked `truncated`:
    /// the newest versions, as many as fit (one past the walk's reach roots it where the walk
    /// stopped), and always the one asked for (one more than 62 versions below its root roots the
    /// tree 62 above it instead). The walk to the root, the direct predecessors and the tree each
    /// have 4,096 steps.
    pub fn rbf_for_tx(&self, txid: &Txid, tip: &BlockHash) -> Result<RbfForTx> {
        // Trees read entries and outputs, never prevouts: no input needs resolving.
        let state = self.pool()?.at(tip)?;
        let mut walk = MAX_RBF_WORK;
        let (mut root_txid, mut truncated) =
            match Self::tree_root(state, txid, &mut walk, &mut RapidHashMap::default()) {
                Ok(root) => (root, false),
                Err(reached) => (reached, true),
            };
        // The version as many levels above it as a tree goes deep: past it, the root is further
        // (or an unseen replacer above the last known version, one level more).
        let live = |txid: &Txid| state.txs.contains(txid);
        let mut above = vec![*txid];
        while let Some(&window) = above.last()
            && window != root_txid
            && above.len() < MAX_RBF_DEPTH
            && let Some(by) = state.graveyard.next_version(&window, live)
        {
            above.push(by);
        }
        let window = *above.last().expect("it");
        if window != root_txid && above.len() == MAX_RBF_DEPTH {
            (root_txid, truncated) = (window, true);
        }
        // The versions from the root down to it, built first: the budget always reaches it. An
        // unseen replacer roots the tree one level above the last known version.
        let mut path = Vec::with_capacity(above.len() + 1);
        if window != root_txid {
            path.push(root_txid);
        }
        path.extend(above.into_iter().rev());
        let mut replaces = Vec::new();
        for (predecessor, _) in state.graveyard.predecessors_of(txid) {
            if replaces.len() == MAX_RBF_WORK {
                truncated = true;
                break;
            }
            replaces.push(*predecessor);
        }
        let mut remaining = MAX_RBF_WORK;
        // A transaction that neither replaced nor was replaced has no tree (mempool.space: null).
        let mut root = Self::build_rbf_node(
            &root_txid,
            &state.txs,
            &state.graveyard,
            &mut remaining,
            &mut truncated,
            0,
            &path,
        )
        .filter(|root| !root.replaces.is_empty());
        if let Some(root) = root.as_mut() {
            root.truncated = truncated;
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

    /// Each tree has `rbf_for_tx`'s budget: one larger or nested deeper comes cut (`truncated`),
    /// as there. The whole walk is bounded too (`RECENT_TREES + 1` trees' budgets, charged for
    /// discarded candidates and stale graveyard entries): once that runs out the listing ends
    /// with the trees found so far.
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
            // The walk to the root and the tree each take their own steps from the total.
            let budget = MAX_RBF_WORK.min(total);
            let mut walk = budget;
            let root = Self::tree_root(state, replaced, &mut walk, &mut roots);
            total -= budget - walk;
            let Some(root) = root.ok().filter(|root| seen.insert(*root)) else {
                continue;
            };
            let budget = MAX_RBF_WORK.min(total);
            let mut remaining = budget;
            let mut truncated = false;
            let built = Self::build_rbf_node(
                &root,
                &state.txs,
                &state.graveyard,
                &mut remaining,
                &mut truncated,
                0,
                &[],
            );
            total -= budget - remaining;
            if let Some(mut node) = built
                && !node.replaces.is_empty()
                && (!full_rbf_only || node.full_rbf)
            {
                node.truncated = truncated;
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

    /// `txid`'s node and what it replaced, depth first, while `remaining` and [`MAX_RBF_DEPTH`]
    /// allow: what doesn't fit is left out, `truncated` told so, but never a version of `path`
    /// (from the root, at its depth: built first). The flags (`full_rbf`) then describe what the
    /// tree holds.
    fn build_rbf_node(
        txid: &Txid,
        txs: &ReadOnlyTxStore,
        graveyard: &TxGraveyard,
        remaining: &mut usize,
        truncated: &mut bool,
        depth: usize,
        path: &[Txid],
    ) -> Option<RbfNode> {
        if depth >= MAX_RBF_DEPTH || *remaining == 0 {
            *truncated = true;
            return None;
        }
        *remaining -= 1;
        let Some((tx, entry, rate, in_mempool)) = Self::resolve_rbf_node(txid, txs, graveyard)
        else {
            // A root neither pending nor buried replaced what lists it (a confirmed conflict).
            return if depth == 0 {
                Self::unseen_root(txid, txs, graveyard, remaining, truncated, path)
            } else {
                None
            };
        };

        let replaces =
            Self::predecessor_nodes(txid, txs, graveyard, remaining, truncated, depth + 1, path);

        let replaced_non_signaling = replaces.iter().any(|c| !c.rbf);
        // mempool.space's tree flag: a replaced leaf's own non-signaling, a replacer's subtree's.
        let full_rbf = if replaces.is_empty() {
            !entry.rbf
        } else {
            replaced_non_signaling || replaces.iter().any(|c| c.full_rbf)
        };
        let value: Sats = tx.output.iter().map(|o| o.value).sum();

        Some(RbfNode {
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
            truncated: false,
            replaces,
        })
    }

    /// `txid`'s tree's root: the newest version the pool knows (following the graveyard's
    /// `next_version`), or the replacer that version names when the pool never saw
    /// it (a conflict a block confirmed: mempool.space's mined replacement). Past `remaining`,
    /// the newest version the walk reached (`Err`). `roots` remembers every version walked, with
    /// the root it reached: a walk meeting one stops there.
    fn tree_root(
        state: &Pool,
        txid: &Txid,
        remaining: &mut usize,
        roots: &mut RapidHashMap<Txid, Txid>,
    ) -> std::result::Result<Txid, Txid> {
        let live = |txid: &Txid| state.txs.contains(txid);
        let mut path = Vec::new();
        let mut at = *txid;
        let root = loop {
            if let Some(&root) = roots.get(&at) {
                break root;
            }
            let Some(left) = remaining.checked_sub(1) else {
                return Err(at);
            };
            *remaining = left;
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
        Ok(root)
    }

    /// What `txid` replaced, as nodes at `depth`, in the graveyard's order while the bounds allow
    /// (past them the rest is left out, not visited): the one on `path` built first, in its place
    /// if the others reach it, last if not.
    fn predecessor_nodes(
        txid: &Txid,
        txs: &ReadOnlyTxStore,
        graveyard: &TxGraveyard,
        remaining: &mut usize,
        truncated: &mut bool,
        depth: usize,
        path: &[Txid],
    ) -> Vec<RbfNode> {
        let next = path.get(depth).filter(|_| path.get(depth - 1) == Some(txid));
        let mut first = next
            .filter(|_| depth < MAX_RBF_DEPTH && *remaining > 0)
            .and_then(|next| {
                Self::build_rbf_node(next, txs, graveyard, remaining, truncated, depth, path)
            });
        let mut replaces = Vec::new();
        for (predecessor, _) in graveyard.predecessors_of(txid) {
            if Some(predecessor) == next {
                replaces.extend(first.take());
                continue;
            }
            if depth >= MAX_RBF_DEPTH || *remaining == 0 {
                *truncated = true;
                break;
            }
            if let Some(node) =
                Self::build_rbf_node(predecessor, txs, graveyard, remaining, truncated, depth, path)
            {
                replaces.push(node);
            }
        }
        replaces.extend(first);
        replaces
    }

    /// An unseen confirmed replacer: what it replaced, its own facts left for the chain.
    fn unseen_root(
        txid: &Txid,
        txs: &ReadOnlyTxStore,
        graveyard: &TxGraveyard,
        remaining: &mut usize,
        truncated: &mut bool,
        path: &[Txid],
    ) -> Option<RbfNode> {
        // When the replacement was seen (mempool.space's tree time): the latest of what it
        // replaced, told it by its block (of as many as a tree holds).
        let seen = graveyard
            .predecessors_of(txid)
            .take(MAX_RBF_WORK)
            .map(|(_, tomb)| tomb.removed_at)
            .max()?
            .timestamp();
        let replaces = Self::predecessor_nodes(txid, txs, graveyard, remaining, truncated, 1, path);
        if replaces.is_empty() {
            return None;
        }
        let replaced_non_signaling = replaces.iter().any(|c| !c.rbf);
        Some(RbfNode {
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
            truncated: false,
            replaces,
        })
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
