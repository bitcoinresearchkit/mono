//! Build per-tx adjacency from the live `TxStore`, then linearize every
//! multi-tx cluster.

use std::mem;

use brk_cpfp::Cluster;
use rustc_hash::{FxBuildHasher, FxHashMap};
use smallvec::SmallVec;

use super::{SnapTx, Snapshot, TxIndex, cluster};
use crate::{key::TxKey, state::TxEntry, stores::TxStore};

pub type KeyIndex = FxHashMap<TxKey, TxIndex>;

impl Snapshot {
    pub(crate) fn build_txs(txs: &TxStore) -> (Vec<SnapTx>, KeyIndex) {
        let n = txs.len();
        let mut key_to_idx = KeyIndex::with_capacity_and_hasher(n, FxBuildHasher);
        for (i, (key, _)) in txs.records().enumerate() {
            key_to_idx.insert(*key, TxIndex::from(i));
        }
        let mut snap_txs: Vec<SnapTx> = txs
            .records()
            .map(|(_, record)| Self::live_tx(&record.entry))
            .collect();
        // Parents from the bodies' inputs, as the pool stands now (a parent can leave or
        // return after its child arrived), each checked by its full txid.
        for (i, (_, record)) in txs.records().enumerate() {
            let mut parents: SmallVec<[TxIndex; 2]> = SmallVec::new();
            for input in &record.tx.input {
                if let Some(&parent) = key_to_idx.get(&TxKey::from(&input.txid))
                    && snap_txs[parent.as_usize()].txid == input.txid
                    && !parents.contains(&parent)
                {
                    parents.push(parent);
                }
            }
            snap_txs[i].parents = parents;
        }

        Self::mirror_children(&mut snap_txs);
        Self::refresh_chunk_rates(&mut snap_txs);
        (snap_txs, key_to_idx)
    }

    fn live_tx(e: &TxEntry) -> SnapTx {
        SnapTx {
            txid: e.txid,
            fee: e.fee,
            vsize: e.vsize,
            weight: e.weight,
            size: e.size,
            chunk_rate: e.fee_rate(),
            parents: SmallVec::new(),
            children: SmallVec::new(),
        }
    }

    fn mirror_children(txs: &mut [SnapTx]) {
        for i in 0..txs.len() {
            let child = TxIndex::from(i);
            let parents = mem::take(&mut txs[i].parents);
            for &p in &parents {
                txs[p.as_usize()].children.push(child);
            }
            txs[i].parents = parents;
        }
    }

    /// Walk every multi-tx connected component once and overwrite each
    /// member's `chunk_rate` with the linearized chunk's feerate.
    fn refresh_chunk_rates(snap_txs: &mut [SnapTx]) {
        let n = snap_txs.len();
        let mut visited = vec![false; n];
        let mut graph = Cluster::default();
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
            debug_assert!(component.len() > 1);
            let members = cluster::load(snap_txs, &component, &mut graph);
            graph.linearize(|txs, feerate| {
                for &local in txs {
                    let m = members[u32::from(local) as usize];
                    snap_txs[m.as_usize()].chunk_rate = feerate;
                }
            });
        }
    }
}
