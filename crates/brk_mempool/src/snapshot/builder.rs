//! Build per-tx adjacency from the live `TxStore`, then linearize every component once: once in
//! its lifetime, as a component unchanged since the last rebuild keeps its linearization.

use std::mem;

use brk_cpfp::Cluster;
use brk_types::{Txid, Weight};
use rustc_hash::{FxBuildHasher, FxHashMap};
use smallvec::SmallVec;

use super::{SnapTx, Snapshot, TxIndex, cluster, partition::Chunks};
use crate::{
    key::TxKey,
    stores::{TxStore, tx_store::TxRecord},
};

pub type KeyIndex = FxHashMap<TxKey, TxIndex>;

/// A component member's identity for the cache: its key, policy weight and modified fee.
type MemberId = (TxKey, Weight, i64);

/// Each multi-transaction component's linearization from the last rebuild, by its members (keys,
/// policy weights and modified fees, sorted): those fix its graph (a txid fixes inputs and fee; a
/// witness variant or a prevout fill can change the weight, an operator's delta the modified
/// fee), so an unchanged one is reused as is. Only the components of the latest rebuild are kept.
#[derive(Default)]
pub struct ChunkCache(FxHashMap<Box<[MemberId]>, Linearized>);

impl ChunkCache {
    /// `component`'s linearization, as positions in `members`: last rebuild's when its members are
    /// unchanged, solved otherwise. `kept` receives it for the next rebuild, and it comes back
    /// from there.
    pub(super) fn linearized<'a>(
        &mut self,
        txs: &[SnapTx],
        component: &[TxIndex],
        members: &mut Vec<(MemberId, TxIndex)>,
        graph: &mut Cluster,
        kept: &'a mut Self,
    ) -> &'a Linearized {
        members.clear();
        members.extend(component.iter().map(|&m| {
            let t = &txs[m.as_usize()];
            ((TxKey::from(&t.txid), t.sigops.adjust_weight(t.weight), t.modified), m)
        }));
        members.sort_unstable_by_key(|&(id, _)| id);
        let id: Box<[MemberId]> = members.iter().map(|&(id, _)| id).collect();
        let linearized = self
            .0
            .remove(&id)
            .unwrap_or_else(|| Snapshot::linearize_component(txs, component, members, graph));
        kept.0.entry(id).insert_entry(linearized).into_mut()
    }
}

pub(super) struct Linearized {
    /// Chunk after chunk, each member's position among the key's members, in topological order.
    order: Box<[u32]>,
    /// Each chunk's end in `order` (its rate is what its members pay, read when used).
    chunks: Box<[u32]>,
}

impl Linearized {
    /// Each chunk's members, as positions among the key's members.
    pub(super) fn chunks(&self) -> impl Iterator<Item = &[u32]> {
        let mut start = 0;
        self.chunks.iter().map(move |&end| {
            let positions = &self.order[start..end as usize];
            start = end as usize;
            positions
        })
    }
}

impl Snapshot {
    /// The pool's transactions, then `template_only` (block 0's bodies the pool lacks), indexed
    /// in that order. A template-only one stands alone, linked to nothing: it left Core's pool,
    /// so no pooled transaction's chunk or CPFP answer counts it. `modified_fees`: authoritative
    /// signed amounts, in txid byte order.
    pub(crate) fn build_txs(
        txs: &TxStore,
        template_only: &[TxRecord],
        modified_fees: &[(Txid, i64)],
        cache: &mut ChunkCache,
    ) -> (Vec<SnapTx>, KeyIndex, Chunks) {
        let records = || {
            txs.records()
                .map(|(key, record)| (*key, record))
                .chain(template_only.iter().map(|record| (record.entry.key(), record)))
        };
        let n = txs.len() + template_only.len();
        let mut key_to_idx = KeyIndex::with_capacity_and_hasher(n, FxBuildHasher);
        let mut keys: Vec<TxKey> = Vec::with_capacity(n);
        for (i, (key, _)) in records().enumerate() {
            // A pooled key keeps its index: a template-only body sharing it (a hash collision)
            // finds no index of its own and leaves block 0 short.
            key_to_idx.entry(key).or_insert(TxIndex::from(i));
            keys.push(key);
        }
        let modified_of = |txid: &Txid| {
            modified_fees
                .binary_search_by(|(other, _)| other.cmp(txid))
                .ok()
                .map(|i| modified_fees[i].1)
        };
        let mut snap_txs = Vec::with_capacity(n);
        snap_txs.extend(records().map(|(_, record)| {
            Self::live_tx(record, modified_of(&record.entry.txid))
        }));
        // Parents from the bodies' inputs, as the pool stands now (a parent can leave or
        // return after its child arrived), each checked by its full txid.
        let pooled = txs.len();
        for (i, (_, record)) in txs.records().enumerate() {
            let mut parents: SmallVec<[TxIndex; 2]> = SmallVec::new();
            for input in &record.tx.input {
                if let Some(&parent) = key_to_idx.get(&TxKey::from(&input.txid))
                    && parent.as_usize() < pooled
                    && snap_txs[parent.as_usize()].txid == input.txid
                    && !parents.contains(&parent)
                {
                    parents.push(parent);
                }
            }
            snap_txs[i].parents = parents;
        }

        Self::mirror_children(&mut snap_txs);
        let chunks = Self::linearize(&mut snap_txs, cache);
        (snap_txs, key_to_idx, chunks)
    }

    fn live_tx(record: &TxRecord, modified: Option<i64>) -> SnapTx {
        let e = &record.entry;
        let modified = modified.unwrap_or_else(|| i64::try_from(u64::from(e.fee)).expect("bounded base fee"));
        SnapTx {
            txid: e.txid,
            fee: e.fee,
            modified,
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
    fn linearize(snap_txs: &mut [SnapTx], cache: &mut ChunkCache) -> Chunks {
        let n = snap_txs.len();
        let mut chunks = Chunks::default();
        let mut visited = vec![false; n];
        let mut graph = Cluster::default();
        let mut kept = ChunkCache::default();
        let mut members: Vec<(MemberId, TxIndex)> = Vec::new();
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
            for &m in &component {
                visited[m.as_usize()] = true;
            }
            let linearized = cache.linearized(snap_txs, &component, &mut members, &mut graph, &mut kept);
            for positions in linearized.chunks() {
                let txs = positions.iter().map(|&p| members[p as usize].1);
                // Ordered by modified fees, shown by what the chunk pays.
                let rate = cluster::paid_rate(snap_txs, txs.clone());
                for m in txs.clone() {
                    snap_txs[m.as_usize()].chunk_rate = rate;
                }
                chunks.push_chunk(txs, rate);
            }
            chunks.end_component();
        }
        *cache = kept;
        chunks
    }

    /// `members`: the component's ids, sorted, each with its transaction.
    fn linearize_component(
        snap_txs: &[SnapTx],
        component: &[TxIndex],
        members: &[(MemberId, TxIndex)],
        graph: &mut Cluster,
    ) -> Linearized {
        let position: FxHashMap<TxIndex, u32> = members
            .iter()
            .enumerate()
            .map(|(p, &(_, m))| (m, p as u32))
            .collect();
        let locals = cluster::load(snap_txs, component, graph);
        let mut order = Vec::with_capacity(component.len());
        let mut chunks = Vec::new();
        graph.linearize(|txs, _| {
            order.extend(
                txs.iter()
                    .map(|&local| position[&locals[u32::from(local) as usize]]),
            );
            chunks.push(order.len() as u32);
        });
        Linearized {
            order: order.into(),
            chunks: chunks.into(),
        }
    }
}
