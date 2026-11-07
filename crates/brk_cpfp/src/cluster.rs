use std::ops::Range;

use brk_types::{
    CpfpCluster, CpfpClusterChunk, CpfpClusterTx, CpfpClusterTxIndex, FeeRate, Sats, Txid, Weight,
};

use crate::linearize::{Scratch, linearize};
#[cfg(test)]
use crate::linearize::ancestor_sets;

/// One member: the fee it pays (served), the fee it ranks by (Core's modified fee: what it pays
/// plus an operator's delta, which can be negative), its weight, its place in the fallback order
/// that settles equal feerates (Core's: by txid) and its range in [`Cluster::parents`]' flat
/// buffer.
pub(crate) struct Node {
    pub fee: Sats,
    pub modified: i64,
    pub weight: Weight,
    pub fallback: u32,
    pub parents: Range<usize>,
}

/// A dependency component in topological order: each member's `(fee, weight)` and its
/// in-cluster parents as local indices of earlier members. Reuse one across components:
/// [`Cluster::clear`] keeps every buffer, the linearization scratch included.
#[derive(Default)]
pub struct Cluster {
    nodes: Vec<Node>,
    parents: Vec<CpfpClusterTxIndex>,
    scratch: Scratch,
}

impl Cluster {
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.parents.clear();
    }

    /// `weight` is what feerates are measured over (the mempool's sigop-adjusted, as Core's);
    /// `parents` must be local indices of earlier members. It ranks by what it pays.
    pub fn push(
        &mut self,
        fee: Sats,
        weight: Weight,
        parents: impl IntoIterator<Item = CpfpClusterTxIndex>,
    ) {
        let fallback = self.nodes.len() as u32;
        self.push_ranked(
            fee,
            i64::try_from(u64::from(fee)).unwrap(),
            weight,
            fallback,
            parents,
        );
    }

    /// [`Self::push`], ranked by `modified` instead (Core's modified fee), `fee` still served.
    /// `modified` is within [`Sats::MAX_MONEY`] either way (the caller clamps an operator's
    /// delta): a tier's sums then fit, 4,096 members at 21 million bitcoin each. `fallback`: its
    /// place among the members in the order settling equal feerates, distinct (Core's: by txid;
    /// [`Self::push`] takes the push order).
    pub fn push_ranked(
        &mut self,
        fee: Sats,
        modified: i64,
        weight: Weight,
        fallback: u32,
        parents: impl IntoIterator<Item = CpfpClusterTxIndex>,
    ) {
        let start = self.parents.len();
        self.parents.extend(parents);
        debug_assert!(
            self.parents[start..]
                .iter()
                .all(|&parent| (u32::from(parent) as usize) < self.nodes.len()),
            "CPFP parents must precede their children"
        );
        debug_assert!(
            modified.unsigned_abs() <= u64::from(Sats::MAX_MONEY),
            "a modified fee past MAX_MONEY"
        );
        self.nodes.push(Node {
            fee,
            modified,
            weight,
            fallback,
            parents: start..self.parents.len(),
        });
    }

    pub fn fee(&self, local: usize) -> Sats {
        self.nodes[local].fee
    }

    /// Linearize into chunks of non-increasing feerate and pass each chunk's members (in order)
    /// and feerate to `emit`, in chunk order: optimal up to 64 members, in Core 31's order, the
    /// best remaining ancestor sets up to 4,096, the push order chunked beyond (`linearize.rs`).
    pub fn linearize(&mut self, emit: impl FnMut(&[CpfpClusterTxIndex], FeeRate)) {
        linearize(&self.nodes, &self.parents, &mut self.scratch, emit);
    }

    /// [`Self::linearize`] through the ancestor-set tier, whatever the size.
    #[cfg(test)]
    pub(crate) fn linearize_ancestor_sets(
        &mut self,
        emit: impl FnMut(&[CpfpClusterTxIndex], FeeRate),
    ) {
        ancestor_sets(&self.nodes, &self.parents, &mut self.scratch, emit);
    }

    /// The cluster as served for the member `seed`, and the seed's effective feerate: that of
    /// the chunk holding it. A lone transaction has no cluster (mempool.space omits it) and its
    /// own feerate. `member(local)` is each member's txid and weight.
    pub fn project(
        &mut self,
        seed: CpfpClusterTxIndex,
        member: impl Fn(usize) -> (Txid, Weight),
    ) -> (Option<CpfpCluster>, FeeRate) {
        if self.nodes.len() == 1 {
            // A lone transaction's rate is what it pays (its ranked fee only orders).
            let node = &self.nodes[u32::from(seed) as usize];
            return (None, FeeRate::from((node.fee, node.weight)));
        }
        let mut chunks = Vec::new();
        self.linearize(|txs, feerate| {
            chunks.push(CpfpClusterChunk {
                txs: txs.to_vec(),
                feerate,
            });
        });
        self.project_from(seed, member, chunks)
    }

    /// [`Self::project`] from this cluster's linearization made already: its chunks in order,
    /// each its members' local indices in the linearization's order.
    pub fn project_from(
        &self,
        seed: CpfpClusterTxIndex,
        member: impl Fn(usize) -> (Txid, Weight),
        chunks: Vec<CpfpClusterChunk>,
    ) -> (Option<CpfpCluster>, FeeRate) {
        if self.nodes.len() == 1 {
            // A lone transaction's rate is what it pays (its ranked fee only orders).
            let node = &self.nodes[u32::from(seed) as usize];
            return (None, FeeRate::from((node.fee, node.weight)));
        }
        debug_assert_eq!(
            chunks.iter().map(|chunk| chunk.txs.len()).sum::<usize>(),
            self.nodes.len(),
            "every member in one chunk"
        );
        let (chunk_index, rate) = chunks
            .iter()
            .enumerate()
            .find(|(_, chunk)| chunk.txs.contains(&seed))
            .map(|(i, chunk)| (i as u32, chunk.feerate))
            .expect("every member is in a chunk");
        let txs = self
            .nodes
            .iter()
            .enumerate()
            .map(|(local, node)| {
                let (txid, weight) = member(local);
                CpfpClusterTx {
                    txid,
                    weight,
                    fee: node.fee,
                    parents: self.parents[node.parents.clone()].to_vec(),
                }
            })
            .collect();
        (
            Some(CpfpCluster {
                txs,
                chunks,
                chunk_index,
            }),
            rate,
        )
    }
}
