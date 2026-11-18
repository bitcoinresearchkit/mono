//! Replacement trees derived from one publication's authoritative removal links.
use crate::{
    JsonBody, ReadOnlyState, json,
    state::{Pool, signals_rbf},
    stores::tx_store::TxRecord,
};
use brk_error::Result;
use brk_types::{BlockHash, FeeRate, Sats, Txid, VSize};
use rapidhash::{RapidHashMap, RapidHashSet};
use serde::Serialize;

pub mod for_tx;
pub mod node;
pub use for_tx::RbfForTx;
pub use node::RbfNode;

/// The transactions a tree holds at most: past it, it is served cut (`truncated`).
const MAX_RBF_WORK: usize = 4096;
/// The versions a tree holds at most, newest first: past it, it is served cut (`truncated`). It
/// bounds JSON nesting too: two levels a version plus the response's own, 127 at 63 versions,
/// within serde_json's default limit (128) every Rust client parses with.
const MAX_RBF_DEPTH: usize = 63;
/// The trees a recent replacements listing holds: mempool.space's count.
const RECENT_TREES: usize = 25;
/// The trees walked for it: spares for those the chain can't serve yet (an unseen replacer it
/// doesn't hold), so the listing still holds 25.
const RECENT_WALKED: usize = 2 * RECENT_TREES;
/// The transactions a listed tree holds at most, its share of one tree's budget: the whole
/// listing holds about what one tree may.
const LISTED_TREE_WORK: usize = MAX_RBF_WORK / RECENT_TREES;

impl ReadOnlyState {
    /// The newest replacement and its predecessors. Bounded to 4,096 nodes and 63 levels;
    /// the requested transaction's path is built first so truncation always keeps it.
    pub fn rbf_for_tx(&self, txid: &Txid, tip: &BlockHash) -> Result<RbfForTx> {
        let state = self.pool()?.at(tip)?;
        let mut budget = MAX_RBF_WORK;
        let (root, mut truncated) =
            match tree_root(state, *txid, &mut budget, &mut RapidHashMap::default()) {
                Ok(root) => (root, false),
                Err(root) => (root, true),
            };
        let mut path = vec![*txid];
        while path.len() < MAX_RBF_DEPTH && path.last() != Some(&root) {
            let Some(by) = state
                .graveyard
                .get(path.last().expect("requested tx"))
                .and_then(|tomb| tomb.direct_replacer())
                .copied()
            else {
                break;
            };
            path.push(by);
        }
        path.reverse();
        truncated |= path[0] != root;
        let replaces = state
            .graveyard
            .predecessors_of(txid)
            .take(MAX_RBF_WORK + 1)
            .map(|(txid, _)| *txid)
            .collect::<Vec<_>>();
        truncated |= replaces.len() > MAX_RBF_WORK;
        let mut walk = TreeWalk::new(state, MAX_RBF_WORK, &path);
        let mut root = walk
            .build(path[0], 0)
            .filter(|node| !node.replaces.is_empty());
        if let Some(root) = &mut root {
            fill_pending_root(state, root);
            root.truncated = truncated || walk.truncated;
        }
        Ok(RbfForTx {
            root,
            replaces: replaces.into_iter().take(MAX_RBF_WORK).collect(),
        })
    }

    /// The first 25 recent trees the chain can enrich, encoded once per publication and reader
    /// tip. Extra candidates let an unseen root awaiting indexing give up its listing place.
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
        let trees = state.recent_trees[usize::from(full_rbf_only)]
            .get_or_init(|| recent_trees(state, full_rbf_only));
        let mut served = Vec::with_capacity(RECENT_TREES);
        for tree in trees {
            if served.len() == RECENT_TREES {
                break;
            }
            let mut tree = tree.clone();
            fill_pending_root(state, &mut tree);
            served.extend(enrich(tree)?);
        }
        let body = json::body(&served);
        *built = Some((reader, body.clone()));
        Ok(body)
    }
}

/// Memoize roots so a listing walks a replacement chain only once. Missing terminal replacers
/// remain roots: their facts come from the chain during enrichment.
fn tree_root(
    state: &Pool,
    txid: Txid,
    remaining: &mut usize,
    roots: &mut RapidHashMap<Txid, Txid>,
) -> std::result::Result<Txid, Txid> {
    let mut path = Vec::new();
    let mut at = txid;
    let root = loop {
        if let Some(&root) = roots.get(&at) {
            break root;
        }
        let Some(left) = remaining.checked_sub(1) else {
            return Err(at);
        };
        *remaining = left;
        path.push(at);
        match state
            .graveyard
            .get(&at)
            .and_then(|tomb| tomb.direct_replacer())
        {
            Some(by) => at = *by,
            None => break at,
        }
    };
    roots.extend(path.into_iter().map(|txid| (txid, root)));
    Ok(root)
}

