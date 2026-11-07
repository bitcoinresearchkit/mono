//! CPFP (Child Pays For Parent) walk over a `Snapshot`'s adjacency.
//!
//! Three walks over one component:
//! - `ancestors`: DFS up `parents` only.
//! - `descendants`: DFS down `children` only.
//! - cluster: the connected component over `parents ∪ children`, in one order whatever the seed,
//!   linearized for wire shape and the seed's chunk feerate (the snapshot's rates the same).

use brk_cpfp::Cluster;
use brk_error::{Error, Result};
use brk_types::{CpfpClusterChunk, CpfpClusterTxIndex, CpfpEntry, CpfpInfo, SigOps, Txid};
use rustc_hash::{FxHashMap, FxHashSet};

use super::{SnapTx, Snapshot, TxIndex, cluster};
use crate::{PendingView, ReadOnlyState};

impl ReadOnlyState {
    /// CPFP info for a tx pending in `view`. Returns `None` when the tx
    /// isn't pending there. Requires the graph and live
    /// fields to share the same transaction revision and the view's chain tip. A tx whose own
    /// inputs aren't all resolved yet answers `StateUpdating`, alone (its sigops need them; its
    /// cluster's rates already stand on Core's vsize for any member still resolving). One whose
    /// cluster a newer block settled part of (the moment before the pool follows it) answers
    /// without what was settled, linearized again.
    pub fn cpfp_info(&self, txid: &Txid, view: &PendingView) -> Result<Option<CpfpInfo>> {
        let state = self.pool()?.at(view.tip())?;
        let snapshot = &state.graph;
        let Some(tx) = state.txs.get(txid).filter(|_| view.pending(txid)) else {
            return Ok(None);
        };
        if !tx.is_resolved() {
            return Err(Error::StateUpdating);
        }
        let Some(index) = snapshot.idx_of_txid(txid) else {
            return Ok(None);
        };
        if !view.settles_nothing() {
            let pending = |member: TxIndex| view.pending(&snapshot.txs[member.as_usize()].txid);
            if !cluster::walk(&snapshot.txs, index).into_iter().all(pending) {
                return Ok(Some(snapshot.cpfp_info_within(index, tx.total_sigop_cost, pending)));
            }
        }
        Ok(Some(snapshot.cpfp_info_at(index, tx.total_sigop_cost)))
    }
}

impl Snapshot {
    /// From this snapshot's linearization of the seed's component: a request maps its chunks
    /// onto the members' order, never linearizing again.
    pub(crate) fn cpfp_info_at(&self, seed_idx: TxIndex, sigops: SigOps) -> CpfpInfo {
        let seed = &self.txs[seed_idx.as_usize()];
        let mut graph = Cluster::default();
        let members = cluster::load(&self.txs, &cluster::walk(&self.txs, seed_idx), &mut graph);
        let local: FxHashMap<TxIndex, u32> = members
            .iter()
            .enumerate()
            .map(|(i, &m)| (m, i as u32))
            .collect();
        let seed_local = local[&seed_idx];
        let chunks = self
            .chunks
            .component_chunks(self.chunk_of[seed_idx.as_usize()] as usize)
            .map(|c| {
                // In the linearization's order, as Core lists a chunk.
                let txs: Vec<CpfpClusterTxIndex> = self
                    .chunks
                    .members(c)
                    .iter()
                    .map(|m| CpfpClusterTxIndex::from(local[m]))
                    .collect();
                CpfpClusterChunk {
                    txs,
                    feerate: self.chunks.rate(c),
                }
            })
            .collect();
        CpfpInfo::new(
            Self::collect_cpfp_entries(&self.txs, seed_idx, |t| &t.parents, |_| true),
            Self::collect_cpfp_entries(&self.txs, seed_idx, |t| &t.children, |_| true),
            seed.fee,
            seed.weight,
            sigops,
            graph.project_from(
                CpfpClusterTxIndex::from(seed_local),
                |local| {
                    let t = &self.txs[members[local].as_usize()];
                    (t.txid, t.weight)
                },
                chunks,
            ),
        )
    }

    /// [`Self::cpfp_info_at`] over the members `keep` accepts alone (what a newer block didn't
    /// settle): the piece of the seed's component left, linearized afresh.
    pub(crate) fn cpfp_info_within(
        &self,
        seed_idx: TxIndex,
        sigops: SigOps,
        keep: impl Fn(TxIndex) -> bool,
    ) -> CpfpInfo {
        let seed = &self.txs[seed_idx.as_usize()];
        let mut graph = Cluster::default();
        let members = cluster::load(
            &self.txs,
            &cluster::walk_within(&self.txs, seed_idx, &keep),
            &mut graph,
        );
        let seed_local = members
            .iter()
            .position(|&member| member == seed_idx)
            .expect("the seed is a member");
        // Ordered by modified fees, shown by what each chunk pays, as the snapshot's.
        let mut chunks = Vec::new();
        graph.linearize(|txs, _| {
            let paid = txs.iter().map(|&local| members[u32::from(local) as usize]);
            chunks.push(CpfpClusterChunk {
                txs: txs.to_vec(),
                feerate: cluster::paid_rate(&self.txs, paid),
            });
        });
        CpfpInfo::new(
            Self::collect_cpfp_entries(&self.txs, seed_idx, |t| &t.parents, &keep),
            Self::collect_cpfp_entries(&self.txs, seed_idx, |t| &t.children, &keep),
            seed.fee,
            seed.weight,
            sigops,
            graph.project_from(
                CpfpClusterTxIndex::from(seed_local as u32),
                |local| {
                    let t = &self.txs[members[local].as_usize()];
                    (t.txid, t.weight)
                },
                chunks,
            ),
        )
    }

    /// DFS from `seed` (exclusive) along `next` through the members `keep` accepts, lifted
    /// directly to wire-shape `CpfpEntry`s: the seed's ancestors or descendants, all of them (its
    /// component bounds them).
    fn collect_cpfp_entries(
        txs: &[SnapTx],
        seed: TxIndex,
        next: impl Fn(&SnapTx) -> &[TxIndex],
        keep: impl Fn(TxIndex) -> bool,
    ) -> Vec<CpfpEntry> {
        let mut visited: FxHashSet<TxIndex> = FxHashSet::default();
        visited.insert(seed);
        let mut out: Vec<CpfpEntry> = Vec::new();
        let mut stack: Vec<TxIndex> = next(&txs[seed.as_usize()]).to_vec();
        while let Some(idx) = stack.pop() {
            if !keep(idx) || !visited.insert(idx) {
                continue;
            }
            let t = &txs[idx.as_usize()];
            out.push(CpfpEntry::from(t));
            stack.extend(next(t).iter().copied());
        }
        out
    }
}
