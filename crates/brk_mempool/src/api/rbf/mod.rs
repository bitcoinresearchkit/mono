//! RBF trees captured from one publication for later confirmed-chain enrichment.

use brk_error::{Error, Result};
use brk_types::{BlockHash, FeeRate, Sats, Transaction, Txid};
use rustc_hash::FxHashSet;

use crate::{
    ReadOnlyState,
    snapshot::Snapshot,
    state::TxEntry,
    stores::{ReadOnlyTxStore, TxGraveyard},
};

pub mod for_tx;
pub mod node;

pub use for_tx::RbfForTx;
pub use node::RbfNode;

const MAX_RBF_WORK: usize = 4096;
// Also bounds later enrichment, JSON nesting and recursive destruction.
const MAX_RBF_DEPTH: usize = 32;

/// A history walk that ran past one of its bounds.
enum Exceeded {
    Work,
    Depth,
}

impl From<Exceeded> for Error {
    fn from(exceeded: Exceeded) -> Self {
        Error::Internal(match exceeded {
            Exceeded::Work => "RBF history exceeds traversal limit",
            Exceeded::Depth => "RBF history exceeds depth limit",
        })
    }
}

impl ReadOnlyState {
    /// Walk forward through `Replaced { by }` to the terminal replacer
    /// and return its full predecessor tree, plus the requested tx's
    /// direct predecessors from the acquired publication.
    /// Rejects histories exceeding 4096 traversal steps or 32 nested nodes;
    /// never returns a silently truncated tree.
    pub fn rbf_for_tx(&self, txid: &Txid, tip: &BlockHash) -> Result<RbfForTx> {
        let state = self.pool()?;
        state.ensure_resolved_at(tip)?;
        let mut remaining = MAX_RBF_WORK;
        let root_txid = state
            .graveyard
            .replacement_root_of(*txid, &mut remaining)
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
        let mut root =
            Self::build_rbf_node(&root_txid, &state.txs, &state.graveyard, &mut remaining, 0)?;
        if let Some(root) = root.as_mut() {
            Self::apply_snapshot_rates(root, &state.graph);
        }
        Ok(RbfForTx { root, replaces })
    }

    /// Recent terminal-replacer trees, most-recent first, deduplicated
    /// by root, capped at `limit`. `full_rbf_only` drops trees with no
    /// non-signaling predecessor.
    /// The whole request shares one work budget, charged for discarded
    /// candidates and stale graveyard entries too. Once it runs out the
    /// listing ends with the complete trees found so far; a tree nested
    /// deeper than `rbf_for_tx` allows is skipped, never truncated.
    pub fn recent_rbf_trees(
        &self,
        full_rbf_only: bool,
        limit: usize,
        tip: &BlockHash,
    ) -> Result<Vec<RbfNode>> {
        let state = self.pool()?;
        state.ensure_resolved_at(tip)?;
        if limit == 0 {
            return Ok(Vec::new());
        }
        let mut seen: FxHashSet<Txid> = FxHashSet::default();
        let mut remaining = MAX_RBF_WORK;
        let mut trees = Vec::new();
        for candidate in state.graveyard.replacements_recent_first() {
            let Some(left) = remaining.checked_sub(1) else {
                break;
            };
            remaining = left;
            let Some((_, by)) = candidate else { continue };
            let Some(root) = state.graveyard.replacement_root_of(*by, &mut remaining) else {
                break;
            };
            if !seen.insert(root) {
                continue;
            }
            match Self::build_rbf_node(&root, &state.txs, &state.graveyard, &mut remaining, 0) {
                Ok(Some(node)) if !full_rbf_only || node.full_rbf => {
                    trees.push(node);
                    if trees.len() == limit {
                        break;
                    }
                }
                Ok(_) | Err(Exceeded::Depth) => {}
                Err(Exceeded::Work) => break,
            }
        }
        // Resolve rates only for returned trees, after filtering and admission.
        for root in &mut trees {
            Self::apply_snapshot_rates(root, &state.graph);
        }
        Ok(trees)
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

        let full_rbf = replaces.iter().any(|c| !c.rbf || c.full_rbf);
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
            return Some((&record.tx, &record.entry, record.entry.fee_rate(), true));
        }
        graveyard.get(txid).map(|tombstone| {
            (
                tombstone.tx.as_ref(),
                &tombstone.entry,
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