fn recent_trees(state: &Pool, full_rbf_only: bool) -> Vec<RbfNode> {
    let mut seen = RapidHashSet::default();
    let mut roots = RapidHashMap::default();
    let mut remaining = MAX_RBF_WORK * (RECENT_WALKED + 1);
    let mut trees = Vec::new();
    for candidate in state.graveyard.replacements_recent_first() {
        let Some(left) = remaining.checked_sub(1) else {
            break;
        };
        remaining = left;
        let Some((replaced, _)) = candidate else {
            continue;
        };
        let budget = LISTED_TREE_WORK.min(remaining);
        let mut walk_budget = budget;
        let root = tree_root(state, *replaced, &mut walk_budget, &mut roots);
        remaining -= budget - walk_budget;
        let Ok(root) = root else {
            continue;
        };
        if !seen.insert(root) {
            continue;
        }
        let budget = LISTED_TREE_WORK.min(remaining);
        let mut walk = TreeWalk::new(state, budget, &[]);
        let node = walk.build(root, 0);
        remaining -= budget - walk.remaining;
        let Some(mut node) = node.filter(|node| !node.replaces.is_empty()) else {
            continue;
        };
        node.truncated = walk.truncated;
        if node.truncated && !node.full_rbf {
            let budget = MAX_RBF_WORK.min(remaining);
            let mut left = budget;
            node.full_rbf = any_non_signaling(state, root, &mut left);
            remaining -= budget - left;
        }
        if full_rbf_only && !node.full_rbf {
            continue;
        }
        trees.push(node);
        if trees.len() == RECENT_WALKED {
            break;
        }
    }
    trees
}

fn any_non_signaling(state: &Pool, root: Txid, remaining: &mut usize) -> bool {
    let mut stack = vec![root];
    while let Some(txid) = stack.pop() {
        for (txid, tombstone) in state.graveyard.predecessors_of(&txid) {
            let Some(left) = remaining.checked_sub(1) else {
                return false;
            };
            *remaining = left;
            if !signals_rbf(&tombstone.record.tx) {
                return true;
            }
            stack.push(*txid);
        }
    }
    false
}

struct TreeWalk<'a> {
    state: &'a Pool,
    remaining: usize,
    truncated: bool,
    path: &'a [Txid],
}

impl<'a> TreeWalk<'a> {
    fn new(state: &'a Pool, remaining: usize, path: &'a [Txid]) -> Self {
        Self {
            state,
            remaining,
            path,
            truncated: false,
        }
    }

    fn build(&mut self, txid: Txid, depth: usize) -> Option<RbfNode> {
        if self.remaining == 0 || depth == MAX_RBF_DEPTH {
            self.truncated = true;
            return None;
        }
        self.remaining -= 1;
        let graveyard = &self.state.graveyard;
        let mut node = RbfNode {
            txid,
            fee: Sats::ZERO,
            vsize: VSize::default(),
            value: Sats::ZERO,
            first_seen: Default::default(),
            rate: FeeRate::default(),
            unseen: false,
            rbf: false,
            replaced_non_signaling: false,
            full_rbf: false,
            truncated: false,
            replaces: Vec::new(),
        };
        if let Some(tombstone) = graveyard.get(&txid) {
            fill_record(&mut node, &tombstone.record, tombstone.chunk_rate);
        } else {
            node.first_seen = graveyard
                .predecessors_of(&txid)
                .next_back()?
                .1
                .removed_at
                .timestamp();
            node.unseen = true;
        }

        // Spend the budget on the requested path first, then newest siblings. Reverse once to
        // preserve mempool.space's oldest-first predecessor order on the wire.
        let next = self
            .path
            .get(depth + 1)
            .filter(|_| self.path.get(depth) == Some(&txid))
            .copied();
        let mut first = next.and_then(|next| self.build(next, depth + 1));
        for (predecessor, _) in graveyard.predecessors_of(&txid).rev() {
            if Some(*predecessor) == next {
                node.replaces.extend(first.take());
                continue;
            }
            if self.remaining == 0 || depth + 1 == MAX_RBF_DEPTH {
                self.truncated = true;
                break;
            }
            node.replaces.extend(self.build(*predecessor, depth + 1));
        }
        // If we stopped before reaching the path child, every sibling built is newer.
        node.replaces.extend(first);
        node.replaces.reverse();
        node.replaced_non_signaling = node.replaces.iter().any(|child| !child.rbf);
        node.full_rbf = if node.replaces.is_empty() {
            !node.rbf
        } else {
            node.replaced_non_signaling || node.replaces.iter().any(|child| child.full_rbf)
        };
        Some(node)
    }
}

fn fill_record(node: &mut RbfNode, record: &TxRecord, rate: FeeRate) {
    node.fee = record.entry.fee;
    node.vsize = record.entry.vsize;
    node.value = record.tx.output.iter().map(|output| output.value).sum();
    node.first_seen = record.entry.first_seen;
    node.rate = rate;
    node.unseen = false;
    // Tree flags use the transaction's own signals, as mempool.space does.
    node.rbf = signals_rbf(&record.tx);
}

fn fill_pending_root(state: &Pool, root: &mut RbfNode) {
    if let Some((record, rate)) = state.pending_head(&root.txid) {
        fill_record(root, record, rate);
    }
}
