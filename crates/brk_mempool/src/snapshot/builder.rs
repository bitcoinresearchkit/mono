//! Build per-tx adjacency from the live `TxStore`, then linearize every component once: once in
//! its lifetime, as a component unchanged since the last rebuild keeps its linearization.

use std::mem;

use brk_cpfp::Cluster;
use brk_types::{FeeRate, Weight};
use rustc_hash::{FxBuildHasher, FxHashMap};
use smallvec::SmallVec;

use super::{SnapTx, Snapshot, TxIndex, cluster, partition::Chunks};
use crate::{
    key::TxKey,
    stores::{TxStore, tx_store::TxRecord},
};

pub type KeyIndex = FxHashMap<TxKey, TxIndex>;

/// Each multi-transaction component's linearization from the last rebuild, by its members (keys
/// and policy weights, sorted): those fix its graph (a txid fixes inputs and fee; a witness
/// variant or a prevout fill can change the weight), so an unchanged one is reused as is. Only
/// the components of the latest rebuild are kept.
#[derive(Default)]
pub struct ChunkCache(FxHashMap<Box<[(TxKey, Weight)]>, Linearized>);

struct Linearized {
    /// Chunk after chunk, each member's position among the key's members, in topological order.
    order: Box<[u32]>,
    /// Each chunk's end in `order` and feerate.
    chunks: Box<[(u32, FeeRate)]>,
}

impl Snapshot {
    pub(crate) fn build_txs(
        txs: &TxStore,
        cache: &mut ChunkCache,
    ) -> (Vec<SnapTx>, KeyIndex, Chunks) {
        let n = txs.len();
        let mut key_to_idx = KeyIndex::with_capacity_and_hasher(n, FxBuildHasher);
        let mut keys: Vec<TxKey> = Vec::with_capacity(n);
        for (i, (key, _)) in txs.records().enumerate() {
            key_to_idx.insert(*key, TxIndex::from(i));
            keys.push(*key);
        }
        let mut snap_txs: Vec<SnapTx> = txs
            .records()
            .map(|(_, record)| Self::live_tx(record))
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
        let chunks = Self::linearize(&mut snap_txs, &keys, cache);
        (snap_txs, key_to_idx, chunks)
    }

    fn live_tx(record: &TxRecord) -> SnapTx {
        let e = &record.entry;
        SnapTx {
            txid: e.txid,
            fee: e.fee,
            vsize: e.vsize,
            weight: e.weight,
            size: e.size,
            sigops: record.policy_sigops(),
            chunk_rate: record.own_rate(),
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

    /// Linearize every component: each member's `chunk_rate` becomes its chunk's feerate (a
    /// lone transaction keeps its own), and the chunks are returned for the projected blocks.
    fn linearize(snap_txs: &mut [SnapTx], keys: &[TxKey], cache: &mut ChunkCache) -> Chunks {
        let n = snap_txs.len();
        let mut chunks = Chunks::default();
        let mut visited = vec![false; n];
        let mut graph = Cluster::default();
        let mut kept: FxHashMap<Box<[(TxKey, Weight)]>, Linearized> = FxHashMap::default();
        let mut members: Vec<((TxKey, Weight), TxIndex)> = Vec::new();
        for seed in 0..n {
            if visited[seed] {
                continue;
            }
            let t = &snap_txs[seed];
            if t.parents.is_empty() && t.children.is_empty() {
                visited[seed] = true;
                chunks.push_chunk([TxIndex::from(seed)], t.chunk_rate);
                chunks.end_component();
                continue;
            }
            let component = cluster::walk(snap_txs, TxIndex::from(seed));
            members.clear();
            for &m in &component {
                visited[m.as_usize()] = true;
                let t = &snap_txs[m.as_usize()];
                members.push(((keys[m.as_usize()], t.sigops.adjust_weight(t.weight)), m));
            }
            members.sort_unstable_by_key(|&(id, _)| id);
            let id: Box<[(TxKey, Weight)]> = members.iter().map(|&(id, _)| id).collect();
            let linearized = cache.0.remove(&id).unwrap_or_else(|| {
                Self::linearize_component(snap_txs, &component, &members, &mut graph)
            });
            let mut start = 0;
            for &(end, rate) in &linearized.chunks {
                let positions = &linearized.order[start..end as usize];
                let txs = positions.iter().map(|&p| members[p as usize].1);
                for m in txs.clone() {
                    snap_txs[m.as_usize()].chunk_rate = rate;
                }
                chunks.push_chunk(txs, rate);
                start = end as usize;
            }
            chunks.end_component();
            kept.insert(id, linearized);
        }
        cache.0 = kept;
        chunks
    }

    /// `members`: the component's ids, sorted, each with its transaction.
    fn linearize_component(
        snap_txs: &[SnapTx],
        component: &[TxIndex],
        members: &[((TxKey, Weight), TxIndex)],
        graph: &mut Cluster,
    ) -> Linearized {
        let position: FxHashMap<TxIndex, u32> =
            members.iter().enumerate().map(|(p, &(_, m))| (m, p as u32)).collect();
        let locals = cluster::load(snap_txs, component, graph);
        let mut order = Vec::with_capacity(component.len());
        let mut chunks = Vec::new();
        graph.linearize(|txs, rate| {
            order.extend(txs.iter().map(|&local| position[&locals[u32::from(local) as usize]]));
            chunks.push((order.len() as u32, rate));
        });
        Linearized {
            order: order.into(),
            chunks: chunks.into(),
        }
    }
}
