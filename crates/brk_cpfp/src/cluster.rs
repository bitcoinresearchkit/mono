use std::ops::Range;

use brk_types::{
    CpfpCluster, CpfpClusterChunk, CpfpClusterTx, CpfpClusterTxIndex, FeeRate, Sats, Txid, VSize,
    Weight,
};

use crate::linearize::{Scratch, linearize};

/// One member: its fee, size and range in [`Cluster::parents`]' flat buffer.
pub(crate) struct Node {
    pub fee: Sats,
    pub vsize: VSize,
    pub parents: Range<usize>,
}

/// A dependency component in topological order: each member's `(fee, vsize)` and its
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

    /// `parents` must be local indices of earlier members.
    pub fn push(
        &mut self,
        fee: Sats,
        vsize: VSize,
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
        self.nodes.push(Node {
            fee,
            vsize,
            parents: start..self.parents.len(),
        });
    }

    pub fn fee(&self, local: usize) -> Sats {
        self.nodes[local].fee
    }

    /// Linearize into descending-feerate chunks and pass each chunk's members (ascending
    /// local indices) and feerate to `emit`, in chunk order. Order: the best remaining ancestor set
    /// first (exact feerate, then more members, then earliest); a group paying more than the chunk
    /// before it merges into it.
    pub fn linearize(&mut self, emit: impl FnMut(&[CpfpClusterTxIndex], FeeRate)) {
        linearize(&self.nodes, &self.parents, &mut self.scratch, emit);
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
            let node = &self.nodes[u32::from(seed) as usize];
            return (None, FeeRate::from((node.fee, node.vsize)));
        }
        let mut chunks = Vec::new();
        let mut seed_chunk = None;
        self.linearize(|txs, feerate| {
            if txs.contains(&seed) {
                seed_chunk = Some((chunks.len() as u32, feerate));
            }
            chunks.push(CpfpClusterChunk {
                txs: txs.to_vec(),
                feerate,
            });
        });
        let (chunk_index, rate) = seed_chunk.expect("every member is in a chunk");
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
