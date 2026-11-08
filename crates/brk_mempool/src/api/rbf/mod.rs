//! RBF trees captured from one publication for later confirmed-chain enrichment.

use brk_error::Result;
use brk_types::{BlockHash, FeeRate, Sats, Transaction, Txid, VSize};
use rapidhash::{RapidHashMap, RapidHashSet};
use serde::Serialize;

use crate::{
    JsonBody, ReadOnlyState, json,
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
/// The trees walked for it: spares for those the chain can't serve yet (an unseen replacer it
/// doesn't hold), so the listing still holds 25.
const RECENT_WALKED: usize = 2 * RECENT_TREES;
/// The transactions a listed tree holds at most, its share of one tree's budget: the whole
/// listing holds about what one tree may. Past it, a tree is listed cut (`truncated`), and
/// served whole (to [`MAX_RBF_WORK`]) for each of its transactions.
const LISTED_TREE_WORK: usize = MAX_RBF_WORK / RECENT_TREES;
/// The versions a tree holds at most, newest first: past it, it is served cut (`truncated`). It
/// bounds enrichment, JSON nesting and recursive destruction too: two levels a version plus the
/// response's own, 127 at 63 versions, within serde_json's default limit (128) every Rust client
/// parses with. mempool.space serves trees whole, however deep.
const MAX_RBF_DEPTH: usize = 63;

impl ReadOnlyState {
    /// The replacement tree holding `txid`, and `txid`'s direct predecessors. The tree is `None`
    /// for a transaction that neither replaced nor was replaced.
    ///
    /// The walk to the root, the predecessor list and the tree each take at most 4,096 steps. A
    /// tree past 4,096 transactions or 63 versions is served cut, its root marked `truncated`. It
    /// keeps the newest versions that fit and always the requested one. If the walk to the root
    /// runs out of steps, the tree is rooted where it stopped. If `txid` lies more than 62 versions
    /// below the root, the tree is rooted 62 versions above `txid`.
    pub fn rbf_for_tx(&self, txid: &Txid, tip: &BlockHash) -> Result<RbfForTx> {
        // Trees read entries and outputs, never prevouts: no input needs resolving.
        let state = self.pool()?.at(tip)?;
        let mut walk = MAX_RBF_WORK;
        let (mut root_txid, mut truncated) =
            match Self::tree_root(state, txid, &mut walk, &mut RapidHashMap::default()) {
                Ok(root) => (root, false),
                Err(reached) => (reached, true),
            };
        let path = Self::path_to(state, txid, root_txid);
        if path[0] != root_txid {
            (root_txid, truncated) = (path[0], true);
        }
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

    /// The versions from the served root down to `txid`, root first. The tree build follows these
    /// first, so its budget always reaches `txid`. When `txid` lies deeper than [`MAX_RBF_DEPTH`],
    /// the path starts that many levels above it. An unseen replacer sits one level above the last
    /// version the pool knows.
    fn path_to(state: &Pool, txid: &Txid, root: Txid) -> Vec<Txid> {
        let live = |txid: &Txid| state.txs.contains(txid);
        let mut above = vec![*txid];
        while let Some(&version) = above.last()
            && version != root
            && above.len() < MAX_RBF_DEPTH
            && let Some(by) = state.graveyard.next_version(&version, live)
        {
            above.push(by);
        }
        let top = *above.last().expect("txid itself");
        // Short of the root without running out of depth: the root is the unseen one above.
        let unseen = top != root && above.len() < MAX_RBF_DEPTH;
        let mut path = Vec::with_capacity(above.len() + 1);
        if unseen {
            path.push(root);
        }
        path.extend(above.into_iter().rev());
        path
    }

    /// Recent replacement trees, newest first, one per root, 50 at most.
    /// [`Self::recent_replacements_json`] serves 25 of them, as mempool.space's `/replacements`
    /// does. `full_rbf_only` drops trees with no non-signaling predecessor. The first request of
    /// each publication walks them.
    pub fn recent_rbf_trees(&self, full_rbf_only: bool, tip: &BlockHash) -> Result<Vec<RbfNode>> {
        let state = self.pool()?.at(tip)?;
        Ok(Self::recent_trees(state, full_rbf_only).to_vec())
    }

    fn recent_trees(state: &Pool, full_rbf_only: bool) -> &[RbfNode] {
        state.recent_trees[usize::from(full_rbf_only)]
            .get_or_init(|| Self::walk_recent_trees(state, full_rbf_only))
    }

    /// The recent listing as served: the first 25 recent trees that `enrich` can serve. `enrich`
    /// adds the chain's facts, and a tree it can't serve yet gives up its place. The body is
    /// encoded once per publication and `reader` tip. Readers arriving during the build wait for
    /// it and share it.
    pub fn recent_replacements_json<T: Serialize, E: From<brk_error::Error>>(
        &self,
        full_rbf_only: bool,
        tip: &BlockHash,
        reader: BlockHash,
        mut enrich: impl FnMut(RbfNode) -> std::result::Result<Option<T>, E>,
    ) -> std::result::Result<JsonBody, E> {
        let state = self.pool()?.at(tip)?;
        let mut built = state.replacements_json[usize::from(full_rbf_only)]
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some((at, body)) = built.as_ref()
            && *at == reader
        {
            return Ok(body.clone());
        }
        let mut served = Vec::with_capacity(RECENT_TREES);
        for tree in Self::recent_trees(state, full_rbf_only) {
            if served.len() == RECENT_TREES {
                break;
            }
            served.extend(enrich(tree.clone())?);
        }
        let body = json::body(&served);
        *built = Some((reader, body.clone()));
        Ok(body)
    }

    /// Each tree has [`LISTED_TREE_WORK`]: one larger or nested deeper comes cut (`truncated`).
    /// The whole walk is bounded too (`RECENT_WALKED + 1` trees' budgets, charged for discarded
    /// candidates and stale graveyard entries): once that runs out the listing ends with the trees
    /// found so far.
    fn walk_recent_trees(state: &Pool, full_rbf_only: bool) -> Vec<RbfNode> {
        let limit = RECENT_WALKED;
        let mut seen: RapidHashSet<Txid> = RapidHashSet::default();
        // Every version a walk passed, with its tree's root: each chain is walked once, however
        // many of its versions the listing meets.
        let mut roots: RapidHashMap<Txid, Txid> = RapidHashMap::default();
        let mut total = LISTED_TREE_WORK.saturating_mul(limit + 1);
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
            let budget = LISTED_TREE_WORK.min(total);
            let mut walk = budget;
            let root = Self::tree_root(state, replaced, &mut walk, &mut roots);
            total -= budget - walk;
            // One whose walk runs out lies deep in a chain whose newer replacements came first
            // (newest first), its tree with them.
            let Some(root) = root.ok().filter(|root| seen.insert(*root)) else {
                continue;
            };
            let budget = LISTED_TREE_WORK.min(total);
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

    /// `txid`'s node and what it replaced, depth first, within `remaining` and [`MAX_RBF_DEPTH`].
    /// What doesn't fit is left out and sets `truncated`. Versions on `path` are built first, so
    /// they are always kept. The flags (`full_rbf`) describe what the served tree holds.
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

    /// What `txid` replaced, as nodes at `depth`, in the graveyard's order within the bounds. Past
    /// them the rest is left out unvisited. The predecessor on `path` is built first and placed in
    /// order when the others reach it, last otherwise.
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
        // When the replacement was seen (mempool.space's tree time): the latest burial among what
        // it replaced, which is the last one listed.
        let seen = graveyard
            .predecessors_of(txid)
            .next_back()?
            .1
            .removed_at
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
