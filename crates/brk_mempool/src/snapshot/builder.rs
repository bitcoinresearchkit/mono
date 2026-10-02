//! Build per-tx adjacency from the live `TxStore`, then run Single Fee
//! Linearization over every multi-tx cluster.

use std::mem;

use brk_types::TxidPrefix;
use derive_more::{Deref, DerefMut};
use rustc_hash::{FxBuildHasher, FxHashMap};
use smallvec::SmallVec;

use super::{SnapTx, Snapshot, TxIndex, cluster};
use crate::{state::TxEntry, stores::TxStore};

#[derive(Default, Deref, DerefMut)]
pub struct PrefixIndex(FxHashMap<TxidPrefix, TxIndex>);

impl Snapshot {
    pub(crate) fn build_txs(txs: &TxStore) -> (Vec<SnapTx>, PrefixIndex) {
        let n = txs.len();
        let mut prefix_to_idx = PrefixIndex(FxHashMap::with_capacity_and_hasher(n, FxBuildHasher));
        for (i, (prefix, _)) in txs.records().enumerate() {
            prefix_to_idx.insert(*prefix, TxIndex::from(i));
        }
        let mut snap_txs: Vec<SnapTx> = txs
            .records()
            .map(|(_, record)| Self::live_tx(&record.entry, &prefix_to_idx))
            .collect();

        Self::mirror_children(&mut snap_txs);
        Self::refresh_chunk_rates(&mut snap_txs);
        (snap_txs, prefix_to_idx)
    }

    fn live_tx(e: &TxEntry, prefix_to_idx: &PrefixIndex) -> SnapTx {
        let parents: SmallVec<[TxIndex; 2]> = e
            .depends
            .iter()
            .filter_map(|p| prefix_to_idx.get(p).copied())
            .collect();
        SnapTx {
            txid: e.txid,
            fee: e.fee,
            vsize: e.vsize,
            weight: e.weight,
            size: e.size,
            chunk_rate: e.fee_rate(),
            parents,
            children: SmallVec::new(),
        }
    }

    fn mirror_children(txs: &mut [SnapTx]) {
        for i in 0..txs.len() {
            let child = TxIndex::from(i);
            let parents = mem::take(&mut txs[i].parents);
            for &p in &parents {
                if let Some(t) = txs.get_mut(p.as_usize()) {
                    t.children.push(child);
                }
            }
            txs[i].parents = parents;
        }
    }

    /// Walk every multi-tx connected component once and overwrite each
    /// member's `chunk_rate` with the linearized chunk's feerate.
    /// Visited bitmap ensures each cluster is linearized exactly once.
    fn refresh_chunk_rates(snap_txs: &mut [SnapTx]) {
        let n = snap_txs.len();
        let mut visited = vec![false; n];
        for seed in 0..n {
            if visited[seed] {
                continue;
            }
            let t = &snap_txs[seed];
            if t.parents.is_empty() && t.children.is_empty() {
                visited[seed] = true;
                continue;
            }
            let component = cluster::walk(snap_txs, TxIndex::from(seed));
            for &m in &component {
                visited[m.as_usize()] = true;
            }
            if component.len() <= 1 {
                continue;
            }
            let (members, chunks) = cluster::linearize(snap_txs, &component);
            for chunk in &chunks {
                for &local in &chunk.txs {
                    let m = members[u32::from(local) as usize];
                    snap_txs[m.as_usize()].chunk_rate = chunk.feerate;
                }
            }
        }
    }
}
