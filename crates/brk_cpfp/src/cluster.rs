use std::ops::Range;

use brk_types::{CpfpClusterChunk, CpfpClusterTxIndex, FeeRate, Sats, VSize};

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

    /// Append the next member. Its `parents` must be local indices of earlier members.
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

    pub fn vsize(&self, local: usize) -> VSize {
        self.nodes[local].vsize
    }

    pub fn parents(&self, local: usize) -> &[CpfpClusterTxIndex] {
        &self.parents[self.nodes[local].parents.clone()]
    }

    /// Linearize into descending-feerate chunks and pass each chunk's members (ascending
    /// local indices) and feerate to `emit`, in chunk order. Order: the best remaining ancestor set
    /// first (exact feerate, then size, then earliest); a group paying more than the chunk
    /// before it merges into it.
    pub fn linearize(&mut self, emit: impl FnMut(&[CpfpClusterTxIndex], FeeRate)) {
        linearize(&self.nodes, &self.parents, &mut self.scratch, emit);
    }

    /// [`Cluster::linearize`] collected into owned chunks.
    pub fn chunks(&mut self) -> Vec<CpfpClusterChunk> {
        let mut chunks = Vec::new();
        self.linearize(|txs, feerate| {
            chunks.push(CpfpClusterChunk {
                txs: txs.to_vec(),
                feerate,
            })
        });
        chunks
    }
}
